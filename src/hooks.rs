//! フック実行システムモジュール
//!
//! このモジュールの役割：
//! - pre_create/post_create/pre_remove/post_remove フックの実行
//! - フック実行時のエラーハンドリングと継続/中断制御
//! - フック実行ログの表示
//! - 環境変数の設定と引数の展開

#![allow(dead_code)]
use crate::core::{HookCommand, TwinError, TwinResult};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Output};

/// フックのタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookType {
    /// 環境作成前に実行
    PreCreate,
    /// 環境作成後に実行
    PostCreate,
    /// 環境削除前に実行
    PreRemove,
    /// 環境削除後に実行
    PostRemove,
}

impl HookType {
    /// フックタイプを文字列として取得
    pub fn as_str(&self) -> &str {
        match self {
            HookType::PreCreate => "pre_create",
            HookType::PostCreate => "post_create",
            HookType::PreRemove => "pre_remove",
            HookType::PostRemove => "post_remove",
        }
    }
}

/// フック実行の結果
#[derive(Debug)]
pub struct HookResult {
    /// フックタイプ
    pub hook_type: HookType,
    /// コマンド文字列
    pub command: String,
    /// 実行成功したか
    pub success: bool,
    /// 終了コード
    pub exit_code: Option<i32>,
    /// 標準出力
    pub stdout: String,
    /// 標準エラー出力
    pub stderr: String,
    /// 実行時間（ミリ秒）
    pub duration_ms: u128,
}

/// フック実行のコンテキスト情報
#[derive(Debug, Clone)]
pub struct HookContext {
    /// エージェント名
    pub agent_name: String,
    /// ワークツリーのパス
    pub worktree_path: PathBuf,
    /// ブランチ名
    pub branch: String,
    /// プロジェクトルートパス
    pub project_root: PathBuf,
    /// 追加の環境変数
    pub env_vars: HashMap<String, String>,
}

impl HookContext {
    /// 新しいコンテキストを作成
    pub fn new(
        agent_name: impl Into<String>,
        worktree_path: impl Into<PathBuf>,
        branch: impl Into<String>,
        project_root: impl Into<PathBuf>,
    ) -> Self {
        Self {
            agent_name: agent_name.into(),
            worktree_path: worktree_path.into(),
            branch: branch.into(),
            project_root: project_root.into(),
            env_vars: HashMap::new(),
        }
    }

    /// 環境変数を追加
    pub fn add_env_var(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.env_vars.insert(key.into(), value.into());
    }

    /// コンテキストを環境変数として取得
    pub fn as_env_vars(&self) -> HashMap<String, String> {
        let mut vars = self.env_vars.clone();

        // 標準のコンテキスト変数を追加
        vars.insert("TWIN_AGENT_NAME".to_string(), self.agent_name.clone());
        vars.insert(
            "TWIN_WORKTREE_PATH".to_string(),
            self.worktree_path.display().to_string(),
        );
        vars.insert("TWIN_BRANCH".to_string(), self.branch.clone());
        vars.insert(
            "TWIN_PROJECT_ROOT".to_string(),
            self.project_root.display().to_string(),
        );

        vars
    }
}

/// フック実行マネージャー
pub struct HookExecutor {
    /// ドライランモード
    dry_run: bool,
    /// タイムアウト（秒）
    timeout_seconds: u64,
    /// エラー時に続行するか
    continue_on_error: bool,
}

impl HookExecutor {
    /// 新しいフック実行マネージャーを作成
    pub fn new() -> Self {
        Self {
            dry_run: false,
            timeout_seconds: 30,
            continue_on_error: false,
        }
    }

    /// ドライランモードを設定
    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.dry_run = dry_run;
    }

    /// タイムアウトを設定
    pub fn set_timeout(&mut self, seconds: u64) {
        self.timeout_seconds = seconds;
    }

    /// エラー時の継続設定
    pub fn set_continue_on_error(&mut self, continue_on_error: bool) {
        self.continue_on_error = continue_on_error;
    }

    /// フックを実行
    pub fn execute(
        &self,
        hook_type: HookType,
        hook: &HookCommand,
        context: &HookContext,
    ) -> TwinResult<HookResult> {
        info!("Executing {} hook: {}", hook_type.as_str(), hook.command);

        // コマンドと引数を展開
        let expanded_command = self.expand_command(&hook.command, context);
        let expanded_args = if hook.args.is_empty() {
            None
        } else {
            Some(
                hook.args
                    .iter()
                    .map(|arg| self.expand_command(arg, context))
                    .collect::<Vec<_>>(),
            )
        };

        if self.dry_run {
            info!("[DRY RUN] Would execute: {}", expanded_command);
            if let Some(args) = &expanded_args {
                info!("[DRY RUN] With args: {:?}", args);
            }

            return Ok(HookResult {
                hook_type,
                command: expanded_command,
                success: true,
                exit_code: Some(0),
                stdout: "[DRY RUN]".to_string(),
                stderr: String::new(),
                duration_ms: 0,
            });
        }

        // 実際にコマンドを実行
        let start_time = std::time::Instant::now();
        let result = self.execute_command(&expanded_command, expanded_args.as_deref(), context)?;
        let duration_ms = start_time.elapsed().as_millis();

        let hook_result = HookResult {
            hook_type,
            command: expanded_command.clone(),
            success: result.status.success(),
            exit_code: result.status.code(),
            stdout: String::from_utf8_lossy(&result.stdout).to_string(),
            stderr: String::from_utf8_lossy(&result.stderr).to_string(),
            duration_ms,
        };

        // ログ出力
        if hook_result.success {
            info!(
                "{} hook completed successfully in {}ms",
                hook_type.as_str(),
                duration_ms
            );
            if !hook_result.stdout.is_empty() {
                debug!("Hook stdout: {}", hook_result.stdout);
            }
        } else {
            error!(
                "{} hook failed with exit code {:?}",
                hook_type.as_str(),
                hook_result.exit_code
            );
            if !hook_result.stderr.is_empty() {
                error!("Hook stderr: {}", hook_result.stderr);
            }

            // エラー時の処理
            if !self.continue_on_error {
                return Err(TwinError::hook(
                    format!("{} hook failed: {}", hook_type.as_str(), expanded_command),
                    hook_type.as_str().to_string(),
                    hook_result.exit_code,
                ));
            } else {
                warn!("Continuing despite hook failure (continue_on_error=true)");
            }
        }

        Ok(hook_result)
    }

    /// 複数のフックを順次実行
    pub fn execute_hooks(
        &self,
        hook_type: HookType,
        hooks: &[HookCommand],
        context: &HookContext,
    ) -> TwinResult<Vec<HookResult>> {
        let mut results = Vec::new();

        for hook in hooks {
            match self.execute(hook_type, hook, context) {
                Ok(result) => {
                    let should_stop = !result.success && !self.continue_on_error;
                    results.push(result);

                    if should_stop {
                        break;
                    }
                }
                Err(e) => {
                    if !self.continue_on_error {
                        return Err(e);
                    }
                    warn!("Hook execution error (continuing): {}", e);
                }
            }
        }

        Ok(results)
    }

    /// コマンド内の変数を展開
    fn expand_command(&self, command: &str, context: &HookContext) -> String {
        let mut result = command.to_string();

        // 基本的な変数展開
        result = result.replace("${AGENT_NAME}", &context.agent_name);
        result = result.replace(
            "${WORKTREE_PATH}",
            &context.worktree_path.display().to_string(),
        );
        result = result.replace("${BRANCH}", &context.branch);
        result = result.replace(
            "${PROJECT_ROOT}",
            &context.project_root.display().to_string(),
        );

        // 追加の環境変数を展開
        for (key, value) in &context.env_vars {
            result = result.replace(&format!("${{{}}}", key), value);
        }

        result
    }

    /// 実際にコマンドを実行
    fn execute_command(
        &self,
        command: &str,
        args: Option<&[String]>,
        context: &HookContext,
    ) -> TwinResult<Output> {
        let mut cmd = if cfg!(windows) {
            let mut c = Command::new("cmd");
            c.arg("/C");
            c
        } else {
            let mut c = Command::new("sh");
            c.arg("-c");
            c
        };

        // コマンド文字列を構築
        let full_command = if let Some(args) = args {
            format!("{} {}", command, args.join(" "))
        } else {
            command.to_string()
        };

        cmd.arg(&full_command);

        // 作業ディレクトリを設定
        cmd.current_dir(&context.worktree_path);

        // 環境変数を設定
        for (key, value) in context.as_env_vars() {
            cmd.env(key, value);
        }

        debug!("Executing command: {}", full_command);
        debug!("Working directory: {:?}", context.worktree_path);

        // タイムアウトを考慮した実行
        let output = if self.timeout_seconds > 0 {
            // タイムアウト付き実行（簡易実装）
            // 実際のプロダクションコードではtokio::time::timeoutなどを使用
            cmd.output().map_err(|e| {
                TwinError::hook(
                    format!("Failed to execute hook command: {}", e),
                    command.to_string(),
                    None,
                )
            })?
        } else {
            cmd.output().map_err(|e| {
                TwinError::hook(
                    format!("Failed to execute hook command: {}", e),
                    command.to_string(),
                    None,
                )
            })?
        };

        Ok(output)
    }
}

/// デフォルトのフック実行マネージャーを作成
impl Default for HookExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_type_string() {
        assert_eq!(HookType::PreCreate.as_str(), "pre_create");
        assert_eq!(HookType::PostCreate.as_str(), "post_create");
        assert_eq!(HookType::PreRemove.as_str(), "pre_remove");
        assert_eq!(HookType::PostRemove.as_str(), "post_remove");
    }

    #[test]
    fn test_context_env_vars() {
        let mut context = HookContext::new(
            "test-agent",
            "/path/to/worktree",
            "feature/test",
            "/path/to/project",
        );
        context.add_env_var("CUSTOM_VAR", "custom_value");

        let env_vars = context.as_env_vars();
        assert_eq!(env_vars.get("TWIN_AGENT_NAME").unwrap(), "test-agent");
        assert_eq!(env_vars.get("TWIN_BRANCH").unwrap(), "feature/test");
        assert_eq!(env_vars.get("CUSTOM_VAR").unwrap(), "custom_value");
    }

    #[test]
    fn test_command_expansion() {
        let context = HookContext::new(
            "my-agent",
            "/workspace/my-agent",
            "feature/my-agent",
            "/workspace",
        );

        let executor = HookExecutor::new();
        let expanded = executor.expand_command(
            "echo 'Working on ${AGENT_NAME} in ${WORKTREE_PATH}'",
            &context,
        );

        assert_eq!(
            expanded,
            "echo 'Working on my-agent in /workspace/my-agent'"
        );
    }

    #[test]
    fn test_dry_run_execution() {
        let mut executor = HookExecutor::new();
        executor.set_dry_run(true);

        let context = HookContext::new("test", "/test", "test", "/test");

        let hook = HookCommand {
            command: "echo test".to_string(),
            args: vec![],
            env: HashMap::new(),
            timeout: 60,
            continue_on_error: false,
        };

        let result = executor
            .execute(HookType::PreCreate, &hook, &context)
            .unwrap();
        assert!(result.success);
        assert_eq!(result.stdout, "[DRY RUN]");
    }
}

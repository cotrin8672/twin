#![allow(dead_code)]
/// Git操作モジュール
///
/// このモジュールの役割：
/// - git worktree add/remove/list コマンドのラッパー
/// - ブランチの作成と管理
/// - 自動コミット機能の実装
/// - Gitリポジトリの状態確認
use crate::core::{TwinError, TwinResult};
use chrono::{DateTime, Local};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Worktreeの情報を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    /// Worktreeのパス
    pub path: PathBuf,
    /// チェックアウトされているブランチ名
    pub branch: String,
    /// コミットハッシュ
    pub commit: String,
    /// エージェント名（ブランチ名から抽出）
    pub agent_name: Option<String>,
    /// 作成日時
    pub created_at: Option<DateTime<Local>>,
    /// 最終更新日時
    pub last_updated: Option<DateTime<Local>>,
    /// ロック状態
    pub locked: bool,
    /// プルーニング可能かどうか
    pub prunable: bool,
}

/// ブランチの情報を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    /// ブランチ名
    pub name: String,
    /// リモートブランチ名
    pub remote: Option<String>,
    /// 現在のブランチかどうか
    pub current: bool,
    /// コミットハッシュ
    pub commit: String,
    /// 上流ブランチとの差分
    pub ahead: usize,
    pub behind: usize,
}

/// Git操作を管理する構造体
pub struct GitManager {
    /// リポジトリのルートパス
    repo_path: PathBuf,
    /// git2ライブラリのリポジトリインスタンス（オプション）
    repository: Option<git2::Repository>,
    /// 実行履歴の記録
    command_history: Vec<String>,
    /// ドライラン モード
    dry_run: bool,
}

impl GitManager {
    /// 新しいGitManagerインスタンスを作成
    pub fn new(repo_path: &Path) -> TwinResult<Self> {
        let repo_path = repo_path.to_path_buf();

        // git2ライブラリを使用してリポジトリを開く
        let repository = match git2::Repository::open(&repo_path) {
            Ok(repo) => {
                info!("Opened Git repository at: {:?}", repo_path);
                Some(repo)
            }
            Err(e) => {
                warn!("Failed to open repository with git2: {}", e);
                // git2で開けない場合でも、gitコマンドは使える可能性があるので続行
                None
            }
        };

        // gitコマンドが使用可能か確認
        Self::verify_git_available()?;

        Ok(Self {
            repo_path,
            repository,
            command_history: Vec::new(),
            dry_run: false,
        })
    }

    /// ドライランモードを設定
    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.dry_run = dry_run;
    }

    /// gitコマンドが使用可能か確認
    fn verify_git_available() -> TwinResult<()> {
        let output = Command::new("git")
            .arg("--version")
            .output()
            .map_err(|e| TwinError::git(format!("Git command not found: {}", e)))?;

        if !output.status.success() {
            return Err(TwinError::git("Git command failed to execute"));
        }

        Ok(())
    }

    /// gitコマンドを実行する共通メソッド
    fn execute_git_command(&mut self, args: &[&str]) -> TwinResult<Output> {
        let command_str = format!("git {}", args.join(" "));
        info!("Executing: {}", command_str);
        self.command_history.push(command_str.clone());

        // 透明性のあるコマンド実行ログ
        if std::env::var("TWIN_VERBOSE").is_ok() || std::env::var("TWIN_DEBUG").is_ok() {
            eprintln!("🔧 実行中: {}", command_str);
        }

        if self.dry_run {
            info!("[DRY RUN] Would execute: {}", command_str);
            if std::env::var("TWIN_VERBOSE").is_ok() || std::env::var("TWIN_DEBUG").is_ok() {
                eprintln!("📝 ドライラン: {}", command_str);
            }
            return Ok(Output {
                #[cfg(unix)]
                status: std::os::unix::process::ExitStatusExt::from_raw(0),
                #[cfg(windows)]
                status: std::os::windows::process::ExitStatusExt::from_raw(0),
                stdout: b"[DRY RUN]".to_vec(),
                stderr: Vec::new(),
            });
        }

        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args(args)
            .output()
            .map_err(|e| TwinError::git(format!("Failed to execute git command: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TwinError::git(format!("Git command failed: {}", stderr)));
        }

        Ok(output)
    }

    /// Worktreeを追加
    pub fn add_worktree(
        &mut self,
        path: &Path,
        branch: Option<&str>,
        create_branch: bool,
    ) -> TwinResult<WorktreeInfo> {
        let mut args = vec!["worktree", "add"];

        // 新しいブランチを作成する場合
        if create_branch && let Some(b) = branch {
            args.push("-b");
            args.push(b);
        }

        // パスを追加
        let path_str = path.to_string_lossy();
        args.push(&path_str);

        // 既存のブランチを指定する場合
        if !create_branch && let Some(b) = branch {
            args.push(b);
        }

        let output = self.execute_git_command(&args)?;
        debug!(
            "Worktree added: {:?}",
            String::from_utf8_lossy(&output.stdout)
        );

        // 作成されたWorktreeの情報を取得
        self.get_worktree_info(path)
    }

    /// Worktreeを削除
    pub fn remove_worktree(&mut self, path: &Path, force: bool) -> TwinResult<()> {
        let mut args = vec!["worktree", "remove"];

        if force {
            args.push("--force");
        }

        let path_str = path.to_string_lossy();
        args.push(&path_str);

        self.execute_git_command(&args)?;
        info!("Worktree removed: {:?}", path);

        Ok(())
    }

    /// Worktreeの一覧を取得
    pub fn list_worktrees(&mut self) -> TwinResult<Vec<WorktreeInfo>> {
        let output = self.execute_git_command(&["worktree", "list", "--porcelain"])?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        self.parse_worktree_list(&stdout)
    }

    /// Worktreeリストの出力をパース
    fn parse_worktree_list(&self, output: &str) -> TwinResult<Vec<WorktreeInfo>> {
        let mut worktrees = Vec::new();
        let mut current_worktree: Option<WorktreeInfo> = None;

        for line in output.lines() {
            if line.starts_with("worktree ") {
                // 前のworktree情報を保存
                if let Some(wt) = current_worktree.take() {
                    worktrees.push(wt);
                }

                // 新しいworktree情報を開始
                let path = PathBuf::from(line.strip_prefix("worktree ").unwrap());
                current_worktree = Some(WorktreeInfo {
                    path,
                    branch: String::new(),
                    commit: String::new(),
                    agent_name: None,
                    created_at: None,
                    last_updated: None,
                    locked: false,
                    prunable: false,
                });
            } else if let Some(ref mut wt) = current_worktree {
                if line.starts_with("HEAD ") {
                    wt.commit = line.strip_prefix("HEAD ").unwrap().to_string();
                } else if line.starts_with("branch ") {
                    wt.branch = line.strip_prefix("branch ").unwrap().to_string();
                    // エージェント名を抽出（例: agent/claude -> claude）
                    if wt.branch.starts_with("agent/") {
                        wt.agent_name = Some(wt.branch[6..].to_string());
                    }
                } else if line == "locked" {
                    wt.locked = true;
                } else if line == "prunable" {
                    wt.prunable = true;
                }
            }
        }

        // 最後のworktree情報を保存
        if let Some(wt) = current_worktree {
            worktrees.push(wt);
        }

        Ok(worktrees)
    }

    /// 特定のWorktreeの情報を取得
    pub fn get_worktree_info(&mut self, path: &Path) -> TwinResult<WorktreeInfo> {
        let worktrees = self.list_worktrees()?;

        // パスを絶対パスに変換して比較
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| TwinError::io(format!("Failed to get current dir: {}", e), None))?
                .join(path)
        };

        // 正規化されたパスでも検索を試みる
        let canonical_path = abs_path.canonicalize().ok();

        worktrees
            .into_iter()
            .find(|wt| {
                // 直接比較
                wt.path == path ||
                wt.path == abs_path ||
                // 正規化されたパスとの比較
                canonical_path.as_ref().is_some_and(|cp| {
                    wt.path.canonicalize().ok().is_some_and(|wtp| wtp == *cp)
                }) ||
                // ファイル名だけでも一致を確認（最後の手段）
                wt.path.file_name() == path.file_name() && path.file_name().is_some()
            })
            .ok_or_else(|| TwinError::not_found("Worktree", path.to_string_lossy().to_string()))
    }

    /// プルーニング可能なWorktreeをクリーンアップ
    pub fn prune_worktrees(&mut self, dry_run: bool) -> TwinResult<Vec<PathBuf>> {
        let mut args = vec!["worktree", "prune"];

        if dry_run {
            args.push("--dry-run");
        }

        let output = self.execute_git_command(&args)?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        // プルーニングされたWorktreeのパスを抽出
        let pruned: Vec<PathBuf> = stdout
            .lines()
            .filter_map(|line| {
                if line.contains("Removing worktrees") {
                    Some(PathBuf::from(line.rsplit(":").next()?.trim()))
                } else {
                    None
                }
            })
            .collect();

        Ok(pruned)
    }

    /// ブランチを作成
    pub fn create_branch(
        &mut self,
        branch_name: &str,
        start_point: Option<&str>,
    ) -> TwinResult<()> {
        let mut args = vec!["branch", branch_name];

        if let Some(start) = start_point {
            args.push(start);
        }

        self.execute_git_command(&args)?;
        info!("Branch created: {}", branch_name);

        Ok(())
    }

    /// ブランチを削除
    pub fn delete_branch(&mut self, branch_name: &str, force: bool) -> TwinResult<()> {
        let mut args = vec!["branch"];

        if force {
            args.push("-D");
        } else {
            args.push("-d");
        }

        args.push(branch_name);

        self.execute_git_command(&args)?;
        info!("Branch deleted: {}", branch_name);

        Ok(())
    }

    /// ブランチの一覧を取得
    pub fn list_branches(&mut self, remote: bool) -> TwinResult<Vec<BranchInfo>> {
        let mut args = vec!["branch", "-v"];

        if remote {
            args.push("-r");
        } else {
            args.push("-a");
        }

        let output = self.execute_git_command(&args)?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        self.parse_branch_list(&stdout)
    }

    /// ブランチリストの出力をパース
    fn parse_branch_list(&self, output: &str) -> TwinResult<Vec<BranchInfo>> {
        let mut branches = Vec::new();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let current = line.starts_with('*');
            let line = if current { &line[2..] } else { line };

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }

            let name = parts[0].to_string();
            let commit = parts[1].to_string();

            branches.push(BranchInfo {
                name,
                remote: None,
                current,
                commit,
                ahead: 0,
                behind: 0,
            });
        }

        Ok(branches)
    }


    /// ブランチが存在するか確認
    pub fn branch_exists(&mut self, branch_name: &str) -> TwinResult<bool> {
        let branches = self.list_branches(false)?;
        Ok(branches.iter().any(|b| b.name == branch_name))
    }

    /// ユニークなブランチ名を生成（既存のブランチと重複しないように）
    pub fn generate_unique_branch_name(
        &mut self,
        base_name: &str,
        max_attempts: usize,
    ) -> TwinResult<String> {
        // まず基本名を試す
        if !self.branch_exists(base_name)? {
            return Ok(base_name.to_string());
        }

        // 番号付きの名前を試す
        for i in 1..=max_attempts {
            let name = format!("{}-{}", base_name, i);
            if !self.branch_exists(&name)? {
                return Ok(name);
            }
        }

        // タイムスタンプ付きの名前を生成
        let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let name = format!("{}-{}", base_name, timestamp);

        if !self.branch_exists(&name)? {
            Ok(name)
        } else {
            Err(TwinError::git(format!(
                "Failed to generate unique branch name for: {}",
                base_name
            )))
        }
    }

    /// コマンド実行履歴を取得
    pub fn get_command_history(&self) -> &[String] {
        &self.command_history
    }

    /// コマンド実行履歴をクリア
    pub fn clear_command_history(&mut self) {
        self.command_history.clear();
    }

    /// リポジトリのルートパスを取得
    pub fn get_repo_path(&self) -> &Path {
        &self.repo_path
    }

    /// 現在のブランチ名を取得
    pub fn get_current_branch(&mut self) -> TwinResult<String> {
        let output = self.execute_git_command(&["rev-parse", "--abbrev-ref", "HEAD"])?;
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(branch)
    }


    /// cdコマンド文字列を生成
    pub fn generate_cd_command(&self, path: &Path) -> String {
        format!("cd \"{}\"", path.display())
    }

    /// シェル関数用のヘルパースクリプトを生成
    pub fn generate_shell_helper(&self, shell_type: ShellType) -> String {
        match shell_type {
            ShellType::Bash | ShellType::Zsh => r#"
# Twin worktree helper function
twin-switch() {
    if [ -z "$1" ]; then
        echo "Usage: twin-switch <agent-name>"
        return 1
    fi
    
    local path=$(twin switch "$1" --print-path)
    if [ $? -eq 0 ] && [ -n "$path" ]; then
        cd "$path"
        echo "Switched to agent: $1"
    else
        echo "Failed to switch to agent: $1"
        return 1
    fi
}

# Twin create and switch function
twin-create() {
    if [ -z "$1" ]; then
        echo "Usage: twin-create <agent-name>"
        return 1
    fi
    
    local path=$(twin create "$1" --print-path)
    if [ $? -eq 0 ] && [ -n "$path" ]; then
        cd "$path"
        echo "Created and switched to agent: $1"
    else
        echo "Failed to create agent: $1"
        return 1
    fi
}
"#
            .to_string(),
            ShellType::PowerShell => r#"
# Twin worktree helper function
function Twin-Switch {
    param(
        [Parameter(Mandatory=$true)]
        [string]$AgentName
    )
    
    $path = twin switch $AgentName --print-path
    if ($LASTEXITCODE -eq 0 -and $path) {
        Set-Location $path
        Write-Host "Switched to agent: $AgentName"
    } else {
        Write-Error "Failed to switch to agent: $AgentName"
    }
}

# Twin create and switch function
function Twin-Create {
    param(
        [Parameter(Mandatory=$true)]
        [string]$AgentName
    )
    
    $path = twin create $AgentName --print-path
    if ($LASTEXITCODE -eq 0 -and $path) {
        Set-Location $path
        Write-Host "Created and switched to agent: $AgentName"
    } else {
        Write-Error "Failed to create agent: $AgentName"
    }
}
"#
            .to_string(),
            ShellType::Fish => r#"
# Twin worktree helper function
function twin-switch
    if test -z "$argv[1]"
        echo "Usage: twin-switch <agent-name>"
        return 1
    end
    
    set -l path (twin switch $argv[1] --print-path)
    if test $status -eq 0; and test -n "$path"
        cd $path
        echo "Switched to agent: $argv[1]"
    else
        echo "Failed to switch to agent: $argv[1]"
        return 1
    end
end

# Twin create and switch function
function twin-create
    if test -z "$argv[1]"
        echo "Usage: twin-create <agent-name>"
        return 1
    end
    
    set -l path (twin create $argv[1] --print-path)
    if test $status -eq 0; and test -n "$path"
        cd $path
        echo "Created and switched to agent: $argv[1]"
    else
        echo "Failed to create agent: $argv[1]"
        return 1
    end
end
"#
            .to_string(),
        }
    }

    /// エイリアス設定を生成
    pub fn generate_aliases(&self, shell_type: ShellType) -> String {
        match shell_type {
            ShellType::Bash | ShellType::Zsh => r#"
# Twin aliases
alias tw='twin'
alias tws='twin-switch'
alias twc='twin-create'
alias twl='twin list'
alias twr='twin remove'
"#
            .to_string(),
            ShellType::PowerShell => r#"
# Twin aliases
Set-Alias -Name tw -Value twin
Set-Alias -Name tws -Value Twin-Switch
Set-Alias -Name twc -Value Twin-Create
Set-Alias -Name twl -Value 'twin list'
Set-Alias -Name twr -Value 'twin remove'
"#
            .to_string(),
            ShellType::Fish => r#"
# Twin aliases
alias tw='twin'
alias tws='twin-switch'
alias twc='twin-create'
alias twl='twin list'
alias twr='twin remove'
"#
            .to_string(),
        }
    }
}

/// サポートされているシェルタイプ
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

impl ShellType {
    /// 現在のシェルを検出
    pub fn detect() -> Option<Self> {
        if cfg!(target_os = "windows") {
            // Windows環境ではPowerShellをデフォルトとする
            return Some(ShellType::PowerShell);
        }

        // Unix系環境では$SHELL環境変数を確認
        if let Ok(shell) = std::env::var("SHELL") {
            if shell.contains("bash") {
                Some(ShellType::Bash)
            } else if shell.contains("zsh") {
                Some(ShellType::Zsh)
            } else if shell.contains("fish") {
                Some(ShellType::Fish)
            } else {
                // デフォルトはBash
                Some(ShellType::Bash)
            }
        } else {
            None
        }
    }

    /// シェルタイプの文字列表現
    pub fn as_str(&self) -> &str {
        match self {
            ShellType::Bash => "bash",
            ShellType::Zsh => "zsh",
            ShellType::Fish => "fish",
            ShellType::PowerShell => "powershell",
        }
    }
}

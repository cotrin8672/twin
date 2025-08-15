//! CLIの出力フォーマット機能
use anyhow::{Result, anyhow};
use std::path::Path;

use crate::git::WorktreeInfo;

/// 出力フォーマッタークラス
pub struct OutputFormatter {
    format: OutputFormat,
}

impl OutputFormatter {
    /// 新しいフォーマッターを作成
    pub fn new(format_str: &str) -> Self {
        let format = OutputFormat::from_str(format_str).unwrap_or(OutputFormat::Table);
        Self { format }
    }

    pub fn format_worktrees(&self, worktrees: &[WorktreeInfo]) -> Result<()> {
        format_worktrees(worktrees, &self.format)
    }
}

/// 出力フォーマット
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Table,
    Json,
    Simple,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "table" => Ok(OutputFormat::Table),
            "json" => Ok(OutputFormat::Json),
            "simple" => Ok(OutputFormat::Simple),
            _ => Err(anyhow!("Invalid output format: {}", s)),
        }
    }
}

/// パス出力（cdコマンド用）
#[allow(dead_code)]
pub fn format_path_output(path: &Path, show_cd_command: bool) -> Result<()> {
    if show_cd_command {
        // cdコマンドとして出力
        let path_str = path.to_string_lossy();

        // パスにスペースが含まれる場合はクォートで囲む
        if path_str.contains(' ') {
            println!("cd \"{}\"", path_str);
        } else {
            println!("cd {}", path_str);
        }
    } else {
        // パスのみ出力
        println!("{}", path.to_string_lossy());
    }

    Ok(())
}

/// Worktree一覧を指定されたフォーマットで出力
pub fn format_worktrees(worktrees: &[WorktreeInfo], format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Table => format_worktrees_table(worktrees),
        OutputFormat::Json => format_worktrees_json(worktrees),
        OutputFormat::Simple => format_worktrees_simple(worktrees),
    }
}

/// Worktreeをテーブル形式で出力
fn format_worktrees_table(worktrees: &[WorktreeInfo]) -> Result<()> {
    if worktrees.is_empty() {
        println!("No worktrees found.");
        return Ok(());
    }

    // メインリポジトリと作業ツリーを分けて表示
    // 通常、最初のエントリーがメインリポジトリ（bare でない限り）
    let (main_repo, work_trees) = if !worktrees.is_empty() {
        // 最初のエントリーか、現在のディレクトリと同じパスのものをメインリポジトリとする
        let current_dir = std::env::current_dir().ok();
        let is_main = |w: &WorktreeInfo| -> bool {
            if let Some(ref cwd) = current_dir {
                // 現在のディレクトリと一致するか確認
                w.path.canonicalize().ok() == cwd.canonicalize().ok()
            } else {
                false
            }
        };
        
        if let Some(main_idx) = worktrees.iter().position(is_main) {
            let mut all: Vec<WorktreeInfo> = worktrees.to_vec();
            let main = all.remove(main_idx);
            (Some(main), all)
        } else {
            // 現在のディレクトリと一致するものがない場合、最初のエントリーをメインとする
            let mut all: Vec<WorktreeInfo> = worktrees.to_vec();
            if !all.is_empty() {
                let main = all.remove(0);
                (Some(main), all)
            } else {
                (None, vec![])
            }
        }
    } else {
        (None, vec![])
    };

    // メインリポジトリの表示
    if let Some(main) = main_repo {
        println!("📁 Main Repository");
        println!("  Branch: {}", if main.branch.is_empty() { "(no branch)" } else { &main.branch });
        println!("  Path:   {}", main.path.to_string_lossy());
        println!("  Commit: {}", &main.commit[..8.min(main.commit.len())]);
        println!();
    }

    // ワークツリーの表示
    if !work_trees.is_empty() {
        println!("🌲 Work Trees");
        println!("{}", "-".repeat(80));
        
        // ヘッダー
        println!("{:<30} {:<10} {:<12} {:<30}", "Branch", "Status", "Commit", "Path");
        println!("{}", "-".repeat(80));

        // Worktree一覧
        for wt in work_trees.iter() {
            let status = if wt.locked {
                "🔒 locked"
            } else if wt.prunable {
                "⚠️  prunable"
            } else {
                "✓ active"
            };

            let branch_display = if wt.branch.is_empty() { 
                "(no branch)".to_string()
            } else if wt.branch.len() > 28 {
                format!("{}...", &wt.branch[..25])
            } else {
                wt.branch.clone()
            };

            let path_display = {
                let path_str = wt.path.to_string_lossy();
                if path_str.len() > 28 {
                    // パスが長い場合は最後の部分を表示
                    if let Some(file_name) = wt.path.file_name() {
                        format!(".../{}", file_name.to_string_lossy())
                    } else {
                        format!("...{}", &path_str[path_str.len()-25..])
                    }
                } else {
                    path_str.to_string()
                }
            };

            println!(
                "{:<30} {:<10} {:<12} {:<30}",
                branch_display,
                status,
                &wt.commit[..8.min(wt.commit.len())],
                path_display
            );
        }
        
        println!("{}", "-".repeat(80));
        println!("Total: {} worktree(s)", work_trees.len());
    }

    Ok(())
}

/// WorktreeをJSON形式で出力
fn format_worktrees_json(worktrees: &[WorktreeInfo]) -> Result<()> {
    let json = serde_json::to_string_pretty(worktrees)?;
    println!("{}", json);
    Ok(())
}

/// Worktreeをシンプル形式で出力
fn format_worktrees_simple(worktrees: &[WorktreeInfo]) -> Result<()> {
    for wt in worktrees {
        if wt.branch.is_empty() {
            println!("(no branch)");
        } else {
            println!("{}", wt.branch);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_from_str() {
        assert!(matches!(
            OutputFormat::from_str("table"),
            Ok(OutputFormat::Table)
        ));
        assert!(matches!(
            OutputFormat::from_str("json"),
            Ok(OutputFormat::Json)
        ));
        assert!(matches!(
            OutputFormat::from_str("simple"),
            Ok(OutputFormat::Simple)
        ));
        assert!(OutputFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_output_format_from_str_case_insensitive() {
        assert!(matches!(
            OutputFormat::from_str("TABLE"),
            Ok(OutputFormat::Table)
        ));
        assert!(matches!(
            OutputFormat::from_str("Json"),
            Ok(OutputFormat::Json)
        ));
        assert!(matches!(
            OutputFormat::from_str("SIMPLE"),
            Ok(OutputFormat::Simple)
        ));
    }

    #[test]
    fn test_output_formatter_new() {
        let _formatter = OutputFormatter::new("table");
        // OutputFormatterがTableフォーマットで作成される

        let _formatter_invalid = OutputFormatter::new("invalid");
        // 無効な形式の場合はデフォルト（Table）にフォールバック
    }
}

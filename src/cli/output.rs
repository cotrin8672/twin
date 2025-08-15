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

    // ヘッダー
    println!("{:<20} {:<15} {:<50}", "Branch", "Status", "Path");
    println!("{}", "-".repeat(85));

    // Worktree一覧
    for wt in worktrees {
        let status = if wt.locked {
            "locked"
        } else if wt.prunable {
            "prunable"
        } else {
            "normal"
        };

        println!(
            "{:<20} {:<15} {:<50}",
            if wt.branch.is_empty() { "(no branch)" } else { &wt.branch },
            status,
            wt.path.to_string_lossy()
        );
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

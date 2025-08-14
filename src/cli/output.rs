use anyhow::{Result, anyhow};
use chrono::{DateTime, Local, Utc};
use serde_json;
use std::path::Path;

use crate::core::{AgentEnvironment, EnvironmentStatus};

/// 出力フォーマッタークラス
pub struct OutputFormatter {
    format: OutputFormat,
}

impl OutputFormatter {
    pub fn new(format_str: &str) -> Self {
        let format = OutputFormat::from_str(format_str).unwrap_or(OutputFormat::Table);
        Self { format }
    }

    pub fn format_environments(&self, environments: &[AgentEnvironment]) -> Result<()> {
        let env_refs: Vec<&AgentEnvironment> = environments.iter().collect();
        format_environments(&env_refs, &self.format, None)
    }
}

/// 出力フォーマットの種類
#[derive(Debug, Clone)]
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
            _ => Err(anyhow!(
                "Invalid output format: {}. Use 'table', 'json', or 'simple'",
                s
            )),
        }
    }
}

/// 環境一覧を指定されたフォーマットで出力
pub fn format_environments(
    environments: &[&AgentEnvironment],
    format: &OutputFormat,
    active_name: Option<&str>,
) -> Result<()> {
    match format {
        OutputFormat::Table => format_table(environments, active_name),
        OutputFormat::Json => format_json(environments),
        OutputFormat::Simple => format_simple(environments, active_name),
    }
}

/// テーブル形式で出力
fn format_table(environments: &[&AgentEnvironment], active_name: Option<&str>) -> Result<()> {
    if environments.is_empty() {
        println!("No environments found.");
        return Ok(());
    }

    // ヘッダー
    println!(
        "{:<2} {:<15} {:<20} {:<12} {:<10}",
        "", "Name", "Branch", "Created", "Status"
    );
    println!("{}", "-".repeat(65));

    // 環境一覧
    for env in environments {
        let active_marker = if Some(env.name.as_str()) == active_name {
            "*"
        } else {
            " "
        };
        let created = format_datetime(&env.created_at);
        let status = format_status(&env.status);

        println!(
            "{:<2} {:<15} {:<20} {:<12} {:<10}",
            active_marker, env.name, env.branch, created, status
        );
    }

    println!();
    if let Some(active) = active_name {
        println!("* Active environment: {}", active);
    }

    Ok(())
}

/// JSON形式で出力
fn format_json(environments: &[&AgentEnvironment]) -> Result<()> {
    let json = serde_json::to_string_pretty(environments)?;
    println!("{}", json);
    Ok(())
}

/// シンプル形式で出力
fn format_simple(environments: &[&AgentEnvironment], active_name: Option<&str>) -> Result<()> {
    for env in environments {
        let active_marker = if Some(env.name.as_str()) == active_name {
            " (active)"
        } else {
            ""
        };
        println!("{}{}", env.name, active_marker);
    }
    Ok(())
}

/// パス出力（cdコマンド用）
pub fn format_path_output(path: &Path, show_cd_command: bool) -> Result<()> {
    if show_cd_command {
        // シェル検出
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

        if shell.contains("fish") {
            println!("cd '{}'", path.display());
        } else if shell.contains("zsh") || shell.contains("bash") {
            println!("cd '{}'", path.display());
        } else {
            // デフォルト
            println!("cd '{}'", path.display());
        }

        println!();
        println!("To change directory, run:");
        println!(
            "  $(twin switch {} --cd-command)",
            path.file_name().unwrap_or_default().to_string_lossy()
        );
    } else {
        println!("{}", path.display());
    }

    Ok(())
}

/// 日時をフォーマット
fn format_datetime(dt: &DateTime<Utc>) -> String {
    let local: DateTime<Local> = dt.with_timezone(&Local);
    local.format("%m/%d %H:%M").to_string()
}

/// ステータスをフォーマット
fn format_status(status: &EnvironmentStatus) -> String {
    match status {
        EnvironmentStatus::Active => "Active".to_string(),
        EnvironmentStatus::Inactive => "Inactive".to_string(),
        EnvironmentStatus::Creating => "Creating".to_string(),
        EnvironmentStatus::Removing => "Removing".to_string(),
        EnvironmentStatus::Error(msg) => format!("Error: {}", msg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::AgentEnvironment;
    use std::path::PathBuf;

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
    fn test_format_status() {
        assert_eq!(format_status(&EnvironmentStatus::Active), "Active");
        assert_eq!(format_status(&EnvironmentStatus::Inactive), "Inactive");
        assert_eq!(format_status(&EnvironmentStatus::Creating), "Creating");
        assert_eq!(format_status(&EnvironmentStatus::Removing), "Removing");
        assert_eq!(
            format_status(&EnvironmentStatus::Error("test".to_string())),
            "Error: test"
        );
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
    fn test_format_table_empty() {
        let environments: Vec<&AgentEnvironment> = vec![];
        let result = format_table(&environments, None);

        assert!(result.is_ok());
        // 空の時は "No environments found." が出力される
    }

    #[test]
    fn test_format_json_empty() {
        let environments: Vec<&AgentEnvironment> = vec![];
        let result = format_json(&environments);

        assert!(result.is_ok());
        // JSON形式では空配列 [] が出力される
    }

    #[test]
    fn test_format_simple_empty() {
        let environments: Vec<&AgentEnvironment> = vec![];
        let result = format_simple(&environments, None);

        assert!(result.is_ok());
        // シンプル形式では何も出力されない
    }

    #[test]
    fn test_format_datetime_formatting() {
        use chrono::{TimeZone, Utc};

        // 2024年1月15日 14:30:00 UTC
        let dt = Utc.with_ymd_and_hms(2024, 1, 15, 14, 30, 0).unwrap();
        let formatted = format_datetime(&dt);

        // フォーマットは "MM/DD HH:MM" 形式
        // ローカルタイムゾーンに依存するため、形式のみチェック
        assert!(formatted.contains("/"));
        assert!(formatted.contains(":"));
        assert!(formatted.len() >= 11); // "01/15 14:30" の最小長
    }

    #[test]
    fn test_output_formatter_new() {
        let formatter = OutputFormatter::new("table");
        // OutputFormatterがTableフォーマットで作成される

        let formatter_invalid = OutputFormatter::new("invalid");
        // 無効な形式の場合はデフォルト（Table）にフォールバック
    }

    #[test]
    fn test_format_environments_with_data() {
        use chrono::Utc;

        let env1 = AgentEnvironment {
            name: "test1".to_string(),
            branch: "feature/test1".to_string(),
            worktree_path: PathBuf::from("/tmp/test1"),
            symlinks: vec![],
            status: EnvironmentStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            config_path: None,
        };

        let env2 = AgentEnvironment {
            name: "test2".to_string(),
            branch: "feature/test2".to_string(),
            worktree_path: PathBuf::from("/tmp/test2"),
            symlinks: vec![],
            status: EnvironmentStatus::Inactive,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            config_path: None,
        };

        let environments = vec![&env1, &env2];

        // Table形式
        let result_table = format_table(&environments, Some("test1"));
        assert!(result_table.is_ok());

        // JSON形式
        let result_json = format_json(&environments);
        assert!(result_json.is_ok());

        // Simple形式
        let result_simple = format_simple(&environments, Some("test1"));
        assert!(result_simple.is_ok());
    }
}

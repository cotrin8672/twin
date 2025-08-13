use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, Utc};
use serde_json;
use std::path::Path;

use crate::core::{AgentEnvironment, EnvironmentStatus};

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
            _ => Err(anyhow!("Invalid output format: {}. Use 'table', 'json', or 'simple'", s)),
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
    println!("{:<2} {:<15} {:<20} {:<12} {:<10}", "", "Name", "Branch", "Created", "Status");
    println!("{}", "-".repeat(65));

    // 環境一覧
    for env in environments {
        let active_marker = if Some(env.name.as_str()) == active_name { "*" } else { " " };
        let created = format_datetime(&env.created_at);
        let status = format_status(&env.status);
        
        println!(
            "{:<2} {:<15} {:<20} {:<12} {:<10}",
            active_marker,
            env.name,
            env.branch,
            created,
            status
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
        let active_marker = if Some(env.name.as_str()) == active_name { " (active)" } else { "" };
        println!("{}{}", env.name, active_marker);
    }
    Ok(())
}

/// パス出力（cdコマンド用）
pub fn format_path_output(path: &Path, show_cd_command: bool) -> Result<()> {
    if show_cd_command {
        // シェル検出
        let shell = std::env::var("SHELL")
            .unwrap_or_else(|_| "/bin/sh".to_string());
        
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
        println!("  $(twin switch {} --cd-command)", path.file_name().unwrap_or_default().to_string_lossy());
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
        assert!(matches!(OutputFormat::from_str("table"), Ok(OutputFormat::Table)));
        assert!(matches!(OutputFormat::from_str("json"), Ok(OutputFormat::Json)));
        assert!(matches!(OutputFormat::from_str("simple"), Ok(OutputFormat::Simple)));
        assert!(OutputFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_format_status() {
        assert_eq!(format_status(&EnvironmentStatus::Active), "Active");
        assert_eq!(format_status(&EnvironmentStatus::Inactive), "Inactive");
        assert_eq!(format_status(&EnvironmentStatus::Creating), "Creating");
        assert_eq!(format_status(&EnvironmentStatus::Removing), "Removing");
        assert_eq!(format_status(&EnvironmentStatus::Error("test".to_string())), "Error: test");
    }
}
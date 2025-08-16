use anyhow::Result;
use std::path::PathBuf;
use tempfile::TempDir;
use twin_cli::cli::commands::handle_init;
use twin_cli::cli::InitArgs;

#[tokio::test]
async fn test_init_command_creates_config_file() -> Result<()> {
    // 一時ディレクトリを作成
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("twin.toml");

    // InitArgsを作成
    let args = InitArgs {
        path: Some(config_path.clone()),
        force: false,
    };

    // コマンドを実行
    handle_init(args).await?;

    // ファイルが作成されたことを確認
    assert!(config_path.exists());

    // ファイルの内容を読み込んで検証
    let content = tokio::fs::read_to_string(&config_path).await?;
    assert!(content.contains("[files]"));
    assert!(content.contains("[hooks]"));

    Ok(())
}

#[tokio::test]
async fn test_init_command_with_force_flag() -> Result<()> {
    // 一時ディレクトリを作成
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("twin.toml");

    // 最初のinitを実行
    let args = InitArgs {
        path: Some(config_path.clone()),
        force: false,
    };
    handle_init(args).await?;

    // カスタム内容でファイルを上書き
    tokio::fs::write(&config_path, "# custom content\n").await?;

    // forceフラグ付きで再実行
    let args = InitArgs {
        path: Some(config_path.clone()),
        force: true,
    };
    handle_init(args).await?;

    // ファイルが新しい設定で上書きされたことを確認
    let content = tokio::fs::read_to_string(&config_path).await?;
    assert!(!content.starts_with("# custom content"));
    assert!(content.contains("[files]"));

    Ok(())
}

#[tokio::test]
async fn test_init_command_error_when_file_exists() -> Result<()> {
    // 一時ディレクトリを作成
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("twin.toml");

    // 最初のinitを実行
    let args = InitArgs {
        path: Some(config_path.clone()),
        force: false,
    };
    handle_init(args).await?;

    // 2回目のinitは失敗するはず（forceなし）
    let args = InitArgs {
        path: Some(config_path.clone()),
        force: false,
    };
    let result = handle_init(args).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));

    Ok(())
}

#[tokio::test]
async fn test_init_command_default_filename() -> Result<()> {
    use std::env;

    // 一時ディレクトリを作成
    let temp_dir = TempDir::new()?;
    let original_dir = env::current_dir()?;

    // 作業ディレクトリを変更
    env::set_current_dir(temp_dir.path())?;

    // パスを指定せずにinitを実行
    let args = InitArgs {
        path: None,
        force: false,
    };
    handle_init(args).await?;

    // デフォルトのファイル名が使われることを確認
    let default_path = PathBuf::from("twin.toml");
    assert!(default_path.exists());

    // 元のディレクトリに戻す
    env::set_current_dir(original_dir)?;

    Ok(())
}

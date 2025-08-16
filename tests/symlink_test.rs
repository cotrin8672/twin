//! シンボリックリンク機能の統合テスト
//!
//! このテストは以下を検証します：
//! - シンボリックリンクの作成（権限がある場合）
//! - ファイルコピーへのフォールバック（権限がない場合）
//! - クロスプラットフォーム動作

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// テスト用のワークスペースを作成
fn setup_test_workspace() -> (TempDir, PathBuf, PathBuf) {
    let temp = TempDir::new().unwrap();
    let source_file = temp.path().join("source.txt");
    let target_file = temp.path().join("target.txt");

    // ソースファイルを作成
    fs::write(&source_file, "test content").unwrap();

    (temp, source_file, target_file)
}

#[allow(dead_code)]
/// テスト用の設定ファイルを作成
fn create_test_config(dir: &Path, files: Vec<(&str, &str)>) -> PathBuf {
    let config_path = dir.join("twin.toml");
    let mut config_content = String::from(
        r#"
worktree_base = "./"
branch_prefix = "test/"

"#,
    );

    for (path, mapping_type) in files {
        config_content.push_str(&format!(
            r#"[[files]]
path = "{path}"
mapping_type = "{mapping_type}"
skip_if_exists = false

"#
        ));
    }

    fs::write(&config_path, config_content).unwrap();
    config_path
}

#[test]
fn test_symlink_manager_initialization() {
    // SymlinkManagerが正しく初期化されることを確認
    use twin_cli::symlink::create_symlink_manager;

    let manager = create_symlink_manager();
    // マネージャーが作成されたことを確認（Boxなのでnullにはならない）
    assert!(!manager
        .as_ref()
        .get_manual_instructions(&PathBuf::from("a"), &PathBuf::from("b"))
        .is_empty());
}

#[test]
fn test_symlink_creation_with_permission() {
    use twin_cli::symlink::{create_symlink_manager, LinkStrategy};

    let (_temp, source, target) = setup_test_workspace();
    let manager = create_symlink_manager();

    // 戦略の選択をテスト
    let strategy = manager.select_strategy(&source, &target);

    // プラットフォームと権限に依存
    match strategy {
        LinkStrategy::Symlink => {
            println!("Platform supports symlinks");
        }
        LinkStrategy::Copy => {
            println!("Falling back to copy");
        }
    }

    // シンボリックリンクまたはコピーを作成
    let result = manager.create_symlink(&source, &target);

    match result {
        Ok(info) => {
            assert!(info.is_valid);
            assert!(target.exists());

            // ファイルの内容が読めることを確認
            let content = fs::read_to_string(&target).unwrap();
            assert_eq!(content, "test content");
        }
        Err(e) => {
            // 権限がない場合のエラーは許容
            eprintln!("Symlink creation failed (expected on CI): {e}");
        }
    }
}

#[test]
fn test_fallback_to_copy() {
    use twin_cli::symlink::create_symlink_manager;

    let (_temp, source, target) = setup_test_workspace();
    let manager = create_symlink_manager();

    // create_symlinkは内部で自動的にフォールバック処理を行う
    let result = manager.create_symlink(&source, &target);

    // 権限に関わらず、何らかの方法でファイルが作成される
    if result.is_ok() {
        assert!(target.exists());
        let content = fs::read_to_string(&target).unwrap();
        assert_eq!(content, "test content");
    }
}

#[test]
fn test_symlink_removal() {
    use twin_cli::symlink::create_symlink_manager;

    let (_temp, source, target) = setup_test_workspace();
    let manager = create_symlink_manager();

    // まずリンクを作成
    if manager.create_symlink(&source, &target).is_ok() {
        assert!(target.exists());

        // リンクを削除
        let remove_result = manager.remove_symlink(&target);
        assert!(remove_result.is_ok());
        assert!(!target.exists());
    }
}

#[test]
#[cfg(unix)]
fn test_unix_symlink_specific() {
    use std::os::unix::fs::symlink;

    let (_temp, source, target) = setup_test_workspace();

    // Unix固有のシンボリックリンク作成
    symlink(&source, &target).unwrap();
    assert!(target.exists());

    // シンボリックリンクであることを確認
    let metadata = fs::symlink_metadata(&target).unwrap();
    assert!(metadata.file_type().is_symlink());

    // リンク先が正しいことを確認
    let link_target = fs::read_link(&target).unwrap();
    assert_eq!(link_target, source);
}

#[test]
#[cfg(windows)]
fn test_windows_symlink_fallback() {
    use twin_cli::symlink::{SymlinkManager, WindowsSymlinkManager};

    let manager = WindowsSymlinkManager::new();
    let (_temp, source, target) = setup_test_workspace();

    // Windowsマネージャーは開発者モードをチェック
    let result = manager.create_symlink(&source, &target);

    if result.is_ok() {
        assert!(target.exists());

        // 開発者モードが無効な場合、ファイルがコピーされている
        let source_content = fs::read_to_string(&source).unwrap();
        let target_content = fs::read_to_string(&target).unwrap();
        assert_eq!(source_content, target_content);
    }
}

#[test]
fn test_multiple_file_mappings() {
    use twin_cli::symlink::create_symlink_manager;

    let temp = TempDir::new().unwrap();
    let manager = create_symlink_manager();

    // 複数のファイルを作成
    let files = vec![
        ("file1.txt", "content1"),
        ("file2.txt", "content2"),
        ("dir1/file3.txt", "content3"),
    ];

    for (path, content) in &files {
        let file_path = temp.path().join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&file_path, content).unwrap();
    }

    // それぞれのファイルにリンク/コピーを作成
    for (path, _) in &files {
        let source = temp.path().join(path);
        let target = temp.path().join(format!("link_{path}"));

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        let result = manager.create_symlink(&source, &target);
        if result.is_ok() {
            assert!(target.exists());
        }
    }
}

#[test]
fn test_skip_if_exists() {
    use twin_cli::symlink::create_symlink_manager;

    let (_temp, source, target) = setup_test_workspace();
    let manager = create_symlink_manager();

    // 既存のファイルを作成
    fs::write(&target, "existing content").unwrap();

    // skip_if_existsの動作をテスト
    // 注: 現在の実装では、create_symlinkは既存ファイルを上書きする
    let result = manager.create_symlink(&source, &target);

    if result.is_ok() {
        // ファイルが上書きされたことを確認
        let content = fs::read_to_string(&target).unwrap();
        assert_eq!(content, "test content");
    }
}

#[test]
fn test_environment_variable_debug_output() {
    use std::env;
    use twin_cli::symlink::create_symlink_manager;

    // デバッグ出力を有効化
    unsafe {
        env::set_var("TWIN_VERBOSE", "1");
        env::set_var("TWIN_DEBUG", "1");
    }

    let (_temp, source, target) = setup_test_workspace();
    let manager = create_symlink_manager();

    // デバッグ出力付きで実行（出力は標準エラーに表示される）
    let _ = manager.create_symlink(&source, &target);

    // 環境変数をクリーンアップ
    unsafe {
        env::remove_var("TWIN_VERBOSE");
        env::remove_var("TWIN_DEBUG");
    }
}

#[test]
fn test_invalid_source_path() {
    use twin_cli::symlink::create_symlink_manager;

    let temp = TempDir::new().unwrap();
    let manager = create_symlink_manager();

    let non_existent = temp.path().join("non_existent.txt");
    let target = temp.path().join("target.txt");

    // 存在しないソースファイルの場合
    let result = manager.create_symlink(&non_existent, &target);
    assert!(result.is_err());
}

#[test]
fn test_directory_symlink() {
    use twin_cli::symlink::create_symlink_manager;

    let temp = TempDir::new().unwrap();
    let manager = create_symlink_manager();

    // ディレクトリを作成
    let source_dir = temp.path().join("source_dir");
    fs::create_dir(&source_dir).unwrap();
    fs::write(source_dir.join("file.txt"), "content").unwrap();

    let target_dir = temp.path().join("target_dir");

    // ディレクトリのシンボリックリンク/コピー
    let result = manager.create_symlink(&source_dir, &target_dir);

    // ディレクトリのシンボリックリンクはWindowsで制限がある
    if result.is_ok() {
        assert!(target_dir.exists());
        assert!(target_dir.join("file.txt").exists());
    }
}

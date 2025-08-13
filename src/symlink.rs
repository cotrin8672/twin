/// シンボリックリンク管理モジュール
///
/// このモジュールの役割：
/// - クロスプラットフォーム対応のシンボリックリンク作成
/// - Unix: ln -s コマンドのラッパー
/// - Windows: 開発者モード対応のmklinkラッパー（フォールバック機能付き）
/// - リンクの検証と削除
use crate::core::{SymlinkInfo, TwinError, TwinResult};
use std::fs;
use std::path::Path;
use std::process::Command;

/// リンク作成の戦略
#[derive(Debug, Clone, Copy)]
pub enum LinkStrategy {
    /// シンボリックリンク（推奨）
    Symlink,
    /// ジャンクション（Windowsディレクトリ用）
    Junction,
    /// ハードリンク（同一ドライブのファイル用）
    Hardlink,
    /// ファイルコピー（フォールバック）
    Copy,
}

/// プラットフォーム共通のトレイト
pub trait SymlinkManager {
    /// シンボリックリンクを作成
    fn create_symlink(&self, source: &Path, target: &Path) -> TwinResult<SymlinkInfo>;

    /// シンボリックリンクを削除
    fn remove_symlink(&self, path: &Path) -> TwinResult<()>;

    /// シンボリックリンクを検証
    fn validate_symlink(&self, path: &Path) -> TwinResult<bool>;

    /// 最適なリンク戦略を選択
    fn select_strategy(&self, source: &Path, target: &Path) -> LinkStrategy;

    /// 手動作成方法の説明を取得
    fn get_manual_instructions(&self, source: &Path, target: &Path) -> String;
}

/// プラットフォーム別の実装を選択
#[cfg(unix)]
pub type PlatformSymlinkManager = UnixSymlinkManager;

#[cfg(windows)]
pub type PlatformSymlinkManager = WindowsSymlinkManager;

/// Unix系OS用の実装
#[cfg(unix)]
pub struct UnixSymlinkManager;

#[cfg(unix)]
impl UnixSymlinkManager {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(unix)]
impl SymlinkManager for UnixSymlinkManager {
    fn create_symlink(&self, source: &Path, target: &Path) -> TwinResult<SymlinkInfo> {
        // 既存のリンクやファイルがある場合は削除
        if target.exists() || target.is_symlink() {
            fs::remove_file(target).ok();
        }

        // 親ディレクトリを作成
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }

        // シンボリックリンクを作成
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            match symlink(source, target) {
                Ok(_) => {
                    let mut info = SymlinkInfo::new(source.to_path_buf(), target.to_path_buf());
                    info.set_success();
                    Ok(info)
                }
                Err(e) => {
                    let mut info = SymlinkInfo::new(source.to_path_buf(), target.to_path_buf());
                    info.set_error(format!("Failed to create symlink: {}", e));
                    Err(TwinError::symlink(
                        format!("Failed to create symlink: {}", e),
                        Some(target.to_path_buf()),
                    ))
                }
            }
        }
    }

    fn remove_symlink(&self, path: &Path) -> TwinResult<()> {
        if path.is_symlink() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    fn validate_symlink(&self, path: &Path) -> TwinResult<bool> {
        if !path.exists() {
            return Ok(false);
        }

        // シンボリックリンクかどうか確認
        let metadata = fs::symlink_metadata(path)?;
        if !metadata.file_type().is_symlink() {
            return Ok(false);
        }

        // リンク先が存在するか確認
        match fs::metadata(path) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false), // 壊れたリンク
        }
    }

    fn select_strategy(&self, _source: &Path, _target: &Path) -> LinkStrategy {
        LinkStrategy::Symlink // Unixでは常にシンボリックリンク
    }

    fn get_manual_instructions(&self, source: &Path, target: &Path) -> String {
        format!(
            "To manually create the symlink, run:\n  ln -s \"{}\" \"{}\"",
            source.display(),
            target.display()
        )
    }
}

/// Windows用の実装
#[cfg(windows)]
pub struct WindowsSymlinkManager {
    /// 開発者モードが有効かどうか
    developer_mode: bool,
    /// 管理者権限で実行されているか
    is_elevated: bool,
}

#[cfg(windows)]
impl WindowsSymlinkManager {
    pub fn new() -> Self {
        Self {
            developer_mode: Self::check_developer_mode(),
            is_elevated: Self::check_elevation(),
        }
    }

    /// 開発者モードが有効か確認
    fn check_developer_mode() -> bool {
        // レジストリをチェック
        let output = Command::new("reg")
            .args(&[
                "query",
                "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\AppModelUnlock",
                "/v",
                "AllowDevelopmentWithoutDevLicense",
            ])
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.contains("0x1");
        }

        false
    }

    /// 管理者権限で実行されているか確認
    fn check_elevation() -> bool {
        // 管理者権限が必要な操作を試みる
        Command::new("net")
            .args(&["session"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// mklinkコマンドを実行
    fn execute_mklink(&self, source: &Path, target: &Path, is_dir: bool) -> TwinResult<()> {
        let mut cmd = Command::new("cmd");
        cmd.arg("/c");

        let mklink_args = if is_dir {
            format!(
                "mklink /D \"{}\" \"{}\"",
                target.display(),
                source.display()
            )
        } else {
            format!("mklink \"{}\" \"{}\"", target.display(), source.display())
        };

        cmd.arg(&mklink_args);

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TwinError::symlink(
                format!("mklink failed: {}", stderr),
                Some(target.to_path_buf()),
            ));
        }

        Ok(())
    }

    /// ジャンクションを作成（ディレクトリ用、管理者権限不要）
    fn create_junction(&self, source: &Path, target: &Path) -> TwinResult<()> {
        let output = Command::new("cmd")
            .args(&[
                "/c",
                &format!(
                    "mklink /J \"{}\" \"{}\"",
                    target.display(),
                    source.display()
                ),
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TwinError::symlink(
                format!("Junction creation failed: {}", stderr),
                Some(target.to_path_buf()),
            ));
        }

        Ok(())
    }

    /// ハードリンクを作成（ファイル用、管理者権限不要）
    fn create_hardlink(&self, source: &Path, target: &Path) -> TwinResult<()> {
        let output = Command::new("cmd")
            .args(&[
                "/c",
                &format!(
                    "mklink /H \"{}\" \"{}\"",
                    target.display(),
                    source.display()
                ),
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TwinError::symlink(
                format!("Hardlink creation failed: {}", stderr),
                Some(target.to_path_buf()),
            ));
        }

        Ok(())
    }

    /// ファイルをコピー
    fn copy_file(&self, source: &Path, target: &Path) -> TwinResult<()> {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::copy(source, target)?;
        Ok(())
    }
}

#[cfg(windows)]
impl SymlinkManager for WindowsSymlinkManager {
    fn create_symlink(&self, source: &Path, target: &Path) -> TwinResult<SymlinkInfo> {
        // 既存のファイルを削除
        if target.exists() {
            fs::remove_file(target).ok();
            fs::remove_dir(target).ok();
        }

        // 親ディレクトリを作成
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }

        let strategy = self.select_strategy(source, target);

        let result = match strategy {
            LinkStrategy::Symlink => self.execute_mklink(source, target, source.is_dir()),
            LinkStrategy::Copy => self.copy_file(source, target),
            _ => unreachable!(),
        };

        let mut info = SymlinkInfo::new(source.to_path_buf(), target.to_path_buf());

        match result {
            Ok(_) => {
                info.set_success();
                Ok(info)
            }
            Err(e) => {
                info.set_error(e.to_string());
                Err(e)
            }
        }
    }

    fn remove_symlink(&self, path: &Path) -> TwinResult<()> {
        if path.exists() {
            let metadata = fs::symlink_metadata(path)?;
            if metadata.is_dir() {
                fs::remove_dir(path)?;
            } else {
                fs::remove_file(path)?;
            }
        }
        Ok(())
    }

    fn validate_symlink(&self, path: &Path) -> TwinResult<bool> {
        if !path.exists() {
            return Ok(false);
        }

        // シンボリックリンクかジャンクションか確認
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            let metadata = fs::symlink_metadata(path)?;
            let attrs = metadata.file_attributes();

            // FILE_ATTRIBUTE_REPARSE_POINT をチェック
            const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
            if attrs & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                // リンク先が存在するか確認
                return match fs::metadata(path) {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                };
            }
        }

        Ok(false)
    }

    fn select_strategy(&self, _source: &Path, _target: &Path) -> LinkStrategy {
        // 開発者モードまたは管理者権限があればシンボリックリンク
        // なければ最初からコピー
        if self.developer_mode || self.is_elevated {
            LinkStrategy::Symlink
        } else {
            LinkStrategy::Copy
        }
    }

    fn get_manual_instructions(&self, source: &Path, target: &Path) -> String {
        if source.is_dir() {
            format!(
                "mklink /D \"{}\" \"{}\"",
                target.display(),
                source.display()
            )
        } else {
            format!("mklink \"{}\" \"{}\"", target.display(), source.display())
        }
    }
}

/// ドライブレターを取得（Windows用）
#[cfg(windows)]
fn get_drive_letter(path: &Path) -> Option<String> {
    path.to_str().and_then(|s| {
        if s.len() >= 2 && s.chars().nth(1) == Some(':') {
            Some(s[0..2].to_string())
        } else {
            None
        }
    })
}

/// ファクトリ関数
pub fn create_symlink_manager() -> Box<dyn SymlinkManager> {
    #[cfg(unix)]
    {
        Box::new(UnixSymlinkManager::new())
    }

    #[cfg(windows)]
    {
        Box::new(WindowsSymlinkManager::new())
    }
}

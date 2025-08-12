use std::path::PathBuf;
use thiserror::Error;

/// アプリケーション全体で使用するResult型
pub type TwinResult<T> = Result<T, TwinError>;

/// Twin アプリケーションのエラー型
/// thiserrorを使って、エラーメッセージの自動生成とFrom実装を行う
#[derive(Error, Debug)]
pub enum TwinError {
    /// Git操作に関するエラー
    #[error("Git error: {message}")]
    Git {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// シンボリックリンク操作に関するエラー
    #[error("Symlink error: {message}")]
    Symlink {
        message: String,
        path: Option<PathBuf>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// 設定ファイルに関するエラー
    #[error("Config error: {message}")]
    Config {
        message: String,
        path: Option<PathBuf>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// 環境管理に関するエラー
    #[error("Environment error: {message}")]
    Environment {
        message: String,
        agent_name: Option<String>,
    },
    
    /// ファイルシステム操作エラー
    #[error("IO error: {message}")]
    Io {
        message: String,
        path: Option<PathBuf>,
        #[source]
        source: Option<std::io::Error>,
    },
    
    /// 並行実行制御エラー（ロック取得失敗など）
    #[error("Lock error: {message}")]
    Lock {
        message: String,
        lock_path: Option<PathBuf>,
    },
    
    /// フック実行エラー
    #[error("Hook execution failed: {message}")]
    Hook {
        message: String,
        hook_type: String,
        exit_code: Option<i32>,
    },
    
    /// 既に存在するエラー
    #[error("{resource} already exists: {name}")]
    AlreadyExists {
        resource: String,
        name: String,
    },
    
    /// 見つからないエラー
    #[error("{resource} not found: {name}")]
    NotFound {
        resource: String,
        name: String,
    },
    
    /// 無効な引数エラー
    #[error("Invalid argument: {message}")]
    InvalidArgument {
        message: String,
    },
    
    /// その他のエラー
    #[error("{0}")]
    Other(String),
}

impl TwinError {
    /// Git関連のエラーを作成
    pub fn git(message: impl Into<String>) -> Self {
        Self::Git {
            message: message.into(),
            source: None,
        }
    }
    
    /// シンボリックリンク関連のエラーを作成
    pub fn symlink(message: impl Into<String>, path: Option<PathBuf>) -> Self {
        Self::Symlink {
            message: message.into(),
            path,
            source: None,
        }
    }
    
    /// 環境関連のエラーを作成
    pub fn environment(message: impl Into<String>, agent_name: Option<String>) -> Self {
        Self::Environment {
            message: message.into(),
            agent_name,
        }
    }
    
    /// 既に存在するエラーを作成
    pub fn already_exists(resource: impl Into<String>, name: impl Into<String>) -> Self {
        Self::AlreadyExists {
            resource: resource.into(),
            name: name.into(),
        }
    }
    
    /// 見つからないエラーを作成
    pub fn not_found(resource: impl Into<String>, name: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
            name: name.into(),
        }
    }
}

/// 標準のIOエラーからの変換
impl From<std::io::Error> for TwinError {
    fn from(err: std::io::Error) -> Self {
        Self::Io {
            message: err.to_string(),
            path: None,
            source: Some(err),
        }
    }
}

/// git2ライブラリのエラーからの変換
impl From<git2::Error> for TwinError {
    fn from(err: git2::Error) -> Self {
        Self::Git {
            message: err.to_string(),
            source: Some(Box::new(err)),
        }
    }
}

/// anyhowエラーからの変換
impl From<anyhow::Error> for TwinError {
    fn from(err: anyhow::Error) -> Self {
        Self::Other(err.to_string())
    }
}
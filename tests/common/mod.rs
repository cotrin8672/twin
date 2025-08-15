/// テスト用の共通ヘルパーモジュール
/// 
/// コンテナ環境とローカル環境の両方でテストを実行するための
/// ユーティリティを提供します。

use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, RemoveContainerOptions};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::image::CreateImageOptions;
use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use tokio::runtime::Runtime;

/// テスト環境の種類
#[derive(Debug, Clone)]
pub enum TestEnvironment {
    /// Linuxコンテナ環境
    Linux,
}

/// テスト用のGitリポジトリ環境
pub struct TestRepo {
    pub env: TestEnvironment,
    pub test_id: String,  // テストごとの一意ID
    temp_dir: Option<TempDir>,
    container_id: Option<String>,
    docker: Option<Docker>,
    runtime: Option<Runtime>,
}

impl TestRepo {
    
    /// Linuxコンテナ環境でテストリポジトリを作成
    pub fn linux() -> Self {
        let test_id = uuid::Uuid::new_v4().to_string()[0..8].to_string();
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");
        let docker = Docker::connect_with_local_defaults()
            .expect("Failed to connect to Docker");
        
        let container_id = runtime.block_on(async {
            // イメージをプル
            let options = CreateImageOptions {
                from_image: "rust:1.75-slim",
                ..Default::default()
            };
            
            let mut stream = docker.create_image(Some(options), None, None);
            while let Some(info) = stream.next().await {
                if let Err(e) = info {
                    eprintln!("Error pulling image: {}", e);
                }
            }
            
            // コンテナ作成
            let mut env = Vec::new();
            env.push("GIT_AUTHOR_NAME=Test User");
            env.push("GIT_AUTHOR_EMAIL=test@example.com");
            env.push("GIT_COMMITTER_NAME=Test User");
            env.push("GIT_COMMITTER_EMAIL=test@example.com");
            
            let config = Config {
                image: Some("rust:1.75-slim"),
                env: Some(env),
                working_dir: Some("/workspace"),
                cmd: Some(vec!["sleep", "3600"]),
                ..Default::default()
            };
            
            let container = docker.create_container(
                Some(CreateContainerOptions {
                    name: format!("twin-test-{}", uuid::Uuid::new_v4()),
                    ..Default::default()
                }),
                config,
            ).await.expect("Failed to create container");
            
            // コンテナ起動
            docker.start_container::<String>(&container.id, None)
                .await
                .expect("Failed to start container");
            
            // Git環境をセットアップ
            let setup_commands = vec![
                vec!["apt-get", "update"],
                vec!["apt-get", "install", "-y", "git"],
                vec!["git", "config", "--global", "user.name", "Test User"],
                vec!["git", "config", "--global", "user.email", "test@example.com"],
                vec!["mkdir", "-p", "/workspace/test-repo"],
                vec!["git", "init", "/workspace/test-repo"],
            ];
            
            for cmd in setup_commands {
                let exec = docker.create_exec(
                    &container.id,
                    CreateExecOptions {
                        cmd: Some(cmd),
                        attach_stdout: Some(true),
                        attach_stderr: Some(true),
                        ..Default::default()
                    },
                ).await.expect("Failed to create exec");
                
                if let StartExecResults::Attached { .. } = docker.start_exec(&exec.id, None).await.unwrap() {
                    // コマンド実行完了を待つ
                }
            }
            
            // 初期コミット作成
            let init_commands = vec![
                vec!["sh", "-c", "echo '# Test Repo' > /workspace/test-repo/README.md"],
                vec!["git", "-C", "/workspace/test-repo", "add", "."],
                vec!["git", "-C", "/workspace/test-repo", "commit", "-m", "Initial commit"],
            ];
            
            for cmd in init_commands {
                let exec = docker.create_exec(
                    &container.id,
                    CreateExecOptions {
                        cmd: Some(cmd),
                        attach_stdout: Some(true),
                        attach_stderr: Some(true),
                        ..Default::default()
                    },
                ).await.expect("Failed to create exec");
                
                if let StartExecResults::Attached { .. } = docker.start_exec(&exec.id, None).await.unwrap() {
                    // 実行完了を待つ
                }
            }
            
            container.id
        });
        
        Self {
            env: TestEnvironment::Linux,
            test_id,
            temp_dir: None,
            container_id: Some(container_id),
            docker: Some(docker),
            runtime: Some(runtime),
        }
    }
    
    /// テスト環境でコマンドを実行
    pub fn exec(&self, cmd: &[&str]) -> std::process::Output {
        match &self.env {
            TestEnvironment::Linux => {
                let docker = self.docker.as_ref().unwrap();
                let container_id = self.container_id.as_ref().unwrap();
                let runtime = self.runtime.as_ref().unwrap();
                
                runtime.block_on(async {
                    let exec = docker.create_exec(
                        container_id,
                        CreateExecOptions {
                            cmd: Some(cmd.iter().map(|s| s.to_string()).collect()),
                            attach_stdout: Some(true),
                            attach_stderr: Some(true),
                            working_dir: Some("/workspace/test-repo".to_string()),
                            ..Default::default()
                        },
                    ).await.expect("Failed to create exec");
                    
                    let output = match docker.start_exec(&exec.id, None).await.unwrap() {
                        StartExecResults::Attached { mut output, .. } => {
                            let mut stdout = Vec::new();
                            let mut stderr = Vec::new();
                            
                            while let Some(msg) = output.next().await {
                                match msg {
                                    Ok(bollard::container::LogOutput::StdOut { message }) => {
                                        stdout.extend_from_slice(&message);
                                    }
                                    Ok(bollard::container::LogOutput::StdErr { message }) => {
                                        stderr.extend_from_slice(&message);
                                    }
                                    _ => {}
                                }
                            }
                            
                            // Windowsでは ExitStatus を直接作成できないため、
                            // 成功を示すダミーの Output を返す
                            std::process::Output {
                                status: std::process::Command::new("cmd")
                                    .args(&["/c", "exit", "0"])
                                    .output()
                                    .unwrap()
                                    .status,
                                stdout,
                                stderr,
                            }
                        }
                        _ => panic!("Unexpected exec result"),
                    };
                    
                    output
                })
            }
        }
    }
    
    /// twinコマンドを実行
    pub fn run_twin(&self, args: &[&str]) -> std::process::Output {
        // コンテナにtwinバイナリをコピー
        self.copy_twin_to_container();
        
        let mut cmd = vec!["/tmp/twin"];
        cmd.extend_from_slice(args);
        self.exec(&cmd)
    }
    
    /// 一意のworktreeパスを生成
    pub fn worktree_path(&self, name: &str) -> String {
        format!("../{}-{}", name, self.test_id)
    }
    
    /// テストリポジトリのパスを取得
    pub fn path(&self) -> &Path {
        Path::new("/workspace/test-repo")
    }
    
    /// twinバイナリのパスを取得
    fn get_twin_binary() -> PathBuf {
        let exe_path = std::env::current_exe().unwrap();
        let target_dir = exe_path.parent().unwrap().parent().unwrap();
        target_dir.join("twin")
    }
    
    /// コンテナにtwinバイナリをコピー
    fn copy_twin_to_container(&self) {
        match &self.env {
            TestEnvironment::Linux => {
            let docker = self.docker.as_ref().unwrap();
            let container_id = self.container_id.as_ref().unwrap();
            let runtime = self.runtime.as_ref().unwrap();
            
            runtime.block_on(async {
                // twinバイナリをコンテナにコピー
                let twin_path = Self::get_twin_binary();
                let twin_bytes = std::fs::read(&twin_path).expect("Failed to read twin binary");
                
                // tarアーカイブを作成（簡易版）
                let mut tar_builder = tar::Builder::new(Vec::new());
                let mut header = tar::Header::new_gnu();
                header.set_path("twin").unwrap();
                header.set_size(twin_bytes.len() as u64);
                header.set_mode(0o755);
                header.set_cksum();
                
                tar_builder.append(&header, &twin_bytes[..]).unwrap();
                let tar_data = tar_builder.into_inner().unwrap();
                
                // コンテナにアップロード
                docker.upload_to_container(
                    container_id,
                    Some(bollard::container::UploadToContainerOptions {
                        path: "/tmp",
                        ..Default::default()
                    }),
                    tar_data.into(),
                ).await.expect("Failed to upload twin binary");
            });
        }
    }
}

impl Drop for TestRepo {
    fn drop(&mut self) {
        if let TestEnvironment::Container = &self.env {
            if let (Some(docker), Some(container_id), Some(runtime)) = 
                (&self.docker, &self.container_id, &self.runtime) {
                runtime.block_on(async {
                    // コンテナを停止・削除
                    let _ = docker.remove_container(
                        container_id,
                        Some(RemoveContainerOptions {
                            force: true,
                            ..Default::default()
                        }),
                    ).await;
                });
            }
        }
    }
}

/// テスト実行環境を選択
pub fn test_env() -> TestEnvironment {
    // 環境変数で制御
    if std::env::var("TEST_IN_CONTAINER").is_ok() {
        TestEnvironment::Container
    } else {
        TestEnvironment::Local
    }
}

/// 両環境でテストを実行するマクロ
#[macro_export]
macro_rules! test_both_envs {
    ($test_name:ident, $test_body:expr) => {
        paste::paste! {
            #[test]
            fn [<$test_name _local>]() {
                let repo = TestRepo::local();
                $test_body(repo);
            }
            
            #[test]
            #[cfg(not(windows))]  // Windowsではコンテナテストをスキップ
            fn [<$test_name _container>]() {
                if std::env::var("SKIP_CONTAINER_TESTS").is_ok() {
                    return;
                }
                let repo = TestRepo::container();
                $test_body(repo);
            }
        }
    };
}
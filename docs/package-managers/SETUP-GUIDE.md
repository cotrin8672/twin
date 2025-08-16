# パッケージマネージャー配布セットアップガイド

## 🎯 配布の優先順位

1. **Cargo (crates.io)** - 最も簡単、Rust標準
2. **Scoop** - Windows向け、個人バケットで即配布可能
3. **Homebrew** - macOS/Linux向け、Tapリポジトリで配布
4. **winget** - Windows公式、審査に時間がかかる
5. **その他** - AUR、snap、flatpakなど

## 📝 セットアップ手順

### 1. Cargo (全プラットフォーム) ⭐推奨

**メリット:**
- Rust開発者には最も馴染みがある
- クロスプラットフォーム対応
- バージョン管理が簡単

**手順:**
1. crates.ioでアカウント作成
2. `cargo login`でAPIトークン設定
3. `Cargo.toml`にメタデータ追加
4. `cargo publish`で公開

**ユーザーのインストール方法:**
```bash
cargo install twin-cli
```

### 2. Scoop (Windows)

**メリット:**
- Windowsユーザーに人気
- 個人バケットですぐ配布可能
- 自動更新機能あり

**手順:**
1. `scoop-twin`リポジトリ作成
2. `bucket/twin.json`を配置
3. リリース後にSHA256ハッシュ更新

**ユーザーのインストール方法:**
```powershell
scoop bucket add twin https://github.com/yourusername/scoop-twin
scoop install twin
```

### 3. Homebrew (macOS/Linux)

**メリット:**
- macOSの事実上の標準
- Linuxでも使用可能
- 依存関係管理が優秀

**手順:**
1. `homebrew-twin`リポジトリ作成（Tap）
2. `Formula/twin.rb`を配置
3. リリース後にSHA256ハッシュ更新

**ユーザーのインストール方法:**
```bash
brew tap yourusername/twin
brew install twin
```

### 4. winget (Windows)

**メリット:**
- Windows 10/11標準搭載
- Microsoft公式
- 企業環境での採用が増加中

**手順:**
1. microsoft/winget-pkgsをフォーク
2. マニフェストYAMLを作成
3. PRを送信（審査あり）
4. 承認後にマージ

**ユーザーのインストール方法:**
```powershell
winget install YourName.Twin
```

## 🔄 自動化のポイント

### GitHub Actions統合

リリース時に各パッケージマネージャーを自動更新：

```yaml
name: Update Package Managers
on:
  release:
    types: [published]

jobs:
  update-scoop:
    runs-on: ubuntu-latest
    steps:
      - name: Update Scoop manifest
        run: |
          # SHA256計算
          # JSONファイル更新
          # Git push

  update-homebrew:
    runs-on: ubuntu-latest
    steps:
      - name: Update Homebrew formula
        run: |
          # SHA256計算
          # Rubyファイル更新
          # Git push
```

## 📊 比較表

| マネージャー | プラットフォーム | 難易度 | 承認必要 | 自動更新 |
|------------|---------------|--------|---------|----------|
| Cargo | 全OS | ⭐ | なし | ✅ |
| Scoop | Windows | ⭐⭐ | なし | ✅ |
| Homebrew | macOS/Linux | ⭐⭐ | なし | ✅ |
| winget | Windows | ⭐⭐⭐ | あり | ✅ |
| AUR | Arch Linux | ⭐⭐⭐ | なし | ❌ |

## 🚀 リリースチェックリスト

1. [ ] バージョンタグ作成 (`v0.1.0`)
2. [ ] GitHub Releaseでバイナリ公開
3. [ ] SHA256ハッシュ計算
4. [ ] Cargo公開 (`cargo publish`)
5. [ ] Scoopマニフェスト更新
6. [ ] Homebrew Formula更新
7. [ ] winget PR送信（オプション）

## 💡 Tips

- **バージョニング**: セマンティックバージョニング（x.y.z）を使用
- **SHA256**: 各リリースバイナリのハッシュ値を必ず更新
- **テスト**: 各パッケージマネージャーでインストールテストを実施
- **ドキュメント**: READMEに各インストール方法を明記
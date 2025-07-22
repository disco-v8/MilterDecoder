# MilterDecoder

MIMEメール解析用の高性能Rust実装Milterプロトコルサーバー

## 概要

MilterDecoderは、Tokioを使用した非同期処理によってRustで書かれたMilter（メールフィルター）プロトコルサーバーです。メールサーバー（Postfixなど）からMilterプロトコル経由でメールデータを受信し、MIME構造を解析して、ヘッダー、本文内容、添付ファイル、エンコーディング詳細を含む詳細なメール情報を出力します。

## 機能

- **非同期処理**: Tokioによる高性能な並行クライアント処理
- **完全なMilterプロトコルサポート**: Postfix/Sendmail Milterプロトコルとの互換性
- **MIMEメール解析**: mail-parserを使用した完全なMIME構造解析
- **詳細出力**: From/To/Subject/Content-Type/エンコーディング/本文/添付ファイルの抽出
- **日本時間サポート**: chrono-tzによるJSTタイムスタンプログ
- **シグナル処理**: 設定再読込用SIGHUP、安全停止用SIGTERM
- **設定可能**: 外部設定ファイルによるサーバー設定
- **デバッグ機能**: NULバイト可視化、デバッグ用16進ダンプ出力

## インストール

### 前提条件

- Rust 1.70以降
- Tokioランタイム
- 対応メールサーバー（Postfix、Sendmailなど）

### ソースからのビルド

```bash
git clone https://github.com/disco-v8/MilterDecoder.git
cd MilterDecoder
cargo build --release
```

## 設定

プロジェクトルートに`MilterDecoder.conf`ファイルを作成してください：

```
Listen [::]:8898
Client_timeout 30
```

### 設定オプション

- `Listen`: サーバーバインドアドレスとポート（IPv4/IPv6サポート）
  - 形式: `IP:PORT` または `PORT`のみ（デュアルスタックがデフォルト）
  - 例: `192.168.1.100:4000` または `8898`
- `Client_timeout`: クライアント無通信タイムアウト秒数

## 使用方法

### サーバーの起動

```bash
./target/release/milter_decoder
```

### Postfixとの連携

`/etc/postfix/main.cf`に追加：

```
smtpd_milters = inet:localhost:8898
non_smtpd_milters = inet:localhost:8898
milter_default_action = accept
```

Postfixを再起動：

```bash
sudo systemctl restart postfix
```

### シグナル処理

- **SIGHUP**: 設定ファイル再読込
- **SIGTERM**: 安全停止

```bash
# 設定再読込
kill -HUP $(pidof milter_decoder)

# 安全停止
kill -TERM $(pidof milter_decoder)
```

## 出力形式

サーバーはJSTタイムスタンプ付きで詳細なメール解析を標準出力に出力します：

```
[2024/07/22 15:30:45] --- BODYEOB時のメール全体 ---
[2024/07/22 15:30:45] [mail-parser] from: sender@example.com
[2024/07/22 15:30:45] [mail-parser] to: recipient@example.com
[2024/07/22 15:30:45] [mail-parser] subject: Test Email
[2024/07/22 15:30:45] [mail-parser] content-type: "text/plain; charset=utf-8"
[2024/07/22 15:30:45] [mail-parser] テキストパート数: 1
[2024/07/22 15:30:45] [mail-parser] 非テキストパート数: 0
[2024/07/22 15:30:45] 本文(1): Hello, this is a test email.
```

### 添付ファイル付きマルチパートメール

```
[2024/07/22 15:31:20] このメールはマルチパートです
[2024/07/22 15:31:20] [mail-parser] テキストパート数: 1
[2024/07/22 15:31:20] [mail-parser] 非テキストパート数: 1
[2024/07/22 15:31:20] 本文(1): Email body content
[2024/07/22 15:31:20] 非テキストパート(1): content_type="application/pdf", encoding=Base64, filename=document.pdf, size=1024 bytes
```

## アーキテクチャ

### モジュール構造

- **main.rs**: サーバー起動、設定管理、シグナル処理
- **client.rs**: クライアント毎のMilterプロトコル処理
- **milter.rs**: Milterコマンドデコードと応答生成
- **milter_command.rs**: Milterプロトコルコマンド定義
- **parse.rs**: MIMEメール解析と出力整形
- **init.rs**: 設定ファイル管理
- **logging.rs**: JSTタイムスタンプログマクロ

### Milterプロトコルフロー

1. **OPTNEG**: プロトコルネゴシエーション
2. **CONNECT**: クライアント接続情報
3. **HELO/EHLO**: SMTP挨拶
4. **DATA**: マクロ情報
5. **HEADER**: メールヘッダー（複数）
6. **BODY**: メール本文内容（複数チャンク）
7. **BODYEOB**: 本文終了 - メール解析と出力をトリガー

## 依存関係

- [tokio](https://tokio.rs/): 非同期ランタイム
- [mail-parser](https://crates.io/crates/mail-parser): MIMEメール解析
- [chrono](https://crates.io/crates/chrono): 日時処理
- [chrono-tz](https://crates.io/crates/chrono-tz): タイムゾーンサポート
- [lazy_static](https://crates.io/crates/lazy_static): グローバル静的変数

## 開発

### 開発モードでの実行

```bash
cargo run
```

### サンプルメールでのテスト

設定されたPostfixインスタンス経由でメールを送信するか、telnetを使用してrawSMTPコマンドを送信することでサーバーをテストできます。

### デバッグ機能

- NULバイト可視化: `\0`バイトは`<NUL>`として表示
- 未知コマンドの16進ダンプ出力
- 詳細なプロトコルコマンドログ
- 説明的なメッセージによるエラー処理

## 貢献

1. リポジトリをフォーク
2. フィーチャーブランチを作成
3. 変更を加える
4. 該当する場合はテストを追加
5. プルリクエストを送信

## ライセンス

このプロジェクトはMITライセンスの下でライセンスされています - 詳細は[LICENSE](LICENSE)ファイルを参照してください。

**注意:**
- 本ソフトウェアはサードパーティ製クレートを利用しており、一部はApache-2.0ライセンスです。
- 特に [mail-parser](https://crates.io/crates/mail-parser) クレートはApache-2.0ライセンスです。再配布や改変の際は各クレートのライセンス条項もご確認ください。

## サポート

問題、質問、または貢献については、GitHubでissueを開いてください。

## 変更履歴

### v0.1.0
- 初回リリース
- 基本的なMilterプロトコル実装
- MIMEメール解析と出力
- 設定ファイルサポート
- シグナル処理
- JSTタイムスタンプログ

## 技術仕様

### サポートされるRustバージョン
- Rust 2021 edition
- 安定版、ベータ版、ナイトリー版でテスト済み

### パフォーマンス特性
- 非同期I/Oによる高いスループット
- 低メモリフットプリント
- 並行クライアント処理

### セキュリティ機能
- 入力バリデーション
- タイムアウト処理
- 安全なエラー処理
- メモリ安全性（Rustの利点）

## トラブルシューティング

### 一般的な問題

**ポートバインドエラー**
```
ポートバインド失敗: Address already in use
```
- 他のプロセスが同じポートを使用していないか確認
- `netstat -tulpn | grep 8898`でポート使用状況を確認

**Postfix接続エラー**
- Postfixの設定を確認
- ファイアウォール設定を確認
- MilterDecoderが正しいアドレスでリッスンしているか確認

**設定ファイルエラー**
- `MilterDecoder.conf`がプロジェクトルートに存在するか確認
- 設定ファイルの構文が正しいか確認

### ログレベル

現在の実装では全てのログが出力されます。将来のバージョンでログレベル制御を追加予定です。

## 将来の計画

- 設定可能なログレベル
- 統計情報とメトリクス
- Webダッシュボード
- プラグインシステム
- データベース統合
- 追加のメールサーバーサポート

## 貢献者

このプロジェクトへの貢献に興味がある場合は、[CONTRIBUTING.md](CONTRIBUTING.md)を参照してください。

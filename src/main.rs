// =========================
// main.rs
// MilterDecoder メインプログラム（Milterプロトコル受信サーバ）
//
// 【このファイルで使う主なクレート】
// - tokio: 非同期TCPサーバ・シグナル・ブロードキャスト（net::TcpListener, sync::broadcast, signal::unix）
// - std: スレッド安全な参照カウント・ロック（Arc, RwLock）
// - client: クライアント受信処理
// - init: 設定ファイル管理
// - logging: JSTタイムスタンプ付きログ出力
// - milter_command: Milterコマンド定義
//
// 【役割】
// - サーバー起動・設定管理・クライアント接続受付・シグナル処理
// =========================

mod client; // クライアント受信処理
mod init; // 設定ファイル管理
mod logging; // JSTタイムスタンプ付きログ出力
mod milter; // Milterコマンドごとのデコード・応答処理
mod milter_command; // Milterコマンド定義
mod parse; // メールパース・出力処理

use init::load_config;
use std::sync::{Arc, RwLock}; // スレッド安全な参照カウント・ロック
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind}; // Unix系: シグナル受信
#[cfg(windows)]
use tokio::signal::windows::{ctrl_c, ctrl_break}; // Windows: Ctrl+C/Break受信
use tokio::{net::TcpListener, sync::broadcast}; // 非同期TCPサーバ・ブロードキャスト // 設定ファイル読込

/// 非同期メイン関数（Tokioランタイム）
/// - サーバー起動・設定管理・クライアント接続受付・シグナル処理
#[tokio::main]
async fn main() {
    // 設定をスレッド安全に共有（Arc+RwLock）
    let config = Arc::new(RwLock::new(load_config()));
    // サーバー再起動・終了通知用ブロードキャストチャネル
    let (shutdown_tx, _) = broadcast::channel::<()>(100);

    #[cfg(unix)]
    {
        // SIGHUP/SIGTERM用にクローン
        let config = Arc::clone(&config); // 設定参照用
        let shutdown_tx_hup = shutdown_tx.clone(); // SIGHUP用
        let shutdown_tx_term = shutdown_tx.clone(); // SIGTERM用
                                                    // SIGHUP受信: 設定ファイル再読込
        tokio::spawn(async move {
            let mut hup = signal(SignalKind::hangup()).expect("SIGHUP登録失敗");
            while hup.recv().await.is_some() {
                printdaytimeln!("SIGHUP受信: 設定ファイル再読込");
                let new_config = load_config(); // 新設定読込
                *config.write().unwrap() = new_config; // 設定更新
                let _ = shutdown_tx_hup.send(()); // 全クライアントへ再起動通知
            }
        });
        // SIGTERM受信: サーバー安全終了
        tokio::spawn(async move {
            let mut term = signal(SignalKind::terminate()).expect("SIGTERM登録失敗");
            while term.recv().await.is_some() {
                printdaytimeln!("SIGTERM受信: サーバー安全終了");
                let _ = shutdown_tx_term.send(()); // 全クライアントへ終了通知
                std::process::exit(0); // プロセス終了
            }
        });
    }

    #[cfg(windows)]
    {
        // Windows用のシグナル処理
        let config = Arc::clone(&config); // 設定参照用
        let shutdown_tx_ctrl_c = shutdown_tx.clone(); // Ctrl+C用
        let shutdown_tx_ctrl_break = shutdown_tx.clone(); // Ctrl+Break用

        // Ctrl+C受信: 設定ファイル再読込（SIGHUP相当）
        tokio::spawn(async move {
            let mut ctrl_c_signal = ctrl_c().expect("Ctrl+C登録失敗");
            while ctrl_c_signal.recv().await.is_some() {
                printdaytimeln!("Ctrl+C受信: 設定ファイル再読込");
                let new_config = load_config(); // 新設定読込
                *config.write().unwrap() = new_config; // 設定更新
                let _ = shutdown_tx_ctrl_c.send(()); // 全クライアントへ再起動通知
            }
        });

        // Ctrl+Break受信: サーバー安全終了（SIGTERM相当）
        tokio::spawn(async move {
            let mut ctrl_break_signal = ctrl_break().expect("Ctrl+Break登録失敗");
            while ctrl_break_signal.recv().await.is_some() {
                printdaytimeln!("Ctrl+Break受信: サーバー安全終了");
                let _ = shutdown_tx_ctrl_break.send(()); // 全クライアントへ終了通知
                std::process::exit(0); // プロセス終了
            }
        });
    }

    loop {
        // サーバー再起動ループ
        let current_config = config.read().unwrap().clone(); // 現在の設定取得
        printdaytimeln!("設定読込: {}", current_config.address); // バインドアドレス表示
        let bind_result = TcpListener::bind(&current_config.address).await; // TCPバインド
        let listener = match bind_result {
            Ok(listener) => {
                printdaytimeln!("待受開始: {}", current_config.address); // バインド成功
                listener // リスナー返却
            }
            Err(e) => {
                eprintln!(
                    "ポートバインド失敗: {}\n他プロセスが {} 使用中?",
                    e, current_config.address
                ); // バインド失敗
                std::process::exit(1); // 異常終了
            }
        };
        let mut shutdown_rx = shutdown_tx.subscribe(); // 再起動・終了通知受信
        loop {
            // クライアント受信ループ
            tokio::select! {
                Ok((stream, addr)) = listener.accept() => {
                    printdaytimeln!("接続: {}", addr); // 新規接続
                    let shutdown_rx = shutdown_tx.subscribe(); // クライアント用レシーバ
                    tokio::spawn(client::handle_client(stream, shutdown_rx)); // クライアント処理開始
                }
                _ = shutdown_rx.recv() => {
                    printdaytimeln!("再起動のためリスナー再バインド"); // 再起動通知
                    break; // サーバーループ再開
                }
            }
        }
    }
}

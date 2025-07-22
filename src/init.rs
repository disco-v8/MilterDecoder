// =========================
// init.rs
// MilterDecoder 設定管理モジュール
//
// 【このファイルで使う主なクレート】
// - std: ファイル入出力（fs::read_to_string）、文字列処理（lines, trim, parse）、同期（sync::RwLock）
// - lazy_static: グローバル変数初期化（設定の静的共有）
//
// 【役割】
// - サーバー設定（Listenアドレス、クライアントタイムアウト等）の読み込み・保持
// - 設定ファイル(MilterDecoder.conf)からConfig構造体を生成
// - グローバル設定CONFIGとして全体で参照可能
// =========================

use std::sync::RwLock; // RwLock: スレッド安全な設定共有
use lazy_static::lazy_static; // lazy_static: グローバル変数初期化

/// サーバー設定情報構造体（Listen/Client_timeoutなど）
/// - address: サーバー待受アドレス（例: 0.0.0.0:8898）
/// - client_timeout: クライアント無通信タイムアウト秒
#[derive(Debug, Clone)]
pub struct Config {
    pub address: String, // サーバー待受アドレス（Listen）
    pub client_timeout: u64, // クライアントタイムアウト秒（Client_timeout）
}

/// 設定ファイル(MilterDecoder.conf)からConfigを生成
/// 設定ファイル(MilterDecoder.conf)からConfigを生成
///
/// # 説明
/// - Listen <アドレス/ポート>、Client_timeout <秒> をパースしてConfig構造体に格納
/// - Listen未指定時は[::]:8898、Client_timeout未指定時は30秒をデフォルト
pub fn load_config() -> Config {
    let text = std::fs::read_to_string("MilterDecoder.conf").expect("設定ファイル読み込み失敗"); // 設定ファイル全体を文字列で取得
    let mut address = None; // Listenアドレス初期値
    let mut client_timeout = 30u64; // タイムアウト初期値（秒）
    for line in text.lines() { // 設定ファイル各行をループ
        let line = line.trim(); // 前後空白除去
        // Listen設定（アドレス/ポート）
        if let Some(rest) = line.strip_prefix("Listen ") {
            let addr = rest.trim(); // アドレス部分取得
            if addr.contains(':') {
                address = Some(addr.to_string()); // IP:Port形式（例: 192.168.0.1:4000）
            } else {
                address = Some(format!("[::]:{}", addr)); // ポートのみ指定時はIPv4/IPv6デュアルスタック（例: 8898）
            }
        // Client_timeout設定（クライアント無通信タイムアウト秒）
        } else if let Some(rest) = line.strip_prefix("Client_timeout ") {
            if let Ok(val) = rest.trim().parse::<u64>() {
                client_timeout = val; // 数値変換成功時のみ反映
            }
        }
    }
    let address = address.unwrap_or_else(|| "[::]:8898".to_string()); // Listen未指定時はIPv4/IPv6デュアルスタック8898番ポート
    Config {
        address, // サーバー待受アドレス
        client_timeout, // クライアントタイムアウト秒
    }
}

// グローバル設定（再読込対応）
// - lazy_staticでCONFIGを初期化し、全体からスレッド安全に参照可能
lazy_static! {
    pub static ref CONFIG: RwLock<Config> = RwLock::new(load_config()); // グローバル設定
}

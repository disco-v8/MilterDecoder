// =========================
// client.rs
// MilterDecoder クライアント接続処理モジュール
//
// 【このファイルで使う主なクレート】
// - tokio: 非同期TCP通信・I/O・ブロードキャスト・タイムアウト等の非同期処理全般（net::TcpStream, io::AsyncReadExt, sync::broadcast）
// - std: 標準ライブラリ（アドレス、コレクション、時間、文字列操作など）
// - super::milter_command: Milterプロトコルのコマンド種別定義・判定用（MilterCommand enum, as_str等）
// - super::milter: Milterコマンドごとのペイロード分解・応答処理（decode_xxx群）
// - crate::parse: MIMEメールのパース・構造化・本文抽出・添付抽出（parse_mail）
// - crate::printdaytimeln!: JSTタイムスタンプ付きログ出力マクロ
//
// 【役割】
// - クライアント1接続ごとのMilterプロトコル非同期処理
// - ヘッダ受信 → コマンド判定 → ペイロード受信 → コマンド別処理 → 応答送信
// - BODYEOB時にメールパース・出力処理の呼び出し
// - タイムアウト・エラーハンドリング・シャットダウン通知処理
// =========================

use tokio::{
    io::AsyncReadExt, // 非同期I/Oトレイト（read等）
    net::TcpStream,   // 非同期TCPストリーム
    sync::broadcast,  // 非同期ブロードキャストチャンネル
};

use super::milter::{
    decode_body, decode_connect, decode_data_macros, decode_eoh_bodyeob, decode_header,
    decode_helo, decode_optneg,
};
use super::milter_command::MilterCommand; // Milterコマンド種別定義・判定 // 各Milterコマンドの分解・応答処理

use crate::parse::parse_mail; // メールパース・出力処理（BODYEOB時に呼び出し）

/// クライアント1接続ごとの非同期処理（Milterプロトコル）
/// 1. ヘッダ受信 → 2. コマンド判定 → 3. ペイロード受信 → 4. コマンド別処理 → 5. 応答送信
///    クライアント1接続ごとのMilterプロトコル非同期処理
pub async fn handle_client(
    mut stream: TcpStream,                    // クライアントTCPストリーム
    mut shutdown_rx: broadcast::Receiver<()>, // サーバーからのシャットダウン通知受信
) {
    // クライアントのIP:Portアドレス取得（接続元識別用）
    let peer_addr = match stream.peer_addr() {
        Ok(addr) => addr.to_string(),    // 正常時はアドレス文字列
        Err(_) => "unknown".to_string(), // 取得失敗時はunknown
    };

    // グローバル設定取得（タイムアウト秒など）
    let config = crate::init::CONFIG.read().unwrap().clone(); // 設定をロックしてクローン
    let timeout_duration = std::time::Duration::from_secs(config.client_timeout); // タイムアウト値をDuration化

    // BODYコマンド受信後はEOHをBODYEOB扱いにするフラグ
    let mut is_body_eob = false; // BODY受信後にEOHをBODYEOBとして扱う
                                 // DATAコマンドでヘッダブロック開始/終了を判定
    let mut is_header_block = false; // ヘッダブロック中かどうか
                                     // ヘッダ情報（複数値対応）
    let mut header_fields: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new(); // ヘッダ格納用
                                          // ボディ情報
    let mut body_field = String::new(); // ボディ格納用
                                        // メインループ: 切断・エラー・タイムアウト・シャットダウン通知以外は繰り返しコマンド受信・応答
    loop {
        // メインループ: 切断・エラー・タイムアウト・シャットダウン通知以外は繰り返しコマンド受信・応答
        // --- フェーズ1: 5バイトヘッダ受信（4バイト:サイズ + 1バイト:コマンド） ---
        let mut header = [0u8; 5]; // 5バイトのMilterヘッダバッファ
        let mut read_bytes = 0; // 受信済みバイト数カウンタ
                                // 5バイト受信するまでループ
        while read_bytes < 5 {
            // 5バイト受信するまでループ
            // タイムアウト・シャットダウン通知を同時監視しつつ受信
            match tokio::select! {
                res = tokio::time::timeout(timeout_duration, stream.read(&mut header[read_bytes..])) => res, // ヘッダ受信
                _ = shutdown_rx.recv() => { // サーバー再起動/終了通知（ブロードキャスト）
                    return; // サーバー都合で切断
                }
            } {
                Ok(Ok(0)) => {
                    // クライアント切断（0バイト受信）
                    crate::printdaytimeln!("切断(phase1): {}", peer_addr);
                    return; // ループ脱出
                }
                Ok(Ok(n)) => {
                    // 受信バイト数を加算
                    read_bytes += n; // 進捗更新
                }
                Ok(Err(e)) => {
                    // 受信エラー（I/O例外）
                    crate::printdaytimeln!("受信エラー: {}: {}", peer_addr, e);
                    return; // ループ脱出
                }
                Err(_) => {
                    // タイムアウト切断
                    crate::printdaytimeln!(
                        "タイムアウト: {} ({}秒間無通信)",
                        peer_addr,
                        config.client_timeout
                    );
                    return; // ループ脱出
                }
            }
        }

        // --- フェーズ2: コマンド判定（Milterコマンド種別） ---
        let size = u32::from_be_bytes([header[0], header[1], header[2], header[3]]); // 4バイト:コマンド+ペイロードサイズ
        let command = header[4]; // 1バイト:コマンド種別
        let milter_cmd = MilterCommand::from_u8(command); // コマンド種別をenum化
        match milter_cmd {
            // コマンド種別ごとに分岐
            Some(cmd) => {
                // EOHコマンド時はEOH/BODYEOB名で出力、それ以外は通常名
                if let MilterCommand::Eoh = cmd {
                    let eoh_str = MilterCommand::Eoh.as_str_eoh(is_body_eob);
                    crate::printdaytimeln!(
                        "コマンド受信: {} (0x{:02X}) size={} from {} [is_body_eob={}]",
                        eoh_str,
                        command,
                        size,
                        peer_addr,
                        is_body_eob
                    );
                } else {
                    crate::printdaytimeln!(
                        "コマンド受信: {} (0x{:02X}) size={} from {}",
                        cmd.as_str(),
                        command,
                        size,
                        peer_addr
                    );
                }
            }
            None => {
                // 未定義コマンドは切断
                crate::printdaytimeln!("不正コマンド: 0x{:02X} (addr: {})", command, peer_addr);
                return;
            }
        }

        // --- フェーズ3: ペイロード受信（4KB単位で分割） ---
        let mut remaining = size.saturating_sub(1) as usize; // 残り受信バイト数（コマンド1バイト分除外）
        let mut payload = Vec::with_capacity(remaining); // ペイロード格納バッファ
                                                         // ペイロード全体を受信するまでループ
        while remaining > 0 {
            // ペイロード全体を受信するまでループ
            // 受信するバイト数を決定（最大4KBずつ）
            let chunk_size = std::cmp::min(4096, remaining); // 受信単位（最大4KB）
            let mut chunk = vec![0u8; chunk_size]; // チャンクバッファを確保
            // タイムアウト付きでペイロード受信
            match tokio::select! {
                res = tokio::time::timeout(timeout_duration, stream.read(&mut chunk)) => res, // ペイロード受信
                _ = shutdown_rx.recv() => { // サーバー再起動/終了通知（ブロードキャスト）
                    return; // サーバー都合で切断
                }
            } {
                Ok(Ok(0)) => {
                    // クライアント切断（0バイト受信）
                    crate::printdaytimeln!("切断(phase3): {}", peer_addr);
                    return; // ループ脱出
                }
                Ok(Ok(n)) => {
                    // 受信データをペイロードへ格納
                    payload.extend_from_slice(&chunk[..n]); // バッファに追加
                                                            // 残りバイト数を減算
                    remaining -= n; // 進捗更新
                }
                Ok(Err(e)) => {
                    // 受信エラー（I/O例外）
                    crate::printdaytimeln!("受信エラー: {}: {}", peer_addr, e);
                    return; // ループ脱出
                }
                Err(_) => {
                    // タイムアウト切断
                    crate::printdaytimeln!(
                        "タイムアウト: {} ({}秒間無通信)",
                        peer_addr,
                        config.client_timeout
                    );
                    return; // ループ脱出
                }
            }
        }

        // ペイロード受信完了ログ（実際の受信サイズを出力）
        crate::printdaytimeln!(
            "ペイロード受信完了: {} bytes from {}",
            payload.len(),
            peer_addr
        ); // 受信サイズ出力

        // --- コマンド別処理: OPTNEG, EOH/BODYEOB, その他 ---
        if let Some(cmd) = milter_cmd {
            // コマンド種別ごとに処理分岐
            // PostfixのMilterプロトコルで送られてくる順番に分岐を並び替え
            // 主要なMilterコマンドごとに分岐し、各処理を実行
            if let MilterCommand::OptNeg = cmd {
                // OPTNEGコマンド解析処理（ネゴシエーション情報の分解・応答）
                decode_optneg(&mut stream, &payload).await; // ネゴシエーション応答
            } else if let MilterCommand::Connect = cmd {
                // CONNECTコマンド時は接続情報の分解＆応答（milter.rsに分離）
                decode_connect(&mut stream, &payload, &peer_addr).await; // 接続情報応答
            } else if let MilterCommand::HeLO = cmd {
                // HELOコマンド時はHELO情報の分解＆応答（milter.rsに分離）
                decode_helo(&mut stream, &payload, &peer_addr).await; // HELO応答
            } else if let MilterCommand::Data = cmd {
                // DATAコマンド時(のマクロ処理)（milter.rsに分離）
                decode_data_macros(&payload, &mut is_header_block); // マクロ情報処理
                                                                    // DATAコマンドではCONTINUE応答を送信しなくてもよい
            } else if let MilterCommand::Header = cmd {
                // SMFIC_HEADER(0x4C)コマンド時、ペイロードをヘッダ配列に格納＆出力（milter.rsに分離）
                decode_header(&payload, &mut header_fields); // ヘッダ格納
                                                             // HEADERコマンドではCONTINUE応答を送信しなくてもよい（Postfix互換）
            } else if let MilterCommand::Body = cmd {
                // BODYコマンドが来たら以降0x45はBODYEOB扱いにする
                is_body_eob = true; // BODY受信後はEOHをBODYEOB扱い
                is_header_block = false; // BODYコマンドでヘッダブロック終了
                                         // BODYペイロードをデコード・保存（ヘッダ配列・ボディも渡す）
                decode_body(&payload, &mut body_field); // ボディ格納
                                                        // BODYコマンドではCONTINUE応答を送信しなくてもよい
            } else if let MilterCommand::Eoh = cmd {
                // EOH/BODYEOBの判定・応答処理をmilter.rsに分離
                decode_eoh_bodyeob(&mut stream, is_body_eob, &peer_addr).await; // EOH/BODYEOB応答
                                                                                // BODYEOB(=is_body_eob==true)のときのみ、直前のヘッダ情報とボディ情報を出力
                if is_body_eob {
                    parse_mail(&header_fields, &body_field); // メールパース・出力
                                                             // 出力後はいろいろクリア
                    header_fields.clear(); // ヘッダ初期化
                    body_field.clear(); // ボディ初期化
                    is_body_eob = false; // BODYEOB→EOH遷移
                }
            } else {
                // その他のコマンドや拡張コマンド時
                // ペイロードデータを16進表記で出力（デバッグ用）
                if !payload.is_empty() {
                    let hexstr = payload
                        .iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(" "); // 16進ダンプ生成
                    crate::printdaytimeln!("ペイロード: {}", hexstr); // 16進ダンプ出力
                }
                // その他の正式なコマンドにはCONTINUE応答を送信しない
            }
        }
    } // メインループ終端
}

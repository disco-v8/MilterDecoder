
// =========================
// milter.rs
// MilterDecoder Milterコマンド処理モジュール
//
// 【このファイルで使う主なクレート】
// - tokio: 非同期TCP通信・I/O・応答送信などの非同期処理全般（net::TcpStream, io::AsyncWriteExt）
// - std: 標準ライブラリ（バイト操作、コレクション、エラー処理、フォーマット等）
// - crate::printdaytimeln!: JSTタイムスタンプ付きでログ出力する独自マクロ
// - crate::milter_command: Milterマクロ種別enum（MilterMacro）
//
// 【役割】
// - Milterコマンドごとのデコード・応答処理（OPTNEG, CONNECT, HELO, DATA, HEADER, BODY, EOH/BODYEOB）
// - ネゴシエーション情報の分解・応答送信
// - マクロペイロードの分解・出力
// - ヘッダ・ボディ情報の格納・加工
// =========================

use tokio::{
    net::TcpStream,      // 非同期TCPストリーム
    io::AsyncWriteExt,  // 非同期I/Oトレイト（write_all等）
};


/// SMFIC_OPTNEGペイロードを分解して出力し、OPTNEG応答を送信
/// OPTNEGコマンドのデコード・応答送信処理
/// - stream: クライアントTCPストリーム
/// - payload: 受信ペイロード
///   Milterプロトコルのネゴシエーション情報を分解し、内容を出力してOPTNEG応答を返す
///   OPTNEGコマンドのデコード・応答送信処理
///
/// # 引数
/// - `stream`: クライアントTCPストリーム
/// - `payload`: 受信ペイロード
///
/// # 説明
/// Milterプロトコルのネゴシエーション情報を分解し、内容を出力してOPTNEG応答を返す
pub async fn decode_optneg(stream: &mut TcpStream, payload: &[u8]) {
    // OPTNEGペイロードは: 4バイトプロトコルバージョン + 4バイト機能フラグ + 4バイトサポートフラグ
    // OPTNEGペイロードは12バイト以上必要（バージョン+アクション+フラグ）
    if payload.len() >= 12 {
        // 4バイトごとに各値を抽出
        // 4バイトごとに各値を抽出
        let protocol_ver = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]); // プロトコルバージョン
        let actions = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]); // アクションフラグ
        let protocol_flags = u32::from_be_bytes([payload[8], payload[9], payload[10], payload[11]]); // サポートフラグ
        // 受信したOPTNEG情報を詳細に出力
        crate::printdaytimeln!("SMFIC_OPTNEG: protocol_ver={} actions=0x{:08X} protocol_flags=0x{:08X}", protocol_ver, actions, protocol_flags);
        // 代表的なMilterアクションフラグを分解して出力
        // 代表的なMilterアクションフラグを分解して出力
        let action_flags = [
            (0x00000001, "ADD_HEADERS"), // ヘッダ追加
            (0x00000002, "CHANGE_BODY"), // 本文変更
            (0x00000004, "ADD_RECIPIENTS"), // 宛先追加
            (0x00000008, "DELETE_RECIPIENTS"), // 宛先削除
            (0x00000010, "QUARANTINE"), // 隔離
            (0x00000020, "REPLACE_HEADERS"), // ヘッダ置換
            (0x00000040, "CHANGE_REPLY"), // 応答変更
        ];
        // 各アクションフラグが立っていれば出力
        for (flag, name) in &action_flags {
            // 各アクションフラグが立っていれば出力
            if actions & flag != 0 {
                crate::printdaytimeln!("Milterアクション: {}", name); // アクションフラグごとに出力
            }
        }
        // プロトコルフラグも分解して出力
        // プロトコルフラグも分解して出力
        let proto_flags = [
            (0x00000001, "NO_CONNECT"), // CONNECT省略
            (0x00000002, "NO_HELO"), // HELO省略
            (0x00000004, "NO_ENVFROM"), // ENVFROM省略
            (0x00000008, "NO_ENVRCPT"), // ENVRCPT省略
            (0x00000010, "NO_BODY"), // BODY省略
            (0x00000020, "NO_HDRS"), // HDRS省略
            (0x00000040, "NO_UNKNOWN"), // UNKNOWN省略
            (0x00000080, "NO_DATA"), // DATA省略
        ];
        // 各プロトコルフラグが立っていれば出力
        for (flag, name) in &proto_flags {
            // 各プロトコルフラグが立っていれば出力
            if protocol_flags & flag != 0 {
                crate::printdaytimeln!("Milterプロトコル: {}", name); // サポートフラグごとに出力
            }
        }
        // OPTNEG応答バッファを生成（13バイト: コマンド1+ペイロード12）
        // OPTNEG応答バッファを生成（13バイト: コマンド1+ペイロード12）
        let mut resp = Vec::with_capacity(13);
        resp.extend_from_slice(&13u32.to_be_bytes()); // 応答サイズ（4バイト）
        resp.push(0x4f); // コマンド: SMFIR_OPTNEG（応答コマンド）
        resp.extend_from_slice(&protocol_ver.to_be_bytes()); // プロトコルバージョン（4バイト）
        // クライアントから受信したアクションフラグをそのまま応答にセット
        let resp_actions = actions;
        resp.extend_from_slice(&resp_actions.to_be_bytes()); // アクションフラグ（4バイト）
        // NO_BODY(0x10)とNO_HDRS(0x20)を立てないサポートフラグを生成（ヘッダ・ボディもMilterで渡される）
        let resp_protocol_flags = protocol_flags & !(0x10 | 0x20);
        resp.extend_from_slice(&resp_protocol_flags.to_be_bytes()); // サポートフラグ（4バイト）
        // クライアントにOPTNEG応答を送信
        // クライアントにOPTNEG応答を送信
        match stream.write_all(&resp).await {
            Ok(_) => crate::printdaytimeln!("SMFIR_OPTNEG応答送信完了: {:?}", resp), // 送信成功時
            Err(e) => crate::printdaytimeln!("SMFIR_OPTNEG応答送信エラー: {}", e), // 送信失敗時
        }
    } else {
        // ペイロード長不足時のエラー出力
        println!("SMFIC_OPTNEGペイロード長不足: {} bytes", payload.len());
    }
}

/// CONNECTコマンドのデコード・応答送信処理
/// 
/// # 引数
/// - `stream`: クライアントTCPストリーム
/// - `payload`: 受信ペイロード
/// - `peer_addr`: クライアントアドレス
/// 
/// # 説明
/// 受信した接続情報を出力し、CONTINUE応答(0x06)をクライアントに返す。
/// CONNECTコマンドのデコード・応答送信処理
///
/// # 引数
/// - `stream`: クライアントTCPストリーム
/// - `payload`: 受信ペイロード
/// - `peer_addr`: クライアントアドレス
///
/// # 説明
/// 受信した接続情報を出力し、CONTINUE応答(0x06)をクライアントに返す。
pub async fn decode_connect(stream: &mut tokio::net::TcpStream, payload: &[u8], peer_addr: &str) {
    // ペイロードをUTF-8文字列化し、接続情報として出力
    let connect_str = String::from_utf8_lossy(payload); // ペイロードをUTF-8文字列化
    crate::printdaytimeln!("接続情報: {}", connect_str); // 接続情報を出力
    // CONTINUE応答（0x06）を生成
    let resp_size: u32 = 1; // 応答サイズ（コマンドのみ）
    let resp_cmd: u8 = 0x06; // CONTINUEコマンド
    let mut resp = Vec::with_capacity(5); // 応答バッファ（5バイト: サイズ4+コマンド1）
    resp.extend_from_slice(&resp_size.to_be_bytes()); // サイズ（4バイト）
    resp.push(resp_cmd); // コマンド（1バイト）
    // クライアントにCONTINUE応答を送信
    // クライアントにCONTINUE応答を送信
    if let Err(e) = stream.write_all(&resp).await {
        crate::printdaytimeln!("応答送信エラー: {}: {}", peer_addr, e); // 送信失敗時
    } else {
        crate::printdaytimeln!("応答送信(connect): CONTINUE (0x06) to {}", peer_addr); // 送信成功時
    }
}

/// HELOコマンドのデコード・応答送信処理
/// 
/// # 引数
/// - `stream`: クライアントTCPストリーム
/// - `payload`: 受信ペイロード
/// - `peer_addr`: クライアントアドレス
/// 
/// # 説明
/// 受信したHELO情報を出力し、CONTINUE応答(0x06)をクライアントに返す。
/// HELOコマンドのデコード・応答送信処理
///
/// # 引数
/// - `stream`: クライアントTCPストリーム
/// - `payload`: 受信ペイロード
/// - `peer_addr`: クライアントアドレス
///
/// # 説明
/// 受信したHELO情報を出力し、CONTINUE応答(0x06)をクライアントに返す。
pub async fn decode_helo(stream: &mut tokio::net::TcpStream, payload: &[u8], peer_addr: &str) {
    // ペイロードをUTF-8文字列化し、HELO情報として出力
    let helo_str = String::from_utf8_lossy(payload); // ペイロードをUTF-8文字列化
    crate::printdaytimeln!("HELO: {}", helo_str); // HELO情報を出力
    // CONTINUE応答（0x06）を生成
    let resp_size: u32 = 1; // 応答サイズ（コマンドのみ）
    let resp_cmd: u8 = 0x06; // CONTINUEコマンド
    let mut resp = Vec::with_capacity(5); // 応答バッファ（5バイト: サイズ4+コマンド1）
    resp.extend_from_slice(&resp_size.to_be_bytes()); // サイズ（4バイト）
    resp.push(resp_cmd); // コマンド（1バイト）
    // クライアントにCONTINUE応答を送信
    // クライアントにCONTINUE応答を送信
    if let Err(e) = stream.write_all(&resp).await {
        crate::printdaytimeln!("応答送信エラー: {}: {}", peer_addr, e); // 送信失敗時
    } else {
        crate::printdaytimeln!("応答送信(helo): CONTINUE (0x06) to {}", peer_addr); // 送信成功時
    }
}

/// DATAコマンドのマクロペイロードを分解・出力する
///
/// # 引数
/// - `payload`: DATAコマンドのペイロード
///
/// # 説明
/// DATAコマンドのペイロードを0x00区切りで分割し、マクロ名・値を出力する。
/// 先頭バイトでマクロ種別を判定し、各マクロ名・値を詳細に出力。
///
/// 【この関数で使う主なクレート】
/// - crate::milter_command::MilterMacro: マクロ種別enum（Postfix/Sendmail互換）
/// - std: バイトスライス分割・文字列変換
pub fn decode_data_macros(payload: &[u8], is_header_block: &mut bool) {
    // DATAコマンドのマクロペイロードを0x00区切りで分割し、マクロ名・値を出力する。
    use crate::milter_command::MilterMacro;
    let parts: Vec<&[u8]> = payload.split(|b| *b == 0x00).filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        // マクロ無し
        return;
    }
    // 先頭バイトでマクロ種別（DATA/CONNECT/HELO/SOH等）を判定
    let phase_macro_val = parts[0].first().copied().unwrap_or(0);
    let phase_macro = MilterMacro::from_u8(phase_macro_val);
    let phase_macro_str = phase_macro.as_str().to_string();
    if phase_macro == MilterMacro::Soh {
        *is_header_block = true;
    }

    // 先頭マクロ名（parts[0]の2バイト目以降）と値（parts[1]）
    if parts[0].len() > 1 && parts.len() > 1 {
        let macro_name_bytes = &parts[0][1..];
        let macro_val_bytes = parts[1];
        let macro_name = if let Some(&b'{') = macro_name_bytes.first() {
            // {name}形式の拡張マクロ
            if let Some(close_idx) = macro_name_bytes[1..].iter().position(|&b| b == b'}') {
                let name = String::from_utf8_lossy(&macro_name_bytes[1..1+close_idx]);
                format!("{}({})", MilterMacro::Vender.as_str(), name)
            } else {
                format!("{}(Unknown)", MilterMacro::Vender.as_str())
            }
        } else {
            macro_name_bytes.first()
                .map(|&b| MilterMacro::from_u8(b).as_str().to_string())
                .unwrap_or(MilterMacro::Unknown(0).as_str().to_string())
        };
        let macro_val = String::from_utf8_lossy(macro_val_bytes).to_string();
        crate::printdaytimeln!("マクロ[{}][{}]={}", phase_macro_str, macro_name, macro_val);
    }
    // 以降は2つずつ: parts[n]=マクロ名, parts[n+1]=値
    let mut idx = 2;
    while idx + 1 < parts.len() {
        let macro_name_bytes = parts[idx];
        let macro_val_bytes = parts[idx + 1];
        let macro_name = if let Some(&b'{') = macro_name_bytes.first() {
            // {name}形式の拡張マクロ
            if let Some(close_idx) = macro_name_bytes[1..].iter().position(|&b| b == b'}') {
                let name = String::from_utf8_lossy(&macro_name_bytes[1..1+close_idx]);
                format!("{}({})", MilterMacro::Vender.as_str(), name)
            } else {
                format!("{}(Unknown)", MilterMacro::Vender.as_str())
            }
        } else {
            macro_name_bytes.first()
                .map(|&b| MilterMacro::from_u8(b).as_str().to_string())
                .unwrap_or(MilterMacro::Unknown(0).as_str().to_string())
        };
        let macro_val = String::from_utf8_lossy(macro_val_bytes).to_string();
        crate::printdaytimeln!("マクロ[{}][{}]={}", phase_macro_str, macro_name, macro_val);
        idx += 2;
    }
}

/// ヘッダペイロードをNUL区切りで分割し、header_fieldsに格納＆内容を可視化出力
pub fn decode_header(payload: &[u8], header_fields: &mut std::collections::HashMap<String, Vec<String>>) {
    let header_str = String::from_utf8_lossy(payload); // ペイロードをUTF-8文字列化
    let header_str_visible = header_str.replace('\0', "<NUL>"); // NULバイトを可視化（デバッグ用）
    crate::printdaytimeln!("ヘッダ内容: {}", header_str_visible); // ヘッダ内容をログ出力
    let mut parts = header_str.splitn(2, '\0'); // NUL区切りでヘッダ名と値に分割
    let key = parts.next().unwrap_or("").trim().trim_end_matches('\0').to_string(); // ヘッダ名（前後空白・末尾NUL除去）
    let val = parts.next().unwrap_or("").trim().trim_end_matches('\0').to_string(); // ヘッダ値（前後空白・末尾NUL除去）
    header_fields.entry(key).or_default().push(val); // ヘッダ名ごとに値を配列で追加
}

/// BODYコマンドのペイロード（ボディ本体）をデコードしてbody_fieldに格納
pub fn decode_body(payload: &[u8], body_field: &mut String) {
    // デコードや文字コード変換は行わず、BODYコマンドのたびにペイロードをbody_fieldへ追記する
    let s = String::from_utf8_lossy(payload); // ペイロードをUTF-8文字列化
    body_field.push_str(&s); // 既存body_fieldに追記
}

/// EOH(0x45)またはBODYEOB(0x45)コマンドの判定・応答送信処理
/// 
/// # 引数
/// - `stream`: クライアントTCPストリーム
/// - `is_body_eob`: trueならBODYEOBとしてACCEPT応答（0x61）、falseならEOHとしてCONTINUE応答（0x06）
/// - `peer_addr`: クライアントアドレス
/// 
/// # 説明
/// EOH/BODYEOBコマンドを判定し、適切な応答（ACCEPT/CONTINUE）をクライアントに送信する。
/// EOH(0x45)またはBODYEOB(0x45)コマンドの判定・応答送信処理
///
/// # 引数
/// - `stream`: クライアントTCPストリーム
/// - `is_body_eob`: trueならBODYEOBとしてACCEPT応答（0x61）、falseならEOHとしてCONTINUE応答（0x06）
/// - `peer_addr`: クライアントアドレス
///
/// # 説明
/// EOH/BODYEOBコマンドを判定し、適切な応答（ACCEPT/CONTINUE）をクライアントに送信する。
pub async fn decode_eoh_bodyeob(
    stream: &mut TcpStream,
    is_body_eob: bool,
    peer_addr: &str
) {
    // 応答コマンド・サイズを決定（BODYEOBなら0x61, EOHなら0x06）
    let (resp_cmd, resp_size) = if is_body_eob {
        (0x61u8, 1u32) // BODYEOB時はACCEPT応答
    } else {
        (0x06u8, 1u32) // EOH時はCONTINUE応答
    };
    let mut resp = Vec::with_capacity(5); // 応答バッファ（5バイト: サイズ4+コマンド1）
    resp.extend_from_slice(&resp_size.to_be_bytes()); // サイズ（4バイト）
    resp.push(resp_cmd); // コマンド（1バイト）
    // クライアントに応答を送信（非同期）
    if let Err(e) = stream.write_all(&resp).await {
        crate::printdaytimeln!("応答送信エラー: {}: {}", peer_addr, e); // 送信失敗時はエラーログ
    } else {
        crate::printdaytimeln!("応答送信(eoh/eob): (0x{:02X}) to {}", resp_cmd, peer_addr); // 送信成功時は詳細ログ
    }
}


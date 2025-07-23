// =========================
// parse.rs
// MilterDecoder メールパース・出力処理モジュール
//
// 【このファイルで使う主なクレート】
// - mail_parser: MIMEメールのパース・構造化・本文抽出・添付抽出（MessageParser, MimeHeaders）
// - std: 標準ライブラリ（コレクション・文字列操作・イテレータ等）
// - crate::printdaytimeln!: JSTタイムスタンプ付きログ出力マクロ
//
// 【役割】
// - BODYEOB時にヘッダ＋ボディを合体してメール全体をパース
// - From/To/Subject/Content-Type/エンコーディング/本文の構造化出力
// - パートごとのテキスト/非テキスト判定・出力
// - 添付ファイル名抽出・属性出力
// - NULバイト混入の可視化・除去
// =========================

// mail_parserクレートからメールパーサー本体とMimeHeadersトレイトをインポート
use mail_parser::{MessageParser, MimeHeaders}; // メールパース・MIMEヘッダアクセス用
                                               // ヘッダ情報格納用のHashMapをインポート
use std::collections::HashMap; // ヘッダ格納用

/// BODYEOB時にヘッダ＋ボディを合体してメール全体をパース・出力する関数
///
/// # 引数
/// - `header_fields`: Milterで受信したヘッダ情報（HashMap<String, Vec<String>>）
/// - `body_field`: Milterで受信したボディ情報（文字列）
///
/// # 説明
/// 1. ヘッダ＋ボディを合体してメール全体の生データを構築
/// 2. mail-parserでMIME構造をパース
/// 3. From/To/Subject/Content-Type/エンコーディング等の情報を出力
/// 4. パートごとのテキスト/非テキスト判定・出力
/// 5. 添付ファイル名抽出・属性出力
/// 6. NULバイト混入の可視化・除去
pub fn parse_mail(header_fields: &HashMap<String, Vec<String>>, body_field: &str) {
    // ヘッダ情報とボディ情報を合体し、RFC準拠のメール全体文字列を作成
    let mut mail_string = String::new(); // メール全体の文字列構築用バッファ
    
    // Milterで受信した各ヘッダを「ヘッダ名: 値」形式でメール文字列に追加
    for (k, vlist) in header_fields {
        // 同一ヘッダ名で複数値がある場合（Received等）も全て処理
        for v in vlist {
            mail_string.push_str(&format!("{}: {}\r\n", k, v)); // RFC準拠のCRLF改行
        }
    }
    
    mail_string.push_str("\r\n"); // ヘッダ部とボディ部の区切り空行（RFC必須）
    
    // ボディ部の改行コードをCRLFに統一（OS依存の改行コード差異を吸収）
    let body_crlf = body_field.replace("\r\n", "\n").replace('\n', "\r\n");
    mail_string.push_str(&body_crlf); // 正規化されたボディを追加
    
    // NULバイト（\0）を可視化文字に置換してデバッグ出力用に整形
    let mail_string_visible = mail_string.replace("\0", "<NUL>");
    crate::printdaytimeln!("--- BODYEOB時のメール全体 ---");
    crate::printdaytimeln!("{}", mail_string_visible); // 生メールデータの可視化出力

    // mail-parserでメール全体をパース
    let parser = MessageParser::default(); // パーサーインスタンス生成
    if let Some(msg) = parser.parse(mail_string.as_bytes()) {
        // パース成功時
        // Fromアドレスを文字列化（複数対応）
        let from = msg
            .from()
            .map(|addrs| {
                addrs
                    .iter()
                    .map(|addr| {
                        let name = addr.name().unwrap_or(""); // 差出人名
                        let address = addr.address().unwrap_or(""); // アドレス
                        if !name.is_empty() {
                            format!("{} <{}>", name, address) // 名前付きフォーマット
                        } else {
                            address.to_string() // アドレスのみ
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ") // 複数アドレスをカンマ区切り
            })
            .unwrap_or_else(|| "(なし)".to_string()); // From無し時のデフォルト
                                                      // Toアドレスを文字列化（複数対応）
        let to = msg
            .to()
            .map(|addrs| {
                addrs
                    .iter()
                    .map(|addr| {
                        let name = addr.name().unwrap_or(""); // 宛先名
                        let address = addr.address().unwrap_or(""); // 宛先アドレス
                        if !name.is_empty() {
                            format!("{} <{}>", name, address) // 名前付きフォーマット
                        } else {
                            address.to_string() // アドレスのみ
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ") // 複数アドレスをカンマ区切り
            })
            .unwrap_or_else(|| "(なし)".to_string()); // To無し時のデフォルト
                                                      // 件名取得
        let subject = msg.subject().unwrap_or("(なし)"); // 件名無し時のデフォルト
        crate::printdaytimeln!("[mail-parser] from: {}", from); // From出力
        crate::printdaytimeln!("[mail-parser] to: {}", to); // To出力
        crate::printdaytimeln!("[mail-parser] subject: {}", subject); // 件名出力
                                                                      // Content-Type（MIMEタイプ）があれば出力
        if let Some(ct) = msg
            .headers()
            .iter()
            .find(|h| h.name().eq_ignore_ascii_case("Content-Type")) // ヘッダ名がContent-Typeか判定
            .map(|h| h.value())
        // ヘッダ値を取得
        {
            crate::printdaytimeln!("[mail-parser] content-type: {:?}", ct); // MIMEタイプ出力
        }
        // Content-Transfer-Encoding（エンコーディング方式）があれば出力
        if let Some(enc) = msg
            .headers()
            .iter()
            .find(|h| h.name().eq_ignore_ascii_case("Content-Transfer-Encoding")) // ヘッダ名がContent-Transfer-Encodingか判定
            .map(|h| h.value())
        // ヘッダ値を取得
        {
            crate::printdaytimeln!("[mail-parser] encoding: {:?}", enc); // エンコーディング出力
        }
        // マルチパートかどうか判定（パート数で判定）
        if msg.parts.len() > 1 {
            crate::printdaytimeln!("[mail-parser] このメールはマルチパートです"); // 複数パート
        } else {
            crate::printdaytimeln!("[mail-parser] このメールはシングルパートです"); // 単一パート
        }
        // テキストパート数・非テキストパート数をカウント
        let mut text_count = 0; // 実際に本文出力対象となるテキストパート数
        let mut non_text_count = 0; // 添付ファイル等の非テキストパート数
        
        // テキストパートのインデックスリストを作成（本文出力で使用）
        let mut text_indices = Vec::new(); // 本文出力対象パートのインデックス記録用
        
        // 各パートを順番に調べてテキスト/非テキストを分類
        for (i, part) in msg.parts.iter().enumerate() {
            // パートがテキスト系かどうか判定（text/plain, text/html, multipart/alternative等）
            if part.is_text() {
                // multipart/*系パートは親パートなので本文出力対象から除外
                // （実際の本文はその子パートに格納されている）
                let is_multipart = part.content_type()
                    .is_some_and(|ct| ct.c_type.eq_ignore_ascii_case("multipart"));
                
                if !is_multipart {
                    // text/plain, text/html等の実体のあるテキストパート
                    text_count += 1; // テキストパート数をカウント
                    text_indices.push(i); // 本文出力用にインデックスを記録
                } else {
                    // multipart/alternative等の親パートは本文出力対象外
                    // （コメントのみで処理は何もしない）
                }
            } else {
                // application/octet-stream等の非テキストパート（添付ファイル等）
                non_text_count += 1; // 非テキストパート数をカウント
            }
        }
        crate::printdaytimeln!("[mail-parser] テキストパート数: {}", text_count); // テキストパート数出力
        crate::printdaytimeln!("[mail-parser] 非テキストパート数: {}", non_text_count); // 非テキストパート数出力

        // 本文出力処理：テキストパートごとに内容を出力
        for (idx, _) in text_indices.iter().enumerate() {
            // text/plain, text/html以外のテキストパートは本文出力をスキップ
            // （例：text/calendar等の特殊なテキストパート）
            let part = &msg.parts[text_indices[idx]]; // 対象パートを取得
            
            // Content-Typeのサブタイプ（plain, html等）を小文字で取得
            let subtype = part.content_type().and_then(|ct| ct.c_subtype.as_deref().map(|s| s.to_ascii_lowercase()));
            
            if let Some(subtype) = subtype {
                // サブタイプがplainまたはhtmlの場合のみ本文出力
                if subtype == "plain" || subtype == "html" {
                    // mail-parserの本文抽出メソッドでテキスト/HTML本文を取得
                    let text = msg.body_text(idx); // プレーンテキスト本文
                    let html = msg.body_html(idx); // HTML本文
                    
                    // テキスト本文があれば出力（ISO-2022-JP等からデコード済み）
                    if let Some(body) = text {
                        crate::printdaytimeln!("[mail-parser] TEXT本文({}): {}", idx + 1, body);
                    }
                    
                    // HTML本文があれば出力（quoted-printable等からデコード済み）
                    if let Some(html_body) = html {
                        crate::printdaytimeln!("[mail-parser] HTML本文({}): {}", idx + 1, html_body);
                    }
                }
                // text/plain, text/html以外は本文出力しない（スキップ）
            }
            // Content-Typeが不明な場合も本文出力しない（スキップ）
        }
        // 添付ファイル等の非テキストパート情報出力処理
        let mut non_text_idx = 0; // 非テキストパートの出力用連番（1から開始）
        
        // 全パートを再度走査して非テキストパートの詳細情報を出力
        for part in msg.parts.iter() {
            // テキストパート以外（添付ファイル、画像等）のみ処理
            if !part.is_text() {
                // Content-Type情報をヘッダから抽出（MIMEタイプ特定用）
                let ct = part
                    .headers
                    .iter()
                    .find(|h| h.name().eq_ignore_ascii_case("content-type")) // Content-Typeヘッダを検索
                    .map(|h| format!("{:?}", h.value())) // ヘッダ値を文字列として整形
                    .unwrap_or("(不明)".to_string()); // Content-Typeが無い場合のデフォルト値
                
                // Content-Transfer-Encodingの種別を文字列化（base64, quoted-printable等）
                let encoding_str = format!("{:?}", part.encoding); // エンコーディング情報
                
                // ファイル名情報を複数のヘッダから抽出
                let fname = part
                    .content_disposition() // Content-Disposition属性を取得
                    .and_then(|cd| {
                        // filename属性を検索（一般的な添付ファイル名指定）
                        cd.attributes()
                            .unwrap_or(&[])
                            .iter()
                            .find(|attr| attr.name.eq_ignore_ascii_case("filename"))
                            .map(|attr| attr.value.to_string())
                    })
                    .or_else(|| {
                        // Content-Typeのname属性も補助的にチェック（古い形式対応）
                        part.content_type()
                            .and_then(|ct| {
                                ct.attributes()
                                    .unwrap_or(&[])
                                    .iter()
                                    .find(|attr| attr.name.eq_ignore_ascii_case("name"))
                                    .map(|attr| attr.value.to_string())
                            })
                    })
                    .unwrap_or_else(|| "(ファイル名なし)".to_string()); // どちらにもファイル名が無い場合
                
                let size = part.body.len(); // パートの生データサイズ（バイト数）
                
                // 非テキストパートの詳細情報を1行で出力
                crate::printdaytimeln!(
                    "[mail-parser] 非テキストパート({}): content_type={}, encoding={}, filename={}, size={} bytes",
                    non_text_idx + 1, ct, encoding_str, fname, size
                );
                
                non_text_idx += 1; // 次の非テキストパート用に連番を進める
            }
        }
    } else {
        // パース失敗時（メール構造が不正等）
        crate::printdaytimeln!("[mail-parser] parse error"); // パース失敗ログ
    }
} // parse_mail関数終端

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
    // ヘッダ情報とボディ情報を合体し、メール全体の生データ文字列を作成
    let mut mail_string = String::new(); // メール全体の文字列バッファ
    for (k, vlist) in header_fields {
        // 各ヘッダ名と値リストをループ
        for v in vlist {
            // 同じヘッダ名の複数値も対応
            mail_string.push_str(&format!("{}: {}\r\n", k, v)); // ヘッダ1行を追加（RFC準拠のCRLF）
        }
    }
    mail_string.push_str("\r\n"); // ヘッダとボディの区切り（空行）
                                  // ボディ部の改行をCRLFに正規化（RFC準拠）
    let body_crlf = body_field.replace("\r\n", "\n").replace('\n', "\r\n"); // 改行コード統一
    mail_string.push_str(&body_crlf); // ボディを追加
                                      // NULバイトを可視化（<NUL>に置換）してデバッグ出力
    let mail_string_visible = mail_string.replace("\0", "<NUL>"); // NULバイト可視化
    crate::printdaytimeln!("--- BODYEOB時のメール全体 ---"); // 区切り線
    crate::printdaytimeln!("{}", mail_string_visible); // 生データ出力

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
            crate::printdaytimeln!("このメールはマルチパートです"); // 複数パート
        } else {
            crate::printdaytimeln!("このメールはシングルパートです"); // 単一パート
        }
        // テキストパート数・非テキストパート数をカウント
        let mut text_count = 0; // テキストパート数
        let mut non_text_count = 0; // 非テキストパート数
                                    // テキストパートのインデックスリストを作成
        let mut text_indices = Vec::new(); // テキストパートインデックス記録用
        for (i, part) in msg.parts.iter().enumerate() {
            // 各パートをインデックス付きでループ
            if part.is_text() {
                // テキストパートか判定
                text_count += 1; // テキストパート数カウント
                text_indices.push(i); // テキストパートのインデックス記録
            } else {
                non_text_count += 1; // 非テキストパート数カウント
            }
        }
        crate::printdaytimeln!("[mail-parser] テキストパート数: {}", text_count); // テキストパート数出力
        crate::printdaytimeln!("[mail-parser] 非テキストパート数: {}", non_text_count); // 非テキストパート数出力
                                                                                        // テキストパートごとに本文を出力
        for (idx, _) in text_indices.iter().enumerate() {
            // テキストパートインデックスをループ
            if let Some(body) = msg.body_text(idx) {
                // 本文デコード成功時
                crate::printdaytimeln!("本文({}): {}", idx + 1, body); // 本文出力（1ベース）
            } else {
                crate::printdaytimeln!("本文({}): (デコード不可)", idx + 1); // デコード不可時
            }
        }
        // 非テキストパートごとに属性を出力
        let mut non_text_idx = 0; // 非テキストパートの出力用インデックス（1ベース用）
        for part in msg.parts.iter() {
            // 各パートを再度ループ
            if !part.is_text() {
                // テキストパート以外のみ処理
                // Content-Type取得（ヘッダからMIMEタイプを抽出）
                let ct = part
                    .headers
                    .iter()
                    .find(|h| h.name().eq_ignore_ascii_case("content-type")) // ヘッダ名がcontent-typeか判定
                    .map(|h| format!("{:?}", h.value())) // ヘッダ値を文字列化
                    .unwrap_or("(不明)".to_string()); // なければ(不明)
                                                      // エンコーディング種別を文字列化（base64等）
                let encoding_str = format!("{:?}", part.encoding); // エンコーディング種別
                                                                   // ファイル名（Content-Disposition/Content-Typeヘッダからfilename/name属性を抽出）
                let fname = part
                    .content_disposition() // Content-Disposition属性を取得
                    .and_then(|cd| {
                        cd.attributes()
                            .unwrap_or(&[])
                            .iter() // 属性リストをイテレート
                            .find(|attr| attr.name.eq_ignore_ascii_case("filename")) // filename属性を探す
                            .map(|attr| attr.value.to_string()) // 見つかれば値を文字列化
                    })
                    .or_else(|| {
                        part.content_type() // Content-Type属性も補助的に参照
                            .and_then(|ct| {
                                ct.attributes()
                                    .unwrap_or(&[])
                                    .iter() // Content-Typeの属性リストをイテレート
                                    .find(|attr| attr.name.eq_ignore_ascii_case("name")) // name属性を探す
                                    .map(|attr| attr.value.to_string()) // 見つかれば値を文字列化
                            })
                    })
                    .unwrap_or_else(|| "(ファイル名なし)".to_string()); // どちらもなければ(ファイル名なし)
                let size = part.body.len(); // パートのバイトサイズ
                                            // 非テキストパートの属性を出力（MIMEタイプ・エンコーディング・ファイル名・サイズ）
                crate::printdaytimeln!(
					"非テキストパート({}): content_type={}, encoding={}, filename={}, size={} bytes",
					non_text_idx + 1, ct, encoding_str, fname, size
				);
                non_text_idx += 1; // インデックスを進める（次の非テキストパート用）
            }
        }
    } else {
        // パース失敗時（メール構造が不正等）
        crate::printdaytimeln!("[mail-parser] parse error"); // パース失敗ログ
    }
} // parse_mail関数終端

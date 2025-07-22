// =========================
// logging.rs
// MilterDecoder ログ出力マクロ定義
//
// 【このファイルで使う主なクレート】
// - chrono: 日時操作・整形（Local::now, format）
// - chrono-tz: タイムゾーン変換（Asia::Tokyo/JST指定）
//
// 【役割】
// - printdaytimeln!: JSTタイムスタンプ付きで標準出力にログを出すマクロ
// =========================

/// JSTタイムスタンプ付きで標準出力にログを出すマクロ
///
/// # 使い方
/// printdaytimeln!("メッセージ: {}", val);
///
/// # 説明
/// - chrono, chrono-tzでJST現在時刻を取得し、先頭に付与して出力
/// - 可変引数でformat!と同様に使える
#[macro_export] // クレート全体で利用可能
macro_rules! printdaytimeln {
    ($($arg:tt)*) => {{ // 可変引数（format!と同じ）
        let now = chrono::Local::now().with_timezone(&chrono_tz::Asia::Tokyo); // JST現在時刻取得
        let log_time = now.format("[%Y/%m/%d %H:%M:%S]"); // タイムスタンプ整形
        println!("{} {}", log_time, format!($($arg)*)); // タイムスタンプ付きログ出力
    }};
}

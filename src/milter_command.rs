// =========================
// milter_command.rs
// Milterプロトコルのマクロ種別・コマンド種別定義
//
// 【このファイルで使う主なクレート】
// - std: 標準ライブラリ（列挙型・マッチ分岐・デバッグ用）
//
// 【役割】
// - MilterMacro: マクロ種別（Postfix/Sendmail互換）
// - MilterCommand: コマンド種別（mfdef.h互換）
// - 各種変換・用途名取得メソッド
// =========================

/// Milterマクロ識別子（Postfix/Sendmail準拠）
/// 1バイト目でどのタイミングのマクロかを示す（例: Connect, Helo, Mail, ...）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MilterMacro {
    Connect = 0x43, // 接続時マクロ（C）
    Helo = 0x48,    // HELO/EHLO時マクロ（H）
    Mail = 0x4d,    // MAIL FROM時マクロ（M）
    Rcpt = 0x52,    // RCPT TO時マクロ（R）
    Eob = 0x45,     // 本文終了時マクロ（EOB, E）
    Data = 0x44,    // DATAコマンド時マクロ（D）
    Soh = 0x54,     // ヘッダスタート時マクロ（T）
    Header = 0x4c,  // ヘッダ受信時マクロ（L）
    Body = 0x42,    // 本文受信時マクロ（B）
    Quit = 0x51,    // セッション終了時マクロ（Q）
    // 1文字マクロ（Postfix/Sendmailでよく使われる伝統的なもの）
    Hostname = 0x6a,   // myhostname（j）
    QueueId = 0x69,    // queue_id（i）
    DaemonName = 0x6e, // daemon_name（n）
    ClientName = 0x73, // client_name（s）
    ClientAddr = 0x72, // client_addr（r）
    BodyType = 0x62,   // body_type（b）
    Version = 0x76,    // $version（v）
    Space = 0x5f,      // 空白（スペース, _）
    Vender = 0x7b,     // Vender拡張（複数文字マクロの開始, {）
    Unknown(u8),       // その他・拡張・未定義（バイト値で保持）
}

impl MilterMacro {
    /// 1バイト識別子からMilterMacroへ変換（Postfix/Sendmail互換）
    /// b'C'などのバイト値からenumへ変換
    pub fn from_u8(b: u8) -> Self {
        match b {
            b'C' => MilterMacro::Connect,
            b'H' => MilterMacro::Helo,
            b'M' => MilterMacro::Mail,
            b'R' => MilterMacro::Rcpt,
            b'E' => MilterMacro::Eob,
            b'D' => MilterMacro::Data,
            b'T' => MilterMacro::Soh,
            b'L' => MilterMacro::Header,
            b'B' => MilterMacro::Body,
            b'Q' => MilterMacro::Quit,
            b'j' => MilterMacro::Hostname,
            b'i' => MilterMacro::QueueId,
            b'n' => MilterMacro::DaemonName,
            b's' => MilterMacro::ClientName,
            b'r' => MilterMacro::ClientAddr,
            b'b' => MilterMacro::BodyType,
            b'v' => MilterMacro::Version,
            b'_' => MilterMacro::Space,
            b'{' => MilterMacro::Vender,
            other => MilterMacro::Unknown(other),
        }
    }
    /// MilterMacroを説明的な文字列（用途名）に変換
    /// 例: MACRO_Connect, MACRO_Helo ...
    pub fn as_str(&self) -> &'static str {
        match self {
            MilterMacro::Connect => "MACRO_Connect",
            MilterMacro::Helo => "MACRO_Helo",
            MilterMacro::Mail => "MACRO_Mail",
            MilterMacro::Rcpt => "MACRO_Rcpt",
            MilterMacro::Eob => "MACRO_Eob",
            MilterMacro::Data => "MACRO_Data",
            MilterMacro::Soh => "MACRO_Soh",
            MilterMacro::Header => "MACRO_Header",
            MilterMacro::Body => "MACRO_Body",
            MilterMacro::Quit => "MACRO_Quit",
            MilterMacro::Hostname => "MACRO_Hostname",
            MilterMacro::QueueId => "MACRO_QueueId",
            MilterMacro::DaemonName => "MACRO_DaemonName",
            MilterMacro::ClientName => "MACRO_ClientName",
            MilterMacro::ClientAddr => "MACRO_ClientAddr",
            MilterMacro::BodyType => "MACRO_BodyType",
            MilterMacro::Version => "MACRO_Version",
            MilterMacro::Space => "MACRO_Space",
            MilterMacro::Vender => "MACRO_Vender",
            MilterMacro::Unknown(_) => "MACRO_Unknown",
        }
    }
}

// =========================
// Milterコマンド定義（mfdef.hより抜粋）
// - Postfix/Sendmail互換のMilterプロトコルコマンドを列挙
// - 主要なコマンド種別（Abort, Accept, AddHeader, ...）を網羅
// - as_str/as_str_eohで用途名を取得可能
// =========================
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MilterCommand {
    Abort = 0x41,     // SMFIC_ABORT ('A'): セッション中断
    Accept = 0x61,    // SMFIC_ACCEPT ('a'): メール受理
    AddHeader = 0x68, // SMFIC_ADDHEADER ('h'): ヘッダ追加
    // AddRcpt = 0x61,    // SMFIC_ADDRCPT ('a', Acceptと重複): 宛先追加
    Body = 0x42,         // SMFIC_BODY ('B'): 本文受信
    Connect = 0x43,      // SMFIC_CONNECT ('C'): 接続情報
    Data = 0x44,         // SMFIC_DATA ('D'): DATAコマンド
    DeleteHeader = 0x64, // SMFIC_DELHEADER ('d'): ヘッダ削除
    DeleteRcpt = 0x72,   // SMFIC_DELRCPT ('r'): 宛先削除
    Eoh = 0x45,          // SMFIC_EOH ('E'): ヘッダ終了、もしくは SMFIC_BODYEOB ('E'): ボディ終了
    Eom = 0x4d,          // SMFIC_EOM ('M'): メール終了
    Header = 0x4c,       // SMFIC_HEADER ('L'): ヘッダ受信
    HeLO = 0x48,         // SMFIC_HELO ('H'): HELO受信
    // Macro = 0x4d,      // SMFIC_MACRO ('M', EOMと重複): マクロ情報
    OptNeg = 0x4f, // SMFIC_OPTNEG ('O'): オプション交渉　Postfixからの最初の接続でこれがくる
    Quit = 0x51,   // SMFIC_QUIT ('Q'): セッション終了
    Rcpt = 0x52,   // SMFIC_RCPT ('R'): 宛先受信
}

impl MilterCommand {
    /// EOH(0x45)を文脈に応じてSMFIC_EOHまたはSMFIC_BODYEOBとして返す補助関数
    /// is_body_eob=trueならBODYEOB、falseならEOHとして用途名を返す
    pub fn as_str_eoh(&self, is_body_eob: bool) -> &'static str {
        match (self, is_body_eob) {
            (MilterCommand::Eoh, true) => "SMFIC_BODYEOB",
            (MilterCommand::Eom, false) => "SMFIC_EOH",
            _ => self.as_str(),
        }
    }
    /// MilterCommandをコマンド名文字列（用途名）に変換
    /// 例: SMFIC_ABORT, SMFIC_ACCEPT ...
    pub fn as_str(&self) -> &'static str {
        match self {
            MilterCommand::Abort => "SMFIC_ABORT",
            MilterCommand::Accept => "SMFIC_ACCEPT",
            MilterCommand::AddHeader => "SMFIC_ADDHEADER",
            // MilterCommand::AddRcpt => "SMFIC_ADDRCPT", // Acceptと重複
            MilterCommand::Body => "SMFIC_BODY",
            MilterCommand::Connect => "SMFIC_CONNECT",
            MilterCommand::Data => "SMFIC_DATA",
            MilterCommand::DeleteHeader => "SMFIC_DELHEADER",
            MilterCommand::DeleteRcpt => "SMFIC_DELRCPT",
            MilterCommand::Eoh => "SMFIC_EOH",
            MilterCommand::Eom => "SMFIC_EOM",
            MilterCommand::Header => "SMFIC_HEADER",
            MilterCommand::HeLO => "SMFIC_HELO",
            // MilterCommand::Macro => "SMFIC_MACRO", // EOMと重複
            MilterCommand::OptNeg => "SMFIC_OPTNEG",
            MilterCommand::Quit => "SMFIC_QUIT",
            MilterCommand::Rcpt => "SMFIC_RCPT",
        }
    }
    /// 1バイト値からMilterCommandへ変換（Postfix/Sendmail互換）
    /// b'A'などのバイト値からenumへ変換
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            b'A' => Some(MilterCommand::Abort),
            b'a' => Some(MilterCommand::Accept), // AddRcptはAcceptと重複
            b'h' => Some(MilterCommand::AddHeader),
            b'B' => Some(MilterCommand::Body),
            b'C' => Some(MilterCommand::Connect),
            b'D' => Some(MilterCommand::Data),
            b'd' => Some(MilterCommand::DeleteHeader),
            b'r' => Some(MilterCommand::DeleteRcpt),
            b'E' => Some(MilterCommand::Eoh),
            b'M' => Some(MilterCommand::Eom), // MacroはEOMと重複
            b'L' => Some(MilterCommand::Header),
            b'H' => Some(MilterCommand::HeLO),
            b'O' => Some(MilterCommand::OptNeg),
            b'Q' => Some(MilterCommand::Quit),
            b'R' => Some(MilterCommand::Rcpt),
            _ => None,
        }
    }
}

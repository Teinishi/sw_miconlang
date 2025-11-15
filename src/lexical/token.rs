use logos::Logos;

#[derive(Logos, Debug, PartialEq, PartialOrd, Clone)]
#[logos(skip r"[ \t\r\n\f]+")] // ホワイトスペースを無視
#[logos(skip r"//[^\n]*")] // コメントを無視
pub enum Token {
    #[token("composite")]
    Composite,
    #[token("microcontroller")]
    Microcontroller,
    #[token("interface")]
    Interface,
    #[token("inputs")]
    Inputs,
    #[token("outputs")]
    Outputs,
    #[token("properties")]
    Properties,
    #[token("tooltips")]
    Tooltips,
    #[token("logic")]
    Logic,
    #[token("let")]
    Let,

    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(";")]
    Semicolon,
    #[token(".")]
    Dot,
    #[token("=")]
    Assignment,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Multiply,
    #[token("/")]
    Divide,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[token("null")]
    Null,
    #[token("false", |_| false)]
    #[token("true", |_| true)]
    Bool(bool),
    #[regex(r"[+-]?(?:0[xX][0-9A-Fa-f]+|0[bB][01]+|0[oO][0-7]+|[1-9]\d*|0)", |lex| parse_int(lex.slice()).unwrap())]
    Int(i64),
    #[regex(r"[+-]?(?:(?:\d+\.\d*|\.\d+)(?:[eE][+-]?\d+)?|\d+[eE][+-]?\d+)", |lex| lex.slice().parse::<f64>().unwrap())]
    Float(f64),
    #[regex(r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#, |lex| serde_json::from_str::<String>(lex.slice()).unwrap())]
    String(String),
}

fn parse_int(s: &str) -> Result<i64, std::num::ParseIntError> {
    let (neg, s) = s
        .strip_prefix('-')
        .map(|rest| (true, rest))
        .or_else(|| s.strip_prefix('+').map(|rest| (false, rest)))
        .unwrap_or((false, s));

    let (radix, digits) = if let Some(rest) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X"))
    {
        (16, rest)
    } else if let Some(rest) = s.strip_prefix("0b").or_else(|| s.strip_prefix("0B")) {
        (2, rest)
    } else if let Some(rest) = s.strip_prefix("0o").or_else(|| s.strip_prefix("0O")) {
        (8, rest)
    } else {
        (10, s)
    };

    let mut n = i64::from_str_radix(digits, radix)?;
    if neg {
        n = -n;
    }
    Ok(n)
}

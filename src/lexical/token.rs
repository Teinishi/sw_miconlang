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
    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?", |lex| lex.slice().parse::<f64>().unwrap())]
    Number(f64),
    #[regex(r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#, |lex| lex.slice().to_owned())]
    String(String),
}

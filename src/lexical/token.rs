use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone, Hash, Eq)]
#[logos(skip r"[ \t\r\n\f]+")] // ホワイトスペースを無視
#[logos(skip r"//[^\n]*")] // コメントを無視
pub enum Token {
    #[token("mcu")]
    Mcu,
    #[token("interface")]
    Interface,
    #[token("inputs")]
    Inputs,
    #[token("outputs")]
    Outputs,
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

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<u32>().ok())]
    Number(u32),
}

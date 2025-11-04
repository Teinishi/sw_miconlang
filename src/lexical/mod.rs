mod token;
pub use token::Token;

use logos::Logos as _;

pub fn lexer(code: &str) -> Result<Vec<(Token, logos::Span)>, Vec<logos::Span>> {
    // 字句解析
    let lex = Token::lexer(code).spanned().collect::<Vec<_>>();

    let mut tokens: Vec<(Token, logos::Span)> = Vec::with_capacity(lex.len());
    let mut lex_errors: Vec<logos::Span> = Vec::new();

    for (token, span) in lex {
        if let Ok(token) = token {
            tokens.push((token, span));
        } else {
            lex_errors.push(span);
        }
    }

    if lex_errors.is_empty() {
        Ok(tokens)
    } else {
        Err(lex_errors)
    }
}

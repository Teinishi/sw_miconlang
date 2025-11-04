mod syntax_tree;

use crate::lexical::Token;
pub use syntax_tree::File;

use chumsky::{Parser, error::Rich, extra, input::Input, prelude::just};

pub fn parser<'src, I>() -> impl Parser<'src, I, File, extra::Err<Rich<'src, I::Token, I::Span>>>
where
    I: Input<'src, Token = Token, Span = std::ops::Range<usize>>,
{
    // 構文解析
    just(Token::Mcu).map(|_| File)
}

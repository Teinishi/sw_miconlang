use crate::lexical::{Module, Token};

use chumsky::{Parser, error::Rich, extra, input::Input, prelude::just};

pub fn parser<'src, I>() -> impl Parser<'src, I, Module, extra::Err<Rich<'src, I::Token, I::Span>>>
where
    I: Input<'src, Token = Token, Span = std::ops::Range<usize>>,
{
    just(Token::Mcu).map(|_| Module)
}

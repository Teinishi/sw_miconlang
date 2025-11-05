mod syntax_tree;

use crate::lexical::Token;
pub use syntax_tree::*;
pub use syntax_tree::{Element, File};

use chumsky::{
    IterParser, Parser,
    error::Rich,
    extra,
    input::Input,
    prelude::{choice, just},
    select,
};

pub fn parser<'src, I>() -> impl Parser<'src, I, File, extra::Err<Rich<'src, I::Token, I::Span>>>
where
    I: Input<'src, Token = Token, Span = std::ops::Range<usize>>,
{
    // 構文解析
    let ident = select! { Token::Ident(v) => v }.labelled("identifier");
    let bool_literal = select! { Token::Bool(v) => LiteralValue::Bool(v) }.labelled("bool");
    let number_literal = select! { Token::Number(v) => LiteralValue::Number(v) }.labelled("number");
    let string_literal = select! { Token::String(v) => LiteralValue::String(v) }.labelled("string");

    // 式
    let expr = choice((bool_literal, number_literal, string_literal)).map(|value| Expr { value });

    // field = expr
    let field_assignment = ident.then_ignore(just(Token::Assignment)).then(expr);

    // microcontroller Name {...}
    let microcontroller = just(Token::Microcontroller)
        .ignore_then(ident)
        .then_ignore(just(Token::LBrace))
        .then(field_assignment.repeated().collect::<Vec<_>>())
        //.then(field_assignment.repeated())
        .then_ignore(just(Token::RBrace))
        .map(|(name, fields)| Element::Microcontroller { name, fields })
        .labelled("microcontroller");

    let element = microcontroller.labelled("element");

    element.repeated().collect().map(|el| File { elements: el })
}

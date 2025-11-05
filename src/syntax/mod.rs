mod syntax_tree;

use crate::lexical::Token;
pub use syntax_tree::*;

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
    let assignment = ident
        .then_ignore(just(Token::Assignment))
        .then(expr)
        .map(|(ident, value)| Assignment { ident, value });

    // name: type { field = expr }
    let interface_node = ident
        .then_ignore(just(Token::Colon))
        .then(ident)
        .then(
            just(Token::LBrace)
                .ignore_then(assignment.clone().repeated().collect::<Vec<_>>())
                .then_ignore(just(Token::RBrace))
                .or_not(),
        )
        .map(|((name, type_name), fields)| MicrocontrollerInterfaceNode {
            name,
            type_name,
            fields,
        });

    // inputs {...}
    let inputs = just(Token::Inputs)
        .ignore_then(just(Token::LBrace))
        .ignore_then(interface_node.clone().repeated().collect::<Vec<_>>())
        .then_ignore(just(Token::RBrace))
        .map(MicrocontrollerInterface::Inputs)
        .labelled("inputs");

    // outputs {...}
    let outputs = just(Token::Outputs)
        .ignore_then(just(Token::LBrace))
        .ignore_then(interface_node.repeated().collect::<Vec<_>>())
        .then_ignore(just(Token::RBrace))
        .map(MicrocontrollerInterface::Outputs)
        .labelled("outputs");

    // interface {...} など
    let microcontroller_interface = just(Token::Interface)
        .ignore_then(just(Token::LBrace))
        .ignore_then(choice((inputs, outputs)).repeated().collect::<Vec<_>>())
        .then_ignore(just(Token::RBrace))
        .map(MicrocontrollerElement::Interface)
        .labelled("interface");

    // microcontroller Name {...}
    let microcontroller = just(Token::Microcontroller)
        .ignore_then(ident)
        .then_ignore(just(Token::LBrace))
        .then(
            choice((
                assignment.map(MicrocontrollerElement::Field),
                microcontroller_interface,
            ))
            .repeated()
            .collect::<Vec<_>>(),
        )
        .then_ignore(just(Token::RBrace))
        .map(|(name, elements)| Element::Microcontroller { name, elements })
        .labelled("microcontroller");

    let element = microcontroller.labelled("element");

    element.repeated().collect().map(|el| File { elements: el })
}

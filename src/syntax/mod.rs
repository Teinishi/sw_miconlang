mod syntax_tree;

use crate::lexical::Token;
pub use syntax_tree::*;

use chumsky::{
    IterParser, Parser,
    error::Rich,
    extra,
    input::Input,
    prelude::{choice, just, recursive},
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
    let expr = recursive(|p| {
        let parenthesized = p
            .clone()
            .delimited_by(just(Token::LParen), just(Token::RParen));

        let atom = choice((bool_literal, number_literal, string_literal))
            .map(Expr::LiteralValue)
            .or(just(Token::Inputs).map(|_| Expr::Ident("inputs".to_owned())))
            .or(just(Token::Outputs).map(|_| Expr::Ident("outputs".to_owned())))
            .or(ident.map(Expr::Ident))
            .or(parenthesized);

        let field_access = atom.clone().foldl(
            just(Token::Dot).ignore_then(ident).repeated(),
            |lhs, rhs| Expr::FieldAccess {
                target: Box::new(lhs),
                field: rhs,
            },
        );

        let unary = just(Token::Minus)
            .repeated()
            .foldr(field_access, |_op, rhs| {
                Expr::UnaryOp(UnaryOp::Neg(Box::new(rhs)))
            });

        let binary_1 = unary.clone().foldl(
            choice((just(Token::Multiply), just(Token::Divide)))
                .then(unary)
                .repeated(),
            |lhs, (op, rhs)| match op {
                Token::Multiply => Expr::BinaryOp(BinaryOp::Mul(Box::new(lhs), Box::new(rhs))),
                Token::Divide => Expr::BinaryOp(BinaryOp::Div(Box::new(lhs), Box::new(rhs))),
                _ => unreachable!(),
            },
        );

        binary_1.clone().foldl(
            choice((just(Token::Plus), just(Token::Minus)))
                .then(binary_1)
                .repeated(),
            |lhs, (op, rhs)| match op {
                Token::Plus => Expr::BinaryOp(BinaryOp::Add(Box::new(lhs), Box::new(rhs))),
                Token::Minus => Expr::BinaryOp(BinaryOp::Sub(Box::new(lhs), Box::new(rhs))),
                _ => unreachable!(),
            },
        )
    })
    .labelled("expression");

    // field = expr
    let assignment = expr
        .clone()
        .then_ignore(just(Token::Assignment))
        .then(expr)
        .map(|(target, value)| Assignment { target, value });

    // 文
    let statement = assignment
        .clone()
        .map(Statement::Assignment)
        .labelled("statement");

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

    // interface {...}
    let microcontroller_interface = just(Token::Interface)
        .ignore_then(just(Token::LBrace))
        .ignore_then(choice((inputs, outputs)).repeated().collect::<Vec<_>>())
        .then_ignore(just(Token::RBrace))
        .map(MicrocontrollerElement::Interface)
        .labelled("interface");

    // logic {...}
    let logic = just(Token::Logic)
        .ignore_then(just(Token::LBrace))
        .ignore_then(statement.repeated().collect::<Vec<_>>())
        .then_ignore(just(Token::RBrace))
        .map(MicrocontrollerElement::Logic)
        .labelled("logic");

    // microcontroller Name {...}
    let microcontroller = just(Token::Microcontroller)
        .ignore_then(ident)
        .then_ignore(just(Token::LBrace))
        .then(
            choice((
                assignment.map(MicrocontrollerElement::Field),
                microcontroller_interface,
                logic,
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

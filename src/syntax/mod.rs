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

pub fn parser<'src, I>()
-> impl Parser<'src, I, Spanned<File>, extra::Err<Rich<'src, I::Token, I::Span>>>
where
    I: Input<'src, Token = Token, Span = std::ops::Range<usize>>,
{
    // 構文解析
    let ident = select! { Token::Ident(v) => v }.labelled("identifier");
    let ident_expr = ident.map_with(|name, e| Spanned {
        inner: Expr::Ident(name),
        span: e.span(),
    });
    let literal = select! {
        Token::Bool(v) => Expr::BoolLiteral(v),
        Token::Int(v) => Expr::IntLiteral(v),
        Token::Float(v) => Expr::FloatLiteral(v),
        Token::String(v) => Expr::StringLiteral(v),
    }
    .map_with(|v, e| Spanned {
        inner: v,
        span: e.span(),
    })
    .labelled("literal");

    // 式
    let expr = recursive(|p| {
        let parenthesized = p
            .clone()
            .delimited_by(just(Token::LParen), just(Token::RParen));

        let atom = choice((
            literal,
            just(Token::Inputs).map_with(|_, e| Spanned {
                inner: Expr::Inputs,
                span: e.span(),
            }),
            just(Token::Outputs).map_with(|_, e| Spanned {
                inner: Expr::Outputs,
                span: e.span(),
            }),
            ident_expr,
            parenthesized,
        ));

        let field_access = atom.clone().foldl_with(
            just(Token::Dot).ignore_then(ident).repeated(),
            |lhs, rhs, e| Spanned {
                inner: Expr::FieldAccess(Box::new(lhs), rhs),
                span: e.span(),
            },
        );

        // 単項演算 (-)
        let unary = just(Token::Minus)
            .repeated()
            .foldr_with(field_access, |_op, rhs, e| Spanned {
                inner: Expr::UnaryOp(UnaryOp::Neg(Box::new(rhs))),
                span: e.span(),
            });

        // 二項演算 (* /)
        let binary_1 = unary.clone().foldl_with(
            choice((just(Token::Multiply), just(Token::Divide)))
                .then(unary)
                .repeated(),
            |lhs, (op, rhs), e| Spanned {
                inner: match op {
                    Token::Multiply => Expr::BinaryOp(BinaryOp::Mul(Box::new(lhs), Box::new(rhs))),
                    Token::Divide => Expr::BinaryOp(BinaryOp::Div(Box::new(lhs), Box::new(rhs))),
                    _ => unreachable!(),
                },
                span: e.span(),
            },
        );

        // 二項演算 (+ -)
        let binary_2 = binary_1.clone().foldl_with(
            choice((just(Token::Plus), just(Token::Minus)))
                .then(binary_1)
                .repeated(),
            |lhs, (op, rhs), e| Spanned {
                inner: match op {
                    Token::Plus => Expr::BinaryOp(BinaryOp::Add(Box::new(lhs), Box::new(rhs))),
                    Token::Minus => Expr::BinaryOp(BinaryOp::Sub(Box::new(lhs), Box::new(rhs))),
                    _ => unreachable!(),
                },
                span: e.span(),
            },
        );

        // タプル
        let tuple = binary_2
            .clone()
            .separated_by(just(Token::Comma))
            .collect::<Vec<_>>()
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .map_with(|v, e| Spanned {
                inner: Expr::Tuple(v),
                span: e.span(),
            })
            .labelled("tuple");

        tuple.or(binary_2)
    })
    .labelled("expression");

    // field = expr
    let assignment = expr
        .clone()
        .then_ignore(just(Token::Assignment))
        .then(expr)
        .map_with(|(target, value), e| Spanned {
            inner: Assignment { target, value },
            span: e.span(),
        });

    // 文
    let statement = assignment
        .clone()
        .map_with(|assignment, e| Spanned {
            inner: Statement::Assignment(assignment),
            span: e.span(),
        })
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
        .map_with(|((name, type_name), fields), e| Spanned {
            inner: MicrocontrollerInterfaceNode {
                name,
                type_name,
                fields,
            },
            span: e.span(),
        });

    // inputs {...}
    let inputs = just(Token::Inputs)
        .ignore_then(just(Token::LBrace))
        .ignore_then(interface_node.clone().repeated().collect::<Vec<_>>())
        .then_ignore(just(Token::RBrace))
        .map_with(|nodes, e| Spanned {
            inner: MicrocontrollerInterface::Inputs(nodes),
            span: e.span(),
        })
        .labelled("inputs");

    // outputs {...}
    let outputs = just(Token::Outputs)
        .ignore_then(just(Token::LBrace))
        .ignore_then(interface_node.repeated().collect::<Vec<_>>())
        .then_ignore(just(Token::RBrace))
        .map_with(|nodes, e| Spanned {
            inner: MicrocontrollerInterface::Outputs(nodes),
            span: e.span(),
        })
        .labelled("outputs");

    // interface {...}
    let microcontroller_interface = just(Token::Interface)
        .ignore_then(just(Token::LBrace))
        .ignore_then(choice((inputs, outputs)).repeated().collect::<Vec<_>>())
        .then_ignore(just(Token::RBrace))
        .map_with(|interface, e| Spanned {
            inner: MicrocontrollerElement::Interface(interface),
            span: e.span(),
        })
        .labelled("interface");

    // logic {...}
    let logic = just(Token::Logic)
        .ignore_then(just(Token::LBrace))
        .ignore_then(statement.repeated().collect::<Vec<_>>())
        .then_ignore(just(Token::RBrace))
        .map_with(|statements, e| Spanned {
            inner: MicrocontrollerElement::Logic(statements),
            span: e.span(),
        })
        .labelled("logic");

    // microcontroller Name {...}
    let microcontroller = just(Token::Microcontroller)
        .ignore_then(ident)
        .then_ignore(just(Token::LBrace))
        .then(
            choice((
                assignment.map_with(|assignment, e| Spanned {
                    inner: MicrocontrollerElement::Field(assignment),
                    span: e.span(),
                }),
                microcontroller_interface,
                logic,
            ))
            .repeated()
            .collect::<Vec<_>>(),
        )
        .then_ignore(just(Token::RBrace))
        .map_with(|(name, elements), e| Spanned {
            inner: Element::Microcontroller { name, elements },
            span: e.span(),
        })
        .labelled("microcontroller");

    let element = microcontroller.labelled("element");

    element.repeated().collect().map_with(|el, e| Spanned {
        inner: File { elements: el },
        span: e.span(),
    })
}

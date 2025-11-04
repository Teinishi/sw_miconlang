use crate::{
    lexical::{Module, Token},
    syntax::parser,
};

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{Parser, error::RichPattern, input::IterInput};
use logos::Logos as _;

pub fn parse(code: &str, filename: &str) -> Option<Module> {
    let len = code.len();

    let cache = Source::from(code);

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

    // 字句解析エラーを表示
    if !lex_errors.is_empty() {
        for span in lex_errors {
            Report::build(ReportKind::Error, (filename, span.clone()))
                .with_message("Syntax Error: Invalid Token")
                .with_label(
                    Label::new((filename, span))
                        .with_message("Unable to parse this word")
                        .with_color(Color::Red),
                )
                .finish()
                .print((filename, &cache))
                .unwrap();
        }
        // 字句解析エラーがあった場合、構文解析はスキップ
        return None;
    }

    // 構文解析
    let result = parser().parse(IterInput::new(tokens.into_iter(), len..len));

    // 構文解析エラーを表示
    if result.has_errors() {
        for e in result.errors() {
            let mut expected = String::new();
            for token in e.expected() {
                if !expected.is_empty() {
                    expected += " or ";
                }
                match token {
                    RichPattern::Token(t) => expected += &format!("token {:?}", t),
                    RichPattern::Label(t) => expected += &format!("label {:?}", t),
                    RichPattern::Identifier(t) => expected += &format!("identifier {:?}", t),
                    RichPattern::Any => expected += "anything",
                    RichPattern::SomethingElse => expected += "something else",
                    RichPattern::EndOfInput => expected += "end of file",
                }
            }

            let label_msg = if e.found().is_none() {
                format!("Expected {}, but file ended", expected)
            } else {
                format!("Expected {}", expected)
            };

            Report::build(ReportKind::Error, (filename, e.span().clone()))
                .with_message("Syntax Error: Failed to parse")
                .with_label(
                    Label::<(&str, logos::Span)>::new((filename, e.span().clone()))
                        .with_message(label_msg)
                        .with_color(Color::Red),
                )
                .finish()
                .print((filename, &cache))
                .unwrap();
        }
        return None;
    }

    result.into_output()
}

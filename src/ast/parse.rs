use crate::{
    lexical::lexer,
    syntax::{File, parser},
};

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{Parser, error::RichPattern, input::IterInput};

pub fn parse(code: &str, filename: &str) -> Option<File> {
    let len = code.len();

    let cache = Source::from(code);

    // 字句解析
    let tokens = lexer(code);

    // 字句解析エラーを表示
    if let Err(errors) = tokens {
        for span in errors {
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
        return None;
    }

    let tokens = tokens.unwrap();

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
                    RichPattern::Token(t) => expected += &format!("{:?}", t),
                    RichPattern::Label(t) => expected += &format!("{:}", t),
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

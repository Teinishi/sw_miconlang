use crate::{
    compile_error::CompileError,
    lexical::tokenize,
    syntax::{File, Spanned, parser},
};

use ariadne::Source;
use chumsky::{Parser, input::IterInput};

pub fn parse(code: &str, filename: &str) -> Option<Spanned<File>> {
    let len = code.len();

    let cache = Source::from(code);

    // 字句解析
    let tokens = tokenize(code);

    // 字句解析エラーを表示
    if let Err(errors) = tokens {
        for span in errors {
            CompileError::invalid_token(filename, span).print(&cache);
        }
        return None;
    }

    let tokens = tokens.unwrap();

    // 構文解析
    let result = parser().parse(IterInput::new(tokens.into_iter(), len..len));

    // 構文解析エラーを表示
    if result.has_errors() {
        for e in result.errors() {
            CompileError::unexpected_token(filename, e).print(&cache);
        }
        return None;
    }

    result.into_output()
}

use crate::{
    compile_error::{CompileError, CompileErrorType},
    lexical::tokenize,
    semantic::analyze_file,
    syntax::parser,
    xml_schema,
};

use ariadne::Source;
use chumsky::{Parser, input::IterInput};
use std::collections::HashMap;

pub fn compile(code: &str, filename: &str, verbose: bool) -> Option<HashMap<String, String>> {
    let len = code.len();

    let cache = Source::from(code);

    // 字句解析
    let tokens = tokenize(code);
    if verbose && let Err(errors) = &tokens {
        for span in errors {
            CompileError::new(filename, span.clone(), CompileErrorType::InvalidToken).print(&cache);
        }
    }
    let tokens = tokens.ok()?;

    // 構文解析
    let tree = parser().parse(IterInput::new(tokens.into_iter(), len..len));
    if verbose && tree.has_errors() {
        for e in tree.errors() {
            CompileError::new(
                filename,
                e.span().clone(),
                CompileErrorType::unexpected_token(e),
            )
            .print(&cache);
        }
    }
    let tree = tree.into_output()?;

    // 意味解析
    let mcs = analyze_file(&tree, filename);
    if verbose && mcs.has_errors() {
        for e in mcs.errors() {
            e.print(&cache);
        }
    }
    let mcs = mcs.into_output()?;

    // XML生成
    let mut xml_files = HashMap::new();
    for (name, mc) in mcs {
        let mc_struct = xml_schema::Microprocessor::try_from(&mc.auto_layout())
            .expect("Unexpected Error: Generated microcontrooler is invalid");

        let mut buf = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n".to_owned();
        quick_xml::se::to_writer_with_root(&mut buf, "microprocessor", &mc_struct)
            .expect("Unexpected Error: XML Serialization Error");

        xml_files.insert(name, buf);
    }

    Some(xml_files)
}

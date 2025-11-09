/*mod ast;
mod compile_error;
mod lexical;
mod semantic;
mod syntax;*/
mod xml_schema;

/*use ast::parse;
use semantic::semantic_analyze;*/
use std::{
    env,
    fs::File,
    io::{Read as _, Write as _},
};

fn main() {
    let filename = env::args().nth(1).expect("expected file argument");
    let mut f = File::open(&filename).expect("file not found");
    let mut content = String::new();
    f.read_to_string(&mut content)
        .expect("something went wrong reading the file");

    /*
    // 読み込んだファイルをパース
    let result = parse(&content, &filename);
    dbg!(&result);
    if let Some(tree) = &result {
        semantic_analyze(tree);
    }
    */

    // 読み込んだファイルを input.xml に書き出す
    let mut file = File::create("input.xml").expect("cannot create input.xml");
    let _ = file
        .write(content.as_bytes())
        .expect("cannot write to output.xml");

    // マイコンXMLとしてデシリアライズ
    let mc: Result<xml_schema::Microprocessor, quick_xml::DeError> =
        quick_xml::de::from_str(&content);
    if let Err(err) = &mc {
        dbg!(err);
    }
    let mc = mc.unwrap();

    // マイコンXMLとしてシリアライズ
    let xml = quick_xml::se::to_string_with_root("microprocessor", &mc);
    if let Err(err) = xml {
        dbg!(err);
        return;
    }
    let xml = xml.unwrap();

    // シリアライズしたXMLを output.xml に書き出す
    let mut file = File::create("output.xml").expect("cannot create output.xml");
    let _ = file
        .write(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n")
        .expect("cannot write to output.xml");
    let _ = file
        .write(xml.as_bytes())
        .expect("cannot write to output.xml");
}

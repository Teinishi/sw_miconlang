#![warn(unused_extern_crates)]

mod ast;
mod compile_error;
mod lexical;
mod microcontroller;
mod semantic;
mod syntax;
mod xml_schema;

#[expect(unused)]
use ast::parse;
#[expect(unused)]
use semantic::analyze_file;
#[expect(unused)]
use std::{
    env,
    fs::File,
    io::{Read as _, Write as _},
    rc::Rc,
};

#[expect(unused)]
use crate::microcontroller::{
    Component, ComponentPosition, InputNode, Link, Microcontroller, Node, NodePosition, NodeType,
    OutputNode, PositionedComponent, PositionedMicrocontroller, PositionedNode,
    UnpositionedMicrocontroller,
};

fn main() {
    /*let filename = env::args().nth(1).expect("expected file argument");
    let mut f = File::open(&filename).expect("file not found");
    let mut content = String::new();
    f.read_to_string(&mut content)
        .expect("something went wrong reading the file");

    // 読み込んだファイルをパース
    let result = parse(&content, &filename);
    dbg!(&result);
    if let Some(tree) = &result {
        let _ = analyze_file(tree);
    }*/

    /*
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
    */

    let input_a = Rc::new(InputNode::new(
        "Input A".to_owned(),
        "The input signal to be processed.".to_owned(),
        NodeType::Number,
        NodePosition::new(0, 0),
    ));
    let input_b = Rc::new(InputNode::new(
        "Input B".to_owned(),
        "The input signal to be processed.".to_owned(),
        NodeType::Number,
        NodePosition::new(0, 1),
    ));

    let add = Rc::new(Component::Add {
        input_a: Some(Link::node(&input_a)),
        input_b: Some(Link::node(&input_b)),
    });
    let abs = Rc::new(Component::Abs {
        input: Some(Link::component(&add, 0)),
    });
    let mul = Rc::new(Component::Multiply {
        input_a: Some(Link::component(&add, 0)),
        input_b: Some(Link::component(&abs, 0)),
    });
    let div = Rc::new(Component::Divide {
        input_a: None,
        input_b: None,
    });

    let output_value = Rc::new(OutputNode::new(
        "Output".to_owned(),
        "The input signal to be processed.".to_owned(),
        NodeType::Number,
        NodePosition::new(0, 2),
        Some(Link::component(&mul, 0)),
    ));
    let output_div0 = Rc::new(OutputNode::new(
        "Div0".to_owned(),
        "The input signal to be processed.".to_owned(),
        NodeType::Bool,
        NodePosition::new(0, 3),
        Some(Link::component(&div, 1)),
        //None,
    ));

    let mc: UnpositionedMicrocontroller = Microcontroller {
        name: "Generated Microcontroller".to_owned(),
        description: "This is description".to_owned(),
        width: 1,
        length: 4,
        nodes: vec![
            Node::Input(input_a),
            Node::Input(input_b),
            Node::Output(output_value),
            Node::Output(output_div0),
        ],
        components: vec![add, abs, mul, div],
    };

    let mc_xml: Result<
        xml_schema::Microprocessor,
        xml_schema::conversion::MicroprocessorConversionError,
    > = (&mc.auto_layout()).try_into();
    if let Err(err) = mc_xml {
        dbg!(err);
        return;
    }
    let mc_xml = mc_xml.unwrap();

    // マイコンXMLとしてシリアライズ
    let xml = quick_xml::se::to_string_with_root("microprocessor", &mc_xml);
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

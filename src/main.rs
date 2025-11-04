mod ast;
mod lexical;
mod syntax;

use ast::parse;
use std::{env, fs::File, io::Read as _};

fn main() {
    let filename = env::args().nth(1).expect("expected file argument");
    let mut f = File::open(&filename).expect("file not found");
    let mut code = String::new();
    f.read_to_string(&mut code)
        .expect("something went wrong reading the file");

    let result = parse(&code, &filename);
    dbg!(result);
}

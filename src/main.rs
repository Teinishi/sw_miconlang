mod lexical;
mod parser;
mod syntax;

use parser::parse;
use std::{env, fs::File, io::Read as _};

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    let mut f = File::open(filename).expect("file not found");
    let mut code = String::new();
    f.read_to_string(&mut code)
        .expect("something went wrong reading the file");

    let result = parse(&code, filename);
    dbg!(result);
}

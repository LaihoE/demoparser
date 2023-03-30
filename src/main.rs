use parsing::parser::Parser;
mod parsing;

fn main() {
    let mut parser = Parser::new("/home/laiho/Documents/demos/cs2/s2-gotv.dem");
    parser.start();
}

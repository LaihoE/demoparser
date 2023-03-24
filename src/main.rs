use parsing::parser::Parser;

mod parsing;

fn main() {
    let mut parser = Parser::new();
    parser.start();
}

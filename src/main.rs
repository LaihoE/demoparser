use parsing::parser::Parser;
mod parsing;

fn main() {
    let mut parser = Parser::new("/home/laiho/Documents/demos/cs2/dem.dem");
    parser.start();
}

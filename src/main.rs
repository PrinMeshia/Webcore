mod ast;
mod parser;
mod codegen;

use std::fs;

fn main() {
    let input = fs::read_to_string("examples/hello.webc")
        .expect("Failed to read input file");

    match parser::parse_component(&input) {
        Ok(components) => {
            println!("Parsed components: {:#?}", components);
            if let Err(e) = codegen::generate(&components, "dist") {
                eprintln!("Codegen error: {}", e);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

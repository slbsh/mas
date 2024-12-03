mod report;

mod lexer;
mod parser;
mod preproc;

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::preproc::Preproc;

static mut HAS_ERROR: bool = false;

fn main() {
	let filename = std::env::args().nth(1).unwrap_or_else(|| {
		eprintln!("{}", report::Report::new("No input file").level(report::Level::Error));
		std::process::exit(1); 
	});

	let content = std::fs::read_to_string(&filename).unwrap();

	let tokens = Lexer::lex(&content);
	if unsafe { HAS_ERROR } { return; }

	let instr = Parser::parse(tokens);
	if unsafe { HAS_ERROR } { return; }

	instr.iter().for_each(|s| println!("{s}"));

}

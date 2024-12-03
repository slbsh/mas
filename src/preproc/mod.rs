use std::collections::HashMap;

use crate::parser::stmt::{Stmt, Dir};
use crate::report::{Report, Level, Pos, Error};

#[derive(Default, Debug)]
pub struct Preproc<'source> {
	stmts: Vec<Stmt<'source>>,

	//instr_defs: HashMap<&'sou
	reg_defs: HashMap<&'source str, usize>,
}

impl<'source> Preproc<'source> {
	pub fn preproc(stmts: Vec<Stmt<'source>>) -> Self {
		let mut preproc = Self { stmts, .. Default::default() };
		
		for stmt in preproc.stmts.iter() {
			match stmt {
				Stmt::Dir(d) => preproc.parse_directive(stmt),
				_ => {},
			}
		}

		println!("{preproc:?}");
		preproc
	}

	pub fn parse_directive(&self, stmt: &Dir<'source>) -> Result<(), Report> {
		match dir {
			Dir::Dreg(r, i) => self.reg_defs.insert(r, i),
			_ => todo!(),
		}
	}
}

enum Instr {
	Mem(Vec<u8>),
	Instr(Vec<u8>),
}

use crate::lexer::token::{Token, TokenKind};
use crate::report::{Report, Level, Pos, Error};

pub mod stmt;

use stmt::{Value, Stmt, Dir};

#[derive(Default)]
pub struct Parser<'source> {
	tokens: Vec<Token<'source>>,
	index:  usize,
	stmts:  Vec<Stmt<'source>>,
}

impl<'source> Parser<'source> {
	pub fn parse(tokens: Vec<Token<'source>>) -> Vec<Stmt<'source>> {
		let mut parser = Parser { tokens, .. Default::default() };
		parser.parse_program();
		parser.stmts
	}

	fn advance(&mut self) {
		self.index += 1;
		assert!(self.index <= self.tokens.len());
	}

	fn current(&self) -> Token<'source> {
		*self.tokens.get(self.index).unwrap()
	}

	fn peek(&self) -> Option<Token<'source>> {
		self.tokens.get(self.index + 1).copied()
	}

	pub fn parse_program(&mut self) {
		while self.index < self.tokens.len() {
			if matches!(self.current().kind, TokenKind::LF) {
				self.advance();
				continue;
			}

			match self.parse_statement() {
				Ok(stmt) => self.stmts.push(stmt),
				Err(e)   => {
					println!("{e}");
					while !matches!(self.current().kind, TokenKind::LF) { self.advance(); }
				},
			}
		}
	}

	fn parse_statement(&mut self) -> Result<Stmt<'source>, Report> {
		match self.current().kind {
			TokenKind::Ident if matches!(self.peek(), Some(Token { kind: TokenKind::Colon, .. })) => { 
				let out = Stmt::Sym(self.current().lit);
				self.advance();
				Ok(out)
			},
			TokenKind::Ident => {
				let token = self.current();
				self.advance();

				if matches!(self.current().kind, TokenKind::LF) { 
					return Ok(Stmt::Instr(token.lit, Vec::new())); 
				}

				let mut args = Vec::with_capacity(4); // value is small, we can afford to overallocate
				loop { 
					args.push(self.parse_value()?);

					if matches!(self.current().kind, TokenKind::LF) { break; }

					match self.current().kind {
						TokenKind::Comma => self.advance(),
						_ => return Err(Error::UnexpectedToken(format!("{:?}", self.current())).span(self.current().pos)),
					}
				}

				Ok(Stmt::Instr(token.lit, args))
			},
			TokenKind::Dot   => {
				self.advance();
				Ok(Stmt::Dir(self.parse_directive()?))
			},
			_ => Err(Error::UnexpectedToken(format!("{:?}", self.current())).span(self.current().pos)),
		}
	}

	fn parse_directive(&mut self) -> Result<Dir<'source>, Report> {
		let token = self.current();
		self.advance();

		match token.lit {
			"org"    => Ok(Dir::Origin(self.parse_immidiate()?)),
			"byte"   => Ok(Dir::Byte(self.lit_bytes_lf()?)),
			"word"   => Ok(Dir::Word(self.lit_bytes_lf()?)),
			"ascii"  => Ok(Dir::Ascii(self.lit_bytes_lf()?)),
			"asciiz" => Ok(Dir::Asciiz(self.lit_bytes_lf()?)),
			"dword"  => Ok(Dir::Dword(self.parse_immidiate()?)),
			"dreg"   => Ok(Dir::Dreg(self.parse_register()?, self.parse_immidiate()?)),
			d => Err(Report::new(format!("Unknown directive {d:?}")).span(token.pos)),
		}
	}

	fn parse_args(&mut self) -> Result<(Value<'source>, Value<'source>), Report> {
		let arg1 = self.parse_value()?;

		match self.current().kind {
			TokenKind::Comma => {
				self.advance();
				Ok((arg1, self.parse_value()?))
			},
			_ => Err(Report::new(format!("expected comma, got {:?}", self.current())).span(self.current().pos)),
		}
	}

	fn parse_value(&mut self) -> Result<Value<'source>, Report> {
		let token = self.current();
		self.advance();

		match token.kind {
			TokenKind::Dollar  => Ok(Value::Imm(self.parse_immidiate()?)),
			TokenKind::Percent => Ok(Value::Reg(self.parse_register()?)),
			TokenKind::Ident   => Ok(Value::Sym(token.lit)),
			_ => Err(Error::UnexpectedToken(format!("{:?}", token)).span(token.pos)),
		}
	}

	fn parse_register(&mut self) -> Result<&'source str, Report> {
		let token = self.current();
		self.advance();

		match token {
			Token { kind: TokenKind::Ident, .. } => Ok(token.lit),
			_ => Err(Error::UnexpectedToken(format!("{token:?}")).span(token.pos)),
		}
	}

	// prob keep type meta
	fn parse_immidiate(&mut self) -> Result<usize, Report> {
		let token = self.current();
		self.advance();

		#[allow(clippy::transmute_float_to_int)]
		match token.kind {
			// maybe not
			TokenKind::IntLit if matches!(token.lit.chars().next().unwrap(), '+' | '-') 
				=> token.lit.parse::<isize>()
					.map_err(|e| Error::ParseError(e.into()).span(token.pos))
					.map(|n| unsafe { std::mem::transmute::<isize, usize>(n) }),

			TokenKind::IntLit
				=> token.lit.parse::<usize>().map_err(|e| Error::ParseError(e.into()).span(token.pos)),

			TokenKind::FloatLit
				=> token.lit.parse::<f64>()
					.map_err(|e| Error::ParseError(e.into()).span(token.pos))
					.map(|n| unsafe { std::mem::transmute::<f64, usize>(n) }),

			TokenKind::CharLit => match token.lit {
				"" => Err(Report::new("Empty char literal").span(token.pos)),
				s if s.starts_with('\\') && s.len() == 1 
					=> Err(Report::new("Trailing escape in literal").span(token.pos)),
				s if s.starts_with("\\") 
					=> escape_map(unsafe { s.chars().nth(2).unwrap_unchecked() })
						.map_err(|e| e.span(token.pos))
						.map(usize::from),
				_ => Ok(token.lit.as_bytes()[0] as usize),
			},
			_ => Err(Error::UnexpectedToken(format!("{:?}", token)).span(token.pos)),
		}
	}

	fn lit_bytes_lf(&mut self) -> Result<Vec<u8>, Report> {
		let mut buf = Vec::new();

		while matches!(self.current().kind, TokenKind::LF) { 
			match self.current().kind {
				TokenKind::StrLit => buf.extend(self.parse_string()?),
				TokenKind::CharLit | TokenKind::IntLit | TokenKind::FloatLit =>
					buf.extend_from_slice(&self.parse_immidiate()?.to_ne_bytes()),
				_ => return Err(Error::UnexpectedToken(format!("{:?}", self.current())).span(self.current().pos)),
			}
		}

		Ok(buf)
	}

	fn parse_string(&mut self) -> Result<Vec<u8>, Report> {
		let token = self.current();
		self.advance();

		match token.kind {
			TokenKind::StrLit => {
				let mut buf = Vec::with_capacity(token.lit.len());
				for c in token.lit.chars() { 
					buf.push(escape_map(c).map_err(|e| e.span(token.pos))?); 
				}
				Ok(buf)
			},
			_ => Err(Error::UnexpectedToken(format!("{:?}", token)).span(token.pos)),
		}
	}
}

fn escape_map(c: char) -> Result<u8, Report> {
	Ok(match c {
		'0' | '@' => 0,
		'A'       => 1,
		'B'       => 2,
		'C'       => 3,
		'D'       => 4,
		'E'       => 5,
		'F'       => 6,
		'G' | 'a' => 7,
		'H' | 'b' => 8,
		'I' | 't' => 9,
		'J' | 'n' => 10,
		'K' | 'v' => 11,
		'L' | 'f' => 12,
		'M' | 'r' => 13,
		'N'       => 14,
		'O'       => 15,
		'P'       => 16,
		'Q'       => 17,
		'R'       => 18,
		'S'       => 19,
		'T'       => 20,
		'U'       => 21,
		'V'       => 22,
		'W'       => 23,
		'X'       => 24,
		'Y'       => 25,
		'Z'       => 26,
		'[' | 'e' => 27,
		'/'       => 28,
		']'       => 29,
		'^'       => 30,
		'_'       => 31,
		'?'       => 32,
		'\\'      => b'\\',
		'\''      => b'\'',
		'"'       => b'"',
		_         => Err(Report::new("Invalid escape sequence"))?,
	})
}

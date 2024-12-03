use crate::report::{Pos, Report, Level, Error};

pub mod token;

use token::{Token, TokenKind};

#[derive(Default)]
pub struct Lexer<'source> {
	slice:  &'source str,
	index:  usize,
	tokens: Vec<Token<'source>>,
	pos:    Pos,
}

impl<'source> Lexer<'source> {
	fn token_simple(&mut self, kind: TokenKind) {
		self.advance(1);
		self.push_range(kind, self.index-1, self.index);
	}

	fn get(&self, n: usize) -> Option<char> {
		self.slice.chars().nth(n)
	}

	fn consume_while<F: FnMut(char) -> bool>(&mut self, mut f: F) -> Option<()> {
		loop {
			let Some(c) = self.get(self.index) else {
				println!("{}", Error::UnexpectedEOF.span(self.pos));
				return None;
			};

			if f(c) { self.advance(1); } 
			else { return Some(()); }
		}
	}

	fn push_range(&mut self, kind: TokenKind, start: usize, end: usize) {
		self.tokens.push(Token { 
			kind, 
			lit: unsafe { self.slice.get_unchecked(start..end)},
			pos: self.pos 
		});
	}

	fn advance(&mut self, n: usize) {
		self.index += n;
		self.pos.index += n;
	}

	pub fn lex(slice: &'source str) -> Vec<Token<'source>> {
		let mut lex = Lexer { slice, .. Default::default() };

		loop {
			let Some(lit) = lex.slice.get(lex.index..=lex.index) 
				else { break; };

			let start_pos = lex.index;

			match unsafe { lit.chars().next().unwrap_unchecked() } {
				'a'..='z' | 'A'..='Z' | '_' => { 
					lex.consume_while(|c| c.is_ascii_alphanumeric() || c == '_');
					lex.push_range(TokenKind::Ident, start_pos, lex.index);
				},

				'0'..='9' => {
					let mut kind = TokenKind::IntLit;
					lex.consume_while(|c| c.is_ascii_digit());
					if lex.get(lex.index) == Some('.') {
						lex.advance(1);
						kind = TokenKind::FloatLit;
						lex.consume_while(|c| c.is_ascii_digit());
					}

					lex.push_range(kind, start_pos, lex.index);
				},

				'\'' => {
					lex.index += 1;
					match lex.get(lex.index) {
						Some('\\') => lex.advance(2),
						Some(_)    => lex.advance(1),
						None       => {
							println!("{}", Error::UnexpectedEOF.span(lex.pos));
							break;
						},
					}

					lex.advance(1);
					if lex.get(lex.index-1) != Some('\'') {
						println!("{}", Report::new("Unterminated char literal").span(lex.pos));
						continue;
					}

					lex.push_range(TokenKind::CharLit, start_pos+1, lex.index-1);
				},

				'"' => {
					loop {
						lex.advance(1);
						match lex.get(lex.index) {
							Some('"')  => { lex.advance(1); break; },
							Some('\\') => lex.advance(2),
							Some(_)    => lex.advance(1),
							None       => {
								println!("{}", Error::UnexpectedEOF.span(lex.pos));
								break;
							},
						}
					}
					
					lex.push_range(TokenKind::StrLit, start_pos+1, lex.index-1);
				},

				' ' | '\t' => {
					lex.advance(1);
					continue;
				},

				';' => {
					lex.index += 1;
					while let Some(c) = lex.get(lex.index) {
						if c == '\n' { 
							lex.index += 1;
							lex.pos.line += 1;
							lex.pos.index = 0;
							break; 
						}
						lex.index += 1;
					}
				},

				'+'  => lex.token_simple(TokenKind::Plus),
				'-'  => lex.token_simple(TokenKind::Minus),
				'('  => lex.token_simple(TokenKind::LParen),
				')'  => lex.token_simple(TokenKind::RParen),
				'.'  => lex.token_simple(TokenKind::Dot),
				','  => lex.token_simple(TokenKind::Comma),
				':'  => lex.token_simple(TokenKind::Colon),
				'$'  => lex.token_simple(TokenKind::Dollar),
				'%'  => lex.token_simple(TokenKind::Percent),
				'@'  => lex.token_simple(TokenKind::At),
				'\n' => {
					lex.pos.line += 1;
					lex.pos.index = 0;
					lex.token_simple(TokenKind::LF)
				},
				c => {
					println!("{}", Report::new(format!("Unexpected character {c:?}"))
							.span(lex.pos).level(Level::Error));
					break;
				},
			}
		}

		lex.tokens
	}
}

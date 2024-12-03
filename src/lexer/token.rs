use std::fmt;

#[derive(Clone, Copy)]
pub struct Token<'source> {
	pub kind: TokenKind,
	pub lit:  &'source str,
	pub pos:  crate::report::Pos,
}

impl<'source> fmt::Debug for Token<'source> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}({}, {:?})", self.kind, self.pos, self.lit)
	}
}

#[derive(Debug, Clone, Copy)]
pub enum TokenKind {
	Ident,

	IntLit,
	FloatLit,
	CharLit,
	StrLit,

	Dot,
	Comma,
	Colon,
	Dollar,
	Percent,
	At,
	Plus,
	Minus,
	LParen,
	RParen,

	LF,
}

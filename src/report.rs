use std::fmt;

#[allow(clippy::enum_variant_names)]
pub enum Error {
	UnexpectedEOF,
	UnexpectedToken(String),
	ParseError(Box<dyn std::error::Error>),
}

impl Error {
	pub fn into(self) -> Report {
		match self {
			Error::UnexpectedEOF      => Report::new("Unexpected EOF"),
			Error::UnexpectedToken(s) => Report::new(format!("Unexpected token {s}")),
			Error::ParseError(e)      => Report::new(format!("Parse error: {e}")),
		}
	}

	pub fn span(self, pos: Pos) -> Report {
		self.into().span(pos)
	}
}


#[derive(Clone, Copy)]
pub struct Pos {
	pub line:  usize,
	pub index: usize,
}

impl Default for Pos {
	fn default() -> Self 
		{ Self { line: 1, index: 0 } }
}

impl Pos {
	pub fn new(line: usize, index: usize) -> Self 
		{ Self { line, index } }
}

impl fmt::Display for Pos {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}:{}", self.line, self.index)
	}
}

pub enum Level {
	Info,
	Warning,
	Error,
}

impl fmt::Display for Level {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", match self {
			Level::Info    => "\x1b[1mINFO\x1b[0m",
			Level::Warning => "\x1b[33;1mWARN\x1b[0m",
			Level::Error   => "\x1b[31;1mERROR\x1b[0m",
		})
	}
}


pub struct Report {
	pub span:    Option<Pos>,
	pub level:   Level,
	pub message: Box<dyn std::fmt::Display>,
}

impl Report {
	pub fn new(message: impl std::fmt::Display + 'static) -> Self {
		Self { 
			level:   Level::Error,
			span:    None,
			message: Box::new(message) 
		}
	}

	pub fn span(mut self, span: Pos) -> Self {
		self.span = Some(span); self
	}

	pub fn level(mut self, level: Level) -> Self {
		self.level = level; self
	}
}

impl fmt::Display for Report {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if matches!(self.level, Level::Error) { unsafe { crate::HAS_ERROR = true; } }
		match &self.span {
			Some(span) => write!(f, "{} {span}: {}", self.level, self.message),
			None       => write!(f, "{} {}", self.level, self.message),
		}
	}
}

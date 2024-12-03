use std::fmt;

pub enum Value<'source> {
	Reg(&'source str),
	Imm(usize),
	Sym(&'source str),
}

impl fmt::Display for Value<'_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Reg(r) => write!(f, "%{r}"),
			Self::Imm(i) => write!(f, "${i}"),
			Self::Sym(s) => write!(f, "{s}"),
		}
	}
}

pub enum Dir<'source> {
	Origin(usize),
	Byte(Vec<u8>),
	Word(Vec<u8>),
	Ascii(Vec<u8>),
	Asciiz(Vec<u8>),
	Dreg(&'source str, usize),
	Dword(usize),
}

impl fmt::Display for Dir<'_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Origin(o)  => write!(f, "org {o}"),
			Self::Byte(b)    => write!(f, "byte {}",   unsafe { std::str::from_utf8_unchecked(b) }),
			Self::Word(w)    => write!(f, "word {}",   unsafe { std::str::from_utf8_unchecked(w) }),
			Self::Ascii(a)   => write!(f, "ascii {}",  unsafe { std::str::from_utf8_unchecked(a) }),
			Self::Asciiz(z)  => write!(f, "asciiz {}", unsafe { std::str::from_utf8_unchecked(z) }),
			Self::Dreg(r, i) => write!(f, "dreg %{r}, ${i}"),
			Self::Dword(d)   => write!(f, "dword {d}"),
		}
	}
}


pub enum Stmt<'source> {
	Instr(&'source str, Vec<Value<'source>>),
	Sym(&'source str),
	Dir(Dir<'source>),
}

impl fmt::Display for Stmt<'_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Instr(i, v) => write!(f, "{i} {}", v.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ")),
			Self::Dir(d)      => write!(f, ".{d}"),
			Self::Sym(s)      => write!(f, "{s}:"),
		}
	}
}

impl fmt::Debug for Stmt<'_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "({self})")
	}
}

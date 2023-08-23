use super::{
    error::ReaderError,
    token::{Token, TokenKind},
};
use logos::{Lexer, Logos};
use lust_util::{
    list::List,
    span::{Span, Spanned},
};

pub struct Reader<'src> {
    src: &'src str,
    lexer: Lexer<'src, TokenKind>,
    peek: Option<Token>,
    errors: Vec<ReaderError>,
}

impl<'src> Reader<'src> {
    /// Creates a new [`Parser`].
    pub fn new(src: &'src str) -> Self {
        Self {
            src,
            lexer: TokenKind::lexer(src),
            peek: None,
            errors: vec![],
        }
    }

    fn fetch_token(&mut self) -> Token {
        match self.lexer.next().map(|res| (res, self.lexer.span())) {
            Some((res, s)) => match res {
                Ok(t) => t.spanned(s.into()),
                Err(_) => {
                    self.errors.push(SyntaxError::LexerError.spanned(s.into()));
                    self.fetch_token()
                }
            },
            None => TokenKind::Eof.spanned(self.lexer.span().into()),
        }
    }

    // /// Parses the source code into a [`Sexpr`].
    // pub fn sexpr(&mut self) -> ReadResult<Spanned<Sexpr>> {
    //     match self.tokens.peek().0 {
    //         Token::LParen => self.list(),
    //         _ => self.atom(),
    //     }
    // }

    // fn list(&mut self) -> ReadResult<Spanned<Sexpr>> {
    //     let start = self.tokens.peek().1.start as usize;
    //     if !self.tokens.eat(&Token::LParen) {
    //         return Err(ReaderError::UnmatchedParen(self.tokens.peek().1));
    //     }
    //     let mut sexprs = vec![];
    //     while !self.tokens.at(&Token::RParen) {
    //         let s = self.sexpr()?;
    //         sexprs.push(s);
    //         self.tokens.next();
    //     }
    //     if !self.tokens.eat(&Token::RParen) {
    //         return Err(ReaderError::UnmatchedParen(self.tokens.peek().1));
    //     }
    //     let list: List<Spanned<Sexpr>> = sexprs.into_iter().rev().collect();
    //     let end = self.tokens.peek().1.end as usize;
    //     Ok((Sexpr::Cons(Box::new(list)), Span::from(start..end)))
    // }

    // fn atom(&mut self) -> ReadResult<Spanned<Sexpr>> {
    //     match self.tokens.peek().0 {
    //         Token::Int(i) => Ok((Sexpr::Atom(Atom::Lit(Lit::Int(i))), self.tokens.peek().1)),
    //         Token::Rational(r) => Ok((
    //             Sexpr::Atom(Atom::Lit(Lit::Rational(r))),
    //             self.tokens.peek().1,
    //         )),
    //         Token::Real(r) => Ok((Sexpr::Atom(Atom::Lit(Lit::Real(r))), self.tokens.peek().1)),
    //         Token::Char(c) => Ok((Sexpr::Atom(Atom::Lit(Lit::Char(c))), self.tokens.peek().1)),
    //         Token::String(s) => Ok((
    //             Sexpr::Atom(Atom::Lit(Lit::String(InternedString::from(
    //                 &s[1..(s.len() - 1)],
    //             )))),
    //             self.tokens.peek().1,
    //         )),
    //         Token::Ident(name) => Ok((
    //             Sexpr::Atom(Atom::Symbol(InternedString::from(name))),
    //             self.tokens.peek().1,
    //         )),
    //         _ => Err(ReaderError::UnexpectedToken(self.tokens.peek())),
    //     }
    // }
}

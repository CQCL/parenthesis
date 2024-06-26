//! Reading s-expressions from strings.
use logos::Logos;
use smol_str::SmolStr;
use std::ops::Range;
use thiserror::Error;

use crate::escape::unescape;
use crate::from_parens::{FromParens, InputStream, ParseError, TokenTree};
use crate::Symbol;

#[derive(Debug, Clone, PartialEq, Logos)]
#[logos(skip r"[ \t\n\f]+")]
enum Token {
    #[token("(", |_| 0)]
    OpenList(usize),

    #[token(")")]
    CloseList,

    #[regex(
        r#""([^"\\]|\\["\\tnr]|u\{[a-fA-F0-9]+\})*""#,
        |lex| Some(unescape(&lex.slice()[1..lex.slice().len() - 1])?.into())
    )]
    String(SmolStr),

    #[regex(
        r#"[a-zA-Z!$%&*/:<=>?\^_~\.@][a-zA-Z!$%&*/:<=>?\^_~0-9+\-\.@]*"#,
        |lex| Symbol::new(lex.slice())
    )]
    #[regex(
        r#"[+-]([a-zA-Z!$%&*/:<=>?\^_~\.@][a-zA-Z!$%&*/:<=>?\^_~0-9+\-\.@]*)?"#,
        |lex| Symbol::new(lex.slice())
    )]
    #[regex(
        r#"\|([^\|\\]|\\u\{[a-fA-F0-9]{1,4}\};|\\[\|\\tnr])*\|"#,
        |lex| Some(unescape(&lex.slice()[1..lex.slice().len() - 1])?.into())
    )]
    Symbol(Symbol),

    #[regex(";[^\n]*\n")]
    Comment,

    #[token("#t", |_| Some(true))]
    #[token("#f", |_| Some(false))]
    Bool(bool),

    #[regex("[+-]?[0-9]+", |lex| lex.slice().parse().map_err(|_| ()), priority = 0)]
    Int(i64),

    #[regex(
        r#"[+-]?[0-9]+\.[0-9]*([eE][+-]?[0-9]+)?"#r,
        |lex| lex.slice().parse().map_err(|_| ()),
        priority = 1
    )]
    #[token("#+inf", |_| f64::INFINITY)]
    #[token("#-inf", |_| -f64::INFINITY)]
    #[token("#nan", |_| f64::NAN)]
    Float(f64),
}

/// Span within a string.
pub type Span = Range<usize>;

/// Error while reading a value from an s-expression string.
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum ReadError {
    #[error("unrecognized syntax")]
    Syntax { span: Span },
    #[error("unexpected end of file")]
    EndOfFile,
    #[error("unexpected closing delimiter")]
    UnexpectedClose { span: Span },
    #[error("expected whitespace")]
    ExpectedWhitespace { after: Span, before: Span },
    #[error(transparent)]
    Parse(#[from] ParseError<Span>),
}

/// Read a value of type `T` from an s-expression string.
pub fn from_str<T>(str: &str) -> Result<T, ReadError>
where
    T: for<'a> FromParens<ReaderStream<'a>>,
{
    let mut tokens: Vec<_> = Token::lexer(str)
        .spanned()
        .filter(|(token, _)| !matches!(token, Ok(Token::Comment)))
        .map(|(token, span)| match token {
            Ok(token) => Ok((token, span)),
            Err(()) => Err(ReadError::Syntax { span: span.clone() }),
        })
        .collect::<Result<_, _>>()?;

    check_whitespace(&tokens)?;
    balance_lists(&mut tokens)?;

    let result = T::from_parens(&mut ReaderStream {
        tokens: &tokens,
        cur_span: 0..0,
        parent_span: 0..str.len(),
    })?;

    Ok(result)
}

fn check_whitespace(tokens: &[(Token, Span)]) -> Result<(), ReadError> {
    for window in tokens.windows(2) {
        let (token_a, span_a) = &window[0];
        let (token_b, span_b) = &window[1];

        match token_a {
            Token::OpenList(_) => continue,
            Token::Comment => continue,
            _ => {}
        }

        match token_b {
            Token::CloseList => continue,
            Token::Comment => continue,
            _ => {}
        }

        if span_a.end == span_b.start {
            return Err(ReadError::ExpectedWhitespace {
                after: span_a.clone(),
                before: span_b.clone(),
            });
        }
    }

    Ok(())
}

/// Check that the parentheses are well-balanced and make the OpenList
/// tokens reflect the distance to their associated CloseList tokens.
fn balance_lists(tokens: &mut [(Token, Span)]) -> Result<(), ReadError> {
    // Stack that holds the indices of all currently unclosed `(`s.
    let mut stack = Vec::new();

    for i in 0..tokens.len() {
        let (token, span) = &tokens[i];

        match token {
            Token::OpenList(_) => stack.push(i),
            Token::CloseList => {
                let Some(j) = stack.pop() else {
                    return Err(ReadError::UnexpectedClose { span: span.clone() });
                };

                tokens[j].0 = Token::OpenList(i - j);
            }
            _ => {}
        }
    }

    if !stack.is_empty() {
        return Err(ReadError::EndOfFile);
    }

    Ok(())
}

/// FromParens stream used by [`from_str`].
#[derive(Clone)]
pub struct ReaderStream<'a> {
    tokens: &'a [(Token, Span)],
    cur_span: Span,
    parent_span: Span,
}

impl<'a> InputStream for ReaderStream<'a> {
    type Span = Span;

    fn next(&mut self) -> Option<TokenTree<Self>> {
        match self.peek()? {
            TokenTree::List(inner) => {
                self.cur_span = inner.parent_span.clone();
                self.tokens = &self.tokens[inner.tokens.len() + 2..];
                Some(TokenTree::List(inner))
            }
            token_tree => {
                self.cur_span = self.tokens[0].1.clone();
                self.tokens = &self.tokens[1..];
                Some(token_tree)
            }
        }
    }

    fn peek(&self) -> Option<TokenTree<Self>> {
        let (token, span) = self.tokens.first()?;

        match token {
            Token::OpenList(skip) => Some(TokenTree::List(ReaderStream {
                tokens: &self.tokens[1..*skip],
                cur_span: span.end..span.end,
                parent_span: span.end..self.tokens[*skip].1.end,
            })),
            Token::CloseList => None,
            Token::String(string) => Some(TokenTree::String(string.clone())),
            Token::Symbol(symbol) => Some(TokenTree::Symbol(symbol.clone())),
            Token::Comment => unreachable!("comments have been stripped before"),
            Token::Bool(bool) => Some(TokenTree::Bool(*bool)),
            Token::Int(int) => Some(TokenTree::Int(*int)),
            Token::Float(float) => Some(TokenTree::Float(*float)),
        }
    }

    fn span(&self) -> Self::Span {
        self.cur_span.clone()
    }

    fn parent_span(&self) -> Self::Span {
        self.parent_span.clone()
    }

    fn is_end(&self) -> bool {
        self.tokens.is_empty()
    }
}

#[cfg(test)]
mod test {
    use super::from_str;
    use crate::Value;
    use rstest::rstest;

    #[rstest]
    #[case("+#f")]
    #[case("++inf.0")]
    #[case("()()")]
    fn require_whitespace(#[case] text: &str) {
        assert!(from_str::<Vec<Value>>(text).is_err());
    }
}

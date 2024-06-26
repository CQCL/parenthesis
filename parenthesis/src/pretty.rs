//! Pretty print s-expressions.
use std::convert::Infallible;

use crate::{
    escape::{escape_string, escape_symbol},
    to_parens::{OutputStream, ToParens},
};
use pretty::BoxDoc;

/// Pretty prints a value of type `T` into an s-expression by writing into an
/// [`std::fmt::Write`].
pub fn to_fmt_pretty<W, P>(value: P, width: usize, f: &mut W) -> std::fmt::Result
where
    W: std::fmt::Write,
    P: ToParens<Pretty>,
{
    let mut pretty = Pretty::new();
    let _ = value.to_parens(&mut pretty);
    let doc = pretty.finish();
    doc.render_fmt(width, f)
}

/// Pretty prints a value that implements [`ToParens`] into an s-expression string.
pub fn to_string_pretty<T>(value: T, width: usize) -> String
where
    T: ToParens<Pretty>,
{
    let mut string = String::new();
    let _ = to_fmt_pretty(value, width, &mut string);
    string
}

/// Output stream used by [`to_string_pretty`] and [`to_fmt_pretty`].
pub struct Pretty {
    stack: Vec<Vec<BoxDoc<'static>>>,
    current: Vec<BoxDoc<'static>>,
}

impl Pretty {
    fn new() -> Self {
        Self {
            stack: Vec::new(),
            current: Vec::new(),
        }
    }

    fn finish(self) -> BoxDoc<'static> {
        BoxDoc::intersperse(self.current, BoxDoc::line())
    }
}

impl OutputStream for Pretty {
    type Error = Infallible;

    fn list<F, R>(&mut self, f: F) -> Result<R, Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<R, Self::Error>,
    {
        self.stack.push(std::mem::take(&mut self.current));
        let result = f(self);
        let docs = std::mem::replace(&mut self.current, self.stack.pop().unwrap());

        self.current.push(
            BoxDoc::text("(")
                .append(BoxDoc::intersperse(docs, BoxDoc::line()).nest(2).group())
                .append(BoxDoc::text(")")),
        );

        result
    }

    fn string(&mut self, string: impl AsRef<str>) -> Result<(), Self::Error> {
        let escaped = escape_string(string.as_ref());
        self.current.push(BoxDoc::text(format!(r#""{}""#, escaped)));
        Ok(())
    }

    fn symbol(&mut self, symbol: impl AsRef<str>) -> Result<(), Self::Error> {
        let escaped = escape_symbol(symbol.as_ref());
        self.current.push(BoxDoc::text(escaped));
        Ok(())
    }

    fn bool(&mut self, bool: bool) -> Result<(), Self::Error> {
        self.current.push(BoxDoc::text(match bool {
            true => "#t",
            false => "#f",
        }));
        Ok(())
    }

    fn int(&mut self, int: i64) -> Result<(), Self::Error> {
        self.current.push(BoxDoc::text(int.to_string()));
        Ok(())
    }

    fn float(&mut self, float: f64) -> Result<(), Self::Error> {
        let text = if float.is_nan() {
            "#nan".to_string()
        } else if float == f64::INFINITY {
            "#+inf".to_string()
        } else if float == -f64::INFINITY {
            "#-inf".to_string()
        } else if float == float.ceil() {
            // To ensure that floats are not confused with ints after printing
            // we always include a decimal point.
            format!("{}.0", float)
        } else {
            float.to_string()
        };

        self.current.push(BoxDoc::text(text));
        Ok(())
    }
}

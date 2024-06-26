//! Types that can be converted to s-expressions.
use smol_str::SmolStr;
use std::{
    borrow::{Borrow, Cow},
    convert::Infallible,
};

use crate::{Symbol, Value};

/// Output stream that s-expressions can be written to.
pub trait OutputStream {
    /// Error while writing into the output stream.
    type Error;

    /// Write a list to the output stream, whose elements are written by the given function.
    fn list<F, R>(&mut self, f: F) -> Result<R, Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<R, Self::Error>;

    /// Write a string to the output stream.
    fn string(&mut self, string: impl AsRef<str>) -> Result<(), Self::Error>;

    /// Write a symbol to the output stream.
    fn symbol(&mut self, symbol: impl AsRef<str>) -> Result<(), Self::Error>;

    /// Write a boolean to the output stream.
    fn bool(&mut self, bool: bool) -> Result<(), Self::Error>;

    /// Write an integer to the output stream.
    fn int(&mut self, int: i64) -> Result<(), Self::Error>;

    /// Write a float to the output stream.
    fn float(&mut self, float: f64) -> Result<(), Self::Error>;
}

/// Types that can be converted to an s-expression.
pub trait ToParens<O>
where
    O: OutputStream,
{
    /// Print an s-expression representation into the given output stream.
    fn to_parens(&self, output: &mut O) -> Result<(), O::Error>;
}

impl<O> ToParens<O> for Value
where
    O: OutputStream,
{
    fn to_parens(&self, output: &mut O) -> Result<(), <O as OutputStream>::Error> {
        match self {
            Value::List(list) => output.list(|output| list.to_parens(output)),
            Value::String(string) => output.string(string),
            Value::Symbol(symbol) => output.symbol(symbol),
            Value::Bool(bool) => output.bool(*bool),
            Value::Int(int) => output.int(*int),
            Value::Float(float) => output.float(float.into_inner()),
        }
    }
}

impl<O> ToParens<O> for SmolStr
where
    O: OutputStream,
{
    #[inline]
    fn to_parens(&self, output: &mut O) -> Result<(), <O as OutputStream>::Error> {
        output.string(self)
    }
}

impl<O> ToParens<O> for String
where
    O: OutputStream,
{
    #[inline]
    fn to_parens(&self, output: &mut O) -> Result<(), <O as OutputStream>::Error> {
        output.string(self)
    }
}

impl<O> ToParens<O> for Symbol
where
    O: OutputStream,
{
    #[inline]
    fn to_parens(&self, output: &mut O) -> Result<(), <O as OutputStream>::Error> {
        output.symbol(self)
    }
}

impl<O, V> ToParens<O> for Vec<V>
where
    O: OutputStream,
    V: ToParens<O>,
{
    fn to_parens(&self, output: &mut O) -> Result<(), O::Error> {
        for value in self.iter() {
            value.to_parens(output)?;
        }

        Ok(())
    }
}

impl<O, V> ToParens<O> for [V]
where
    O: OutputStream,
    V: ToParens<O>,
{
    fn to_parens(&self, output: &mut O) -> Result<(), O::Error> {
        for value in self.iter() {
            value.to_parens(output)?;
        }

        Ok(())
    }
}

impl<O> ToParens<O> for f64
where
    O: OutputStream,
{
    #[inline]
    fn to_parens(&self, output: &mut O) -> Result<(), <O as OutputStream>::Error> {
        output.float(*self)
    }
}

impl<O> ToParens<O> for i64
where
    O: OutputStream,
{
    #[inline]
    fn to_parens(&self, output: &mut O) -> Result<(), <O as OutputStream>::Error> {
        output.int(*self)
    }
}

impl<O, T> ToParens<O> for &T
where
    T: ToParens<O>,
    O: OutputStream,
{
    #[inline]
    fn to_parens(&self, output: &mut O) -> Result<(), <O as OutputStream>::Error> {
        T::to_parens(*self, output)
    }
}

impl<'a, O, T> ToParens<O> for Cow<'a, T>
where
    T: ToParens<O> + Clone,
    O: OutputStream,
{
    #[inline]
    fn to_parens(&self, output: &mut O) -> Result<(), <O as OutputStream>::Error> {
        T::to_parens(self.borrow(), output)
    }
}

#[cfg(feature = "macros")]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
pub use parenthesis_macros::ToParens;

/// Convert a value of type `T` to a vector of [`Value`]s.
pub fn to_values<T>(value: T) -> Vec<Value>
where
    T: ToParens<ValueOutputStream>,
{
    let mut output = ValueOutputStream::new();
    let _ = value.to_parens(&mut output);
    output.finish()
}

/// Output stream used for [`to_values`].
pub struct ValueOutputStream {
    stack: Vec<Vec<Value>>,
    current: Vec<Value>,
}

impl ValueOutputStream {
    fn new() -> Self {
        Self {
            stack: Vec::new(),
            current: Vec::new(),
        }
    }

    fn finish(self) -> Vec<Value> {
        self.current
    }
}

impl OutputStream for ValueOutputStream {
    type Error = Infallible;

    fn list<F, R>(&mut self, f: F) -> Result<R, Self::Error>
    where
        F: FnOnce(&mut Self) -> Result<R, Self::Error>,
    {
        self.stack.push(std::mem::take(&mut self.current));
        let result = f(self);
        let list = std::mem::replace(&mut self.current, self.stack.pop().unwrap());
        self.current.push(Value::List(list));
        result
    }

    fn string(&mut self, string: impl AsRef<str>) -> Result<(), Self::Error> {
        self.current.push(Value::from(string.as_ref()));
        Ok(())
    }

    fn symbol(&mut self, symbol: impl AsRef<str>) -> Result<(), Self::Error> {
        self.current.push(Value::from(Symbol::new(symbol)));
        Ok(())
    }

    fn bool(&mut self, bool: bool) -> Result<(), Self::Error> {
        self.current.push(Value::from(bool));
        Ok(())
    }

    fn int(&mut self, int: i64) -> Result<(), Self::Error> {
        self.current.push(Value::from(int));
        Ok(())
    }

    fn float(&mut self, float: f64) -> Result<(), Self::Error> {
        self.current.push(Value::from(float));
        Ok(())
    }
}

use crate::{token::Token, value::Value, Result};
use logos::Span;

pub fn parse_simple_value(token: Token<'_>) -> Result<Value<'_>> {
    match token {
        Token::Bool(b) => Ok(Value::Bool(b)),
        Token::Null => Ok(Value::Null),
        Token::Float(n) => Ok(Value::Float(n)),
        Token::Integer(n) => Ok(Value::Integer(n)),
        Token::String(s) => Ok(Value::String(s)),
        Token::Any(s) => Ok(Value::String(s)),
        _ => Err((
            "unexpected token when expecting simple value".to_owned(),
            Span::default(),
        )),
    }
}

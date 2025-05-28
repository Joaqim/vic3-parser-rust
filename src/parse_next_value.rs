use logos::Lexer;

use crate::{token::Token, value::Value, Result};

use crate::parse_array::parse_array;

/// Parse a token stream into a value.
pub fn parse_next_value<'source>(
    lexer: &mut Lexer<'source, Token<'source>>,
) -> Result<Value<'source>> {
    match lexer.next() {
        Some(Ok(Token::String(s))) => Ok(Value::String(s)),
        Some(Ok(Token::Float(n))) => Ok(Value::Float(n)),
        Some(Ok(Token::Integer(n))) => Ok(Value::Integer(n)),
        Some(Ok(Token::Any(s))) => Ok(Value::String(s)),
        Some(Ok(Token::BraceOpen)) => parse_array(lexer),
        Some(Ok(Token::BraceClose)) => {
            // This handles the edge case in your original code
            Err((
                "unexpected '}' when expecting value".to_owned(),
                lexer.span(),
            ))
        }
        _ => Err(("expected value".to_owned(), lexer.span())),
    }
}

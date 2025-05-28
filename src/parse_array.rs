use logos::{Lexer, Span};

use crate::parse_next_value::parse_next_value;
use crate::parse_simple_value::parse_simple_value;
use crate::{token::Token, value::Value, Result};

use std::collections::HashMap;

pub fn parse_array<'source>(lexer: &mut Lexer<'source, Token<'source>>) -> Result<Value<'source>> {
    let span = lexer.span();
    let mut array = Vec::new();

    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::BraceClose) => {
                return if array.iter().any(|value| matches!(value, Value::Object(_))) {
                    flatten_array(array)
                } else {
                    Ok(Value::Array(array))
                };
            }

            Ok(Token::BraceOpen) => {
                let sub_array = parse_array(lexer)?;
                array.push(sub_array);
            }

            Ok(Token::Any(s)) => {
                let value = parse_any_token_in_array(lexer, s)?;
                match value {
                    ArrayParseResult::Single(v) => array.push(v),
                    ArrayParseResult::Multiple(mut values) => array.append(&mut values),
                    ArrayParseResult::EndArray(v) => {
                        array.push(v);
                        return Ok(Value::Array(array));
                    }
                }
            }

            Ok(token) => {
                array.push(parse_simple_value(token)?);
            }

            _ => return Err(("failed to parse token in array ".to_owned(), lexer.span())),
        }
    }

    Err(("unmatched opening bracket".to_owned(), span))
}

enum ArrayParseResult<'source> {
    Single(Value<'source>),
    Multiple(Vec<Value<'source>>),
    EndArray(Value<'source>),
}

fn parse_any_token_in_array<'source>(
    lexer: &mut Lexer<'source, Token<'source>>,
    token_value: &'source str,
) -> Result<ArrayParseResult<'source>> {
    match lexer.next() {
        Some(Ok(Token::EqualSign)) => {
            let value = parse_next_value(lexer)?;
            Ok(ArrayParseResult::Single(Value::Object(HashMap::from([(
                token_value,
                value,
            )]))))
        }

        Some(Ok(Token::BraceClose)) => Ok(ArrayParseResult::EndArray(Value::String(token_value))),

        Some(Ok(Token::String(s))) => Ok(ArrayParseResult::Multiple(vec![
            Value::String(token_value),
            Value::String(s),
        ])),

        Some(Ok(Token::Any(s))) => Ok(ArrayParseResult::Multiple(vec![
            Value::String(token_value),
            Value::String(s),
        ])),

        Some(Ok(Token::Number(n))) => Ok(ArrayParseResult::Multiple(vec![
            Value::String(token_value),
            Value::Number(n),
        ])),

        _ => Err((
            "unexpected token after identifier in array".to_owned(),
            lexer.span(),
        )),
    }
}
/// Flatten an array of objects into a single object
fn flatten_array(array: Vec<Value>) -> Result<Value> {
    let mut result = HashMap::with_capacity(array.len());
    for item in array {
        match item {
            Value::Object(object) => {
                result.extend(object);
            }
            _ => {
                return Err((
                    "array containing object is mixed with non-object value".to_owned(),
                    Span::default(),
                ));
            }
        }
    }
    Ok(Value::Object(result))
}

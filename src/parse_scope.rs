use logos::Lexer;

use crate::parse_next_value::parse_next_value;
use crate::parse_simple_value::parse_simple_value;
use crate::{token::Token, value::Value, Result};

use std::collections::HashMap;

fn expect_token<'source>(
    lexer: &mut Lexer<'source, Token<'source>>,
    expected: Token<'source>,
    error_message: &str,
) -> Result<Value<'source>> {
    match lexer.next() {
        Some(Ok(token)) if std::mem::discriminant(&token) == std::mem::discriminant(&expected) => {
            parse_simple_value(token).or(Ok(Value::Null))
        }
        _ => Err((error_message.to_owned(), lexer.span())),
    }
}

pub fn parse_scope<'source>(lexer: &mut Lexer<'source, Token<'source>>) -> Result<Value<'source>> {
    let scope_name_token = expect_token(lexer, Token::Any(""), "expected scope name")?;
    let scope_name = match scope_name_token {
        Value::String(name) => name,
        _ => unreachable!("scope_name_token should always be a Value::String"),
    };

    expect_token(lexer, Token::EqualSign, "expected '=' after scope name")?;
    expect_token(lexer, Token::BraceOpen, "expected '{' after '='")?;

    let object_contents = parse_object_contents(lexer)?;

    Ok(Value::Object(HashMap::from([(
        scope_name,
        Value::Object(object_contents),
    )])))
}

fn parse_object_contents<'source>(
    lexer: &mut Lexer<'source, Token<'source>>,
) -> Result<HashMap<&'source str, Value<'source>>> {
    let mut map = HashMap::new();
    let mut current_key: Option<&str> = None;

    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::BraceClose) => {
                if current_key.is_some() {
                    return Err((
                        "unexpected '}', expected value after key".to_owned(),
                        lexer.span(),
                    ));
                }
                return Ok(map);
            }

            Ok(Token::Any(key)) if current_key.is_none() => {
                current_key = Some(key);
            }

            Ok(Token::EqualSign) if current_key.is_some() => {
                let key = current_key.take().unwrap();
                let value = parse_next_value(lexer)?;
                map.insert(key, value);
            }
            _ => {
                return Err((
                    format!(
                        "unexpected token in object context, current_key: {:?}",
                        current_key
                    ),
                    lexer.span(),
                ));
            }
        }
    }

    Err(("unmatched opening brace".to_owned(), lexer.span()))
}

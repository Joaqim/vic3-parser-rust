use logos::{Lexer, Logos};

use crate::parse_next_value::parse_next_value;

use crate::{token::Token, value::Value, Result};

use std::collections::HashMap;

pub fn parse_program(source: &String) -> Result<Value> {
    let mut lexer = Token::lexer(source.as_str());

    let program = parse_object_contents(&mut lexer)?;

    Ok(program)
}

fn parse_object_contents<'source>(
    lexer: &mut Lexer<'source, Token<'source>>,
) -> Result<Value<'source>> {
    let mut map = HashMap::new();
    let mut current_key: Option<&str> = None;

    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::BraceClose) if current_key.is_some() => {
                return Err((
                    "unexpected '}', expected value after key".to_owned(),
                    lexer.span(),
                ));
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

    Ok(Value::Object(map))
}

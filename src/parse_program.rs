use logos::{Lexer, Logos};

use crate::parse_next_value::parse_next_value;

use crate::{token::Token, value::Value, Result};

pub fn parse_program(source: &str) -> Result<Value> {
    let mut lexer = Token::lexer(source);

    let program = parse_object_contents(&mut lexer)?;

    Ok(program)
}

fn parse_object_contents<'source>(
    lexer: &mut Lexer<'source, Token<'source>>,
) -> Result<Value<'source>> {
    let mut variables: Vec<(&str, Value)> = Vec::new();
    let mut current_key: Option<&str> = None;

    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::BraceClose) => {
                if current_key.is_some() {
                    return Err((
                        "unexpected '}', expected '=' followed by value after key".to_owned(),
                        lexer.span(),
                    ));
                }
                return Ok(Value::Object(variables));
            }

            Ok(Token::Any(key)) if current_key.is_none() => {
                current_key = Some(key);
            }

            Ok(Token::EqualSign) if current_key.is_some() => {
                let key = current_key.take().unwrap();
                let value = parse_next_value(lexer)?;
                variables.push((key, value));
            }
            Ok(Token::Comment1) => (),
            Err(_) => todo!(),
            _ => {
                return Err((
                    format!(
                        "unexpected token '{:?}' in object context, current_key: {:?}",
                        token.unwrap(),
                        current_key
                    ),
                    lexer.span(),
                ));
            }
        }
    }

    Ok(Value::Object(variables))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Token;

    #[test]
    fn test_parse_empty_object() {
        let mut lexer = Token::lexer("}");
        let result = parse_object_contents(&mut lexer);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Object(_)));
    }

    #[test]
    fn test_parse_single_key_value_pair() {
        let mut lexer = Token::lexer("key = value }");
        let result = parse_object_contents(&mut lexer);
        assert!(result.is_ok(), "{}", result.unwrap_err().0);
        // TODO Assert object keys and values
    }

    #[test]
    fn test_parse_multiple_key_value_pairs() {
        let mut lexer = Token::lexer("key1 = value1 key2 = value2 }");
        let result = parse_object_contents(&mut lexer);
        assert!(result.is_ok(), "{}", result.unwrap_err().0);
        // TODO: Assert object keys and values
    }

    #[test]
    fn test_parse_key_without_value() {
        let mut lexer = Token::lexer("key }");
        let result = parse_object_contents(&mut lexer);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.0,
            "unexpected '}', expected '=' followed by value after key"
        );
    }

    #[test]
    fn test_parse_unexpected_token() {
        let mut lexer = Token::lexer("123 }");
        let result = parse_object_contents(&mut lexer);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.0,
            "unexpected token 'Integer(123)' in object context, current_key: None"
        );
    }

    #[test]
    fn test_parse_missing_value_after_key() {
        let mut lexer = Token::lexer(" key = }");
        let result = parse_object_contents(&mut lexer);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.0, "unexpected '}' when expecting value");
    }
}

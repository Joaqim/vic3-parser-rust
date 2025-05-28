use logos::{Lexer, Span};

use crate::parse_next_value::parse_next_value;
use crate::parse_simple_value::parse_simple_value;
use crate::{token::Token, value::Value, Result};

use std::collections::HashMap;

/// Parse a token stream into a Value::Array.
///
/// Will flatten array and return a Value::Object if any element in the array is an object.
/// Will fail if array contains both simple values and objects
///
///
/// # Errors
///
/// Will return an error if the token stream is exhausted before the array is closed.
/// Will return an error if any value fails to be parsed
pub fn parse_array<'source>(lexer: &mut Lexer<'source, Token<'source>>) -> Result<Value<'source>> {
    let span = lexer.span();
    let mut array = Vec::new();

    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::BraceClose) => {
                // Turns array into Value::Object if any element is an object
                // Will fail if array also contains non-objects
                if array.iter().any(|value| matches!(value, Value::Object(_))) {
                    return flatten_array(array);
                } else {
                    return Ok(Value::Array(array));
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

        Some(Ok(token)) => Ok(ArrayParseResult::Multiple(vec![
            Value::String(token_value),
            parse_simple_value(token)?,
        ])),

        _ => Err((
            "unexpected token after identifier in array".to_owned(),
            lexer.span(),
        )),
    }
}
/// Flatten an array of objects into a single object
/// Returns an error if the array contains any non-object value
fn flatten_array(array: Vec<Value>) -> Result<Value> {
    let result: HashMap<&str, Value> =
        array
            .into_iter()
            .try_fold(HashMap::new(), |mut acc, item| {
                if let Value::Object(object) = item {
                    acc.extend(object);
                    Ok(acc)
                } else {
                    Err((
                        "array containing object is mixed with non-object value".to_owned(),
                        Span::default(),
                    ))
                }
            })?;
    Ok(Value::Object(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;

    #[test]
    fn test_parse_empty_array() {
        let mut lexer = Token::lexer("}");
        let result = parse_array(&mut lexer);
        assert!(result.is_ok());
        let array = match result.unwrap() {
            Value::Array(arr) => arr,
            _ => panic!(),
        };
        assert!(array.is_empty());
    }

    #[test]
    fn test_parse_single_element_array() {
        let mut lexer = Token::lexer("hello }");
        let result = parse_array(&mut lexer);
        assert!(result.is_ok());
        let array = match result.unwrap() {
            Value::Array(arr) => arr,
            _ => panic!(),
        };
        assert_eq!(array.len(), 1);
        assert!(matches!(array[0], Value::String(_)));
    }

    #[test]
    fn test_parse_multiple_element_array() {
        let mut lexer = Token::lexer("hello world }");
        let result = parse_array(&mut lexer);
        assert!(result.is_ok());
        let array = match result.unwrap() {
            Value::Array(arr) => arr,
            _ => panic!(),
        };
        assert_eq!(array.len(), 2);
        assert!(matches!(array[0], Value::String(_)));
        assert!(matches!(array[1], Value::String(_)));
    }

    #[test]
    fn test_parse_mixed_simple_values_array() {
        let mut lexer = Token::lexer(" hello world 1 2 3.4 \"test\" }");
        let result = parse_array(&mut lexer);
        assert!(result.is_ok());
        let array = match result.unwrap() {
            Value::Array(arr) => arr,
            _ => panic!(),
        };
        assert_eq!(array.len(), 6);
        assert!(matches!(array[0], Value::String("hello")));
        assert!(matches!(array[1], Value::String("world")));
        assert!(matches!(array[2], Value::Integer(1)));
        assert!(matches!(array[3], Value::Integer(2)));
        assert!(matches!(array[4], Value::Float(3.4)));
        assert!(matches!(array[5], Value::String("test")));
    }

    #[test]
    fn test_flatten_array_empty() {
        let array = Vec::new();
        let result = flatten_array(array);
        assert!(result.is_ok());
        let object = match result.unwrap() {
            Value::Object(obj) => obj,
            _ => panic!(),
        };
        assert!(object.is_empty());
    }

    #[test]
    fn test_flatten_array_single_object() {
        let mut object = HashMap::new();
        object.insert("key", Value::String("value"));
        let array = vec![Value::Object(object)];
        let result = flatten_array(array);
        assert!(result.is_ok());
        let object = match result.unwrap() {
            Value::Object(obj) => obj,
            _ => panic!(),
        };
        assert_eq!(object.len(), 1);
        assert_eq!(object.get("key").unwrap(), &Value::String("value"));
    }

    #[test]
    fn test_flatten_array_multiple_objects() {
        let mut object1 = HashMap::new();
        object1.insert("key1", Value::String("value1"));
        let mut object2 = HashMap::new();
        object2.insert("key2", Value::String("value2"));
        let array = vec![Value::Object(object1), Value::Object(object2)];
        let result = flatten_array(array);
        assert!(result.is_ok());
        let object = match result.unwrap() {
            Value::Object(obj) => obj,
            _ => panic!(),
        };
        assert_eq!(object.len(), 2);
        assert_eq!(object.get("key1").unwrap(), &Value::String("value1"));
        assert_eq!(object.get("key2").unwrap(), &Value::String("value2"));
    }

    #[test]
    fn test_flatten_array_mixed_values() {
        let mut object = HashMap::new();
        object.insert("key", Value::String("value"));
        let array = vec![Value::Object(object), Value::String("non-object")];
        let result = flatten_array(array);
        assert!(result.is_err());
    }

    #[test]
    fn test_flatten_array_non_object_values() {
        let array = vec![Value::String("non-object"), Value::Integer(42)];
        let result = flatten_array(array);
        assert!(result.is_err());
    }
}

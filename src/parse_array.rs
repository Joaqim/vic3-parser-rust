use logos::{Lexer, Span};
use ordered_hash_map::OrderedHashMap;

use crate::parse_next_value::parse_next_value;
use crate::parse_simple_value::parse_simple_value;
use crate::{token::Token, value::Value, Result};

/// Parse a token stream into a `Value::Array`, `Value::Object` or `Value::Empty`.
///
/// Flattens the array into a `Value::Object` if any element is an object.
/// Fails if the array contains both simple values and objects.
///
/// # Errors
///
/// Returns an error if the token stream is exhausted before the array is closed.
/// Returns an error if any value fails to be parsed.
pub fn parse_array<'source>(lexer: &mut Lexer<'source, Token<'source>>) -> Result<Value<'source>> {
    let initial_span = lexer.span();
    let mut elements = Vec::new();

    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::BraceClose) => {
                if elements.iter().any(|elem| matches!(elem, Value::Object(_))) {
                    return flatten_array(elements);
                }
                if elements.is_empty() {
                    return Ok(Value::Empty);
                }
                return Ok(Value::Array(elements));
            }
            Ok(Token::BraceOpen) => {
                let nested_array = parse_array(lexer)?;
                elements.push(nested_array);
            }
            Ok(Token::Any(identifier)) => {
                let array_parse_result = parse_any_token_in_array(lexer, identifier)?;
                match array_parse_result {
                    ArrayParseResult::Single(value) => elements.push(value),
                    ArrayParseResult::Multiple(mut values) => elements.append(&mut values),
                    ArrayParseResult::EndArray(value) => {
                        elements.push(value);
                        return Ok(Value::Array(elements));
                    }
                }
            }
            Ok(token) => {
                elements.push(parse_simple_value(token)?);
            }
            _ => return Err(("Failed to parse token in array".to_owned(), lexer.span())),
        }
    }

    Err(("Unmatched opening bracket".to_owned(), initial_span))
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
            Ok(ArrayParseResult::Single(Value::Object(
                OrderedHashMap::from_iter(vec![(token_value, value)]),
            )))
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
fn flatten_array(objects: Vec<Value>) -> Result<Value> {
    if objects.is_empty() {
        return Ok(Value::Empty);
    }

    let mut ordered_map: OrderedHashMap<&str, Value<'_>> = OrderedHashMap::new();

    for object in objects {
        match object {
            Value::Object(obj) => ordered_map.extend(obj),
            _ => {
                return Err((
                    "array containing object is mixed with non-object value".into(),
                    Span::default(),
                ))
            }
        }
    }
    Ok(Value::Object(ordered_map))
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

        matches!(result.unwrap(), Value::Empty);
    }

    #[test]
    fn test_flatten_empty_array() {
        let array = Vec::new();
        let result = flatten_array(array);
        assert!(result.is_ok());
        matches!(result.unwrap(), Value::Empty);
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
    fn test_flatten_array_single_object() {
        let mut ordered_map = OrderedHashMap::new();
        ordered_map.insert("key", Value::String("value"));
        let object = vec![Value::Object(ordered_map)];
        let result = flatten_array(object);
        assert!(result.is_ok());
        let object = match result.unwrap() {
            Value::Object(obj) => obj,
            _ => panic!(),
        };
        assert_eq!(object.len(), 1);
        assert_eq!(
            object,
            OrderedHashMap::from_iter(vec![("key", Value::String("value"))].into_iter())
        );
    }

    #[test]
    fn test_flatten_array_multiple_objects() {
        let mut ordered_map1 = OrderedHashMap::new();
        ordered_map1.insert("key1", Value::String("value1"));
        let mut ordered_map2 = OrderedHashMap::new();
        ordered_map2.insert("key2", Value::String("value2"));
        let array = vec![Value::Object(ordered_map1), Value::Object(ordered_map2)];
        let result = flatten_array(array);
        assert!(result.is_ok());
        let object = match result.unwrap() {
            Value::Object(obj) => obj,
            _ => panic!(),
        };
        assert_eq!(object.len(), 2);
        assert_eq!(
            object,
            OrderedHashMap::from_iter(
                vec![
                    ("key1", Value::String("value1")),
                    ("key2", Value::String("value2"))
                ]
                .into_iter()
            )
        );
    }

    #[test]
    fn test_flatten_array_non_object_values() {
        let mut array = vec![Value::String("non-object"), Value::Integer(42)];
        let result = flatten_array(array.clone());
        assert!(result.is_err());
        array.push(Value::Object(OrderedHashMap::from_iter(vec![(
            "key",
            Value::String("value"),
        )])));
        let result = flatten_array(array);
        assert!(result.is_err());

        let array = vec![Value::Object(OrderedHashMap::from_iter(
            vec![("key", Value::String("value"))].into_iter(),
        ))];
        let result = flatten_array(array);
        assert!(result.is_ok());
    }
}

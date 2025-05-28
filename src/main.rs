use clap::Parser;

use logos::{Lexer, Logos, Span};

use std::collections::HashMap;
use std::fs;

type Error = (String, Span);

type Result<T> = std::result::Result<T, Error>;

/* ANCHOR: tokens */
#[derive(Debug, Logos, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")]
enum Token<'source> {
    #[token("false", |_| false, priority = 1)]
    #[token("true", |_| true, priority = 1)]
    Bool(bool),

    #[token("{", priority = 1)]
    BraceOpen,

    #[token("}", priority = 1)]
    BraceClose,

    #[token("=", priority = 1)]
    EqualSign,

    #[token("null", priority = 1)]
    Null,

    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?", |lex| lex.slice().parse::<f64>().unwrap(), priority = 3)]
    Number(f64),

    #[regex(r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#, |lex| lex.slice().trim_matches('"'), priority = 3)]
    String(&'source str),

    #[regex(r"[A-Za-z0-9_:]+", |lex| lex.slice(), priority = 0)]
    Any(&'source str),

    #[logos(skip)]
    #[regex(r"[#|(//)].*\n", priority = 1)]
    Comment1,
    #[logos(skip)]
    #[regex(r"\/\*[^\*\/]*", priority = 2)]
    Comment2,
}
/* ANCHOR_END: tokens */

/* ANCHOR: values */
#[derive(Debug, Clone)]
enum Value<'source> {
    /// null.
    Null,
    /// true or false.
    Bool(bool),
    /// Any floating point number.
    Number(f64),
    /// Any quoted string.
    String(&'source str),
    /// An array of values
    Array(Vec<Value<'source>>),
    /// An dictionary mapping keys and values.
    Object(HashMap<&'source str, Value<'source>>),
}

impl serde::Serialize for Value<'_> {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::String(s) => serializer.serialize_str(s),
            Value::Number(n) => serializer.serialize_f64(*n),
            Value::Array(arr) => arr.serialize(serializer),
            Value::Object(obj) => obj.serialize(serializer),
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::Null => serializer.serialize_none(),
        }
    }
}

/* ANCHOR_END: values */

/* ANCHOR: value */
/// Parse a token stream into a value.
fn parse_next_value<'source>(lexer: &mut Lexer<'source, Token<'source>>) -> Result<Value<'source>> {
    match lexer.next() {
        Some(Ok(Token::String(s))) => Ok(Value::String(s)),
        Some(Ok(Token::Number(n))) => Ok(Value::Number(n)),
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
/* ANCHOR_END: value */
/* ANCHOR: scope */
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

fn parse_scope<'source>(lexer: &mut Lexer<'source, Token<'source>>) -> Result<Value<'source>> {
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

/* ANCHOR_END: scope */
/* ANCHOR: array */
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

fn parse_array<'source>(lexer: &mut Lexer<'source, Token<'source>>) -> Result<Value<'source>> {
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

fn parse_simple_value<'source>(token: Token<'source>) -> Result<Value<'source>> {
    match token {
        Token::Bool(b) => Ok(Value::Bool(b)),
        Token::Null => Ok(Value::Null),
        Token::Number(n) => Ok(Value::Number(n)),
        Token::String(s) => Ok(Value::String(s)),
        Token::Any(s) => Ok(Value::String(s)),
        _ => Err((
            "unexpected token when expecting simple value".to_owned(),
            Span::default(),
        )),
    }
}
/* ANCHOR_END: array */

#[derive(Parser, Debug)]
#[clap(author = "Joaqim Planstedt", version, about)]
/// Application configuration
struct Args {
    #[arg(short = 'a')]
    ast: bool,

    /// file to parse
    #[arg()]
    file: Option<String>,
}
fn main() {
    let args = Args::parse();
    let filename = args.file.expect("Expected file argument");
    let src = fs::read_to_string(&filename).expect("Failed to read file");

    let mut lexer = Token::lexer(src.as_str());

    match parse_scope(&mut lexer) {
        Ok(value) => {
            if args.ast {
                println!("{:#?}", value);
            } else {
                println!("{}", serde_json::to_string_pretty(&value).unwrap());
            }
        }
        Err((msg, span)) => {
            use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};

            let mut colors = ColorGenerator::new();

            let a = colors.next();

            Report::build(ReportKind::Error, (&filename, 12..12))
                .with_message("Invalid JSON".to_string())
                .with_label(
                    Label::new((&filename, span))
                        .with_message(msg)
                        .with_color(a),
                )
                .finish()
                .eprint((&filename, Source::from(src)))
                .unwrap();
        }
    }
}

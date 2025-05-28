use clap::Parser;
use logos::Logos;
use std::fs;

pub type Error = (String, logos::Span);

pub type Result<T> = std::result::Result<T, Error>;

mod token;
mod value;

mod parse_array;
mod parse_next_value;
mod parse_scope;
mod parse_simple_value;

use crate::parse_scope::parse_scope;
use crate::token::Token;

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

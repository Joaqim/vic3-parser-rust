use clap::Parser;
use std::fs;

pub type Error = (String, logos::Span);

pub type Result<T> = std::result::Result<T, Error>;

mod token;
mod value;

mod parse_array;
mod parse_next_value;
mod parse_program;
mod parse_simple_value;
mod parse_variables;

use parse_program::parse_program;

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

    match parse_program(&src) {
        Ok(value) => {
            if args.ast {
                println!("{:#?}", value);
            } else {
                match serde_json::to_string_pretty(&value) {
                    Ok(val) => println!("{}", val),
                    Err(err) => {
                        eprintln!("Failed to serialize: {}", err);
                    }
                }
            }
        }
        Err((msg, span)) => {
            use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};

            let mut colors = ColorGenerator::new();

            let a = colors.next();

            Report::build(ReportKind::Error, (&filename, 12..12))
                .with_message("Failed to parse Input".to_string())
                .with_label(
                    Label::new((&filename, span))
                        .with_message(msg)
                        .with_color(a),
                )
                .finish()
                .eprint((&filename, Source::from(&src)))
                .unwrap();
        }
    };
}

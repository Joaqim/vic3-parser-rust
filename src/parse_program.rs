use logos::Logos;

use crate::parse_variables::parse_variables;

use crate::{token::Token, value::Value, Result};

pub fn parse_program<'source>(source: &'source str) -> Result<Value<'source>> {
    let mut lexer = Token::<'source>::lexer(source);

    parse_variables(&mut lexer)
}

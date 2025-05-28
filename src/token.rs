use logos::Logos;

/* ANCHOR: tokens */
#[derive(Debug, Logos, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")]
pub enum Token<'source> {
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

    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)(?:[eE][+-]?\d+)?", |lex| lex.slice().parse::<f64>().unwrap(), priority = 3)]
    Float(f64),

    #[regex(r"-?(?:0|[0-9]\d*)", |lex| lex.slice().parse::<i64>().unwrap(), priority = 2)]
    Integer(i64),

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

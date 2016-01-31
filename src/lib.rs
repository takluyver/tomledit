extern crate toml;

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Punctutation,
    Whitespace,
    Comment,
    BareKey,
    BasicString,
    LiteralString,
    MultilineBasicString,
    MultilineLiteralString,
    Integer,
    Float,
    Boolean,
    Datetime,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    kind: TokenType,
    text: String,
}

pub fn read_whitespace(s: &str) -> (Token, &str) {
    let mut ends_at = s.len();
    for (i, c) in s.char_indices() {
        match c {
            ' '|'\t' => (),
            _ => {
                ends_at = i;
                break;
            }
        }
    }
    let (tok, remainder) = s.split_at(ends_at);
    return (Token{kind: TokenType::Whitespace, text:String::from(tok)}, remainder)
}

pub fn read_comment(s: &str) -> (Token, &str) {
    let mut ends_at = s.len();
    let mut tok = String::new();
    for (i, c) in s.char_indices() {
        match c {
            '\n'|'\r' => {
                ends_at = i;
                break;
            }
            _ => tok.push(c),
        }
    }
    let (tok, remainder) = s.split_at(ends_at);
    return (Token{kind: TokenType::Comment, text:String::from(tok)}, remainder)
}

pub fn tokenise(s: &str) {
    let mut tokens = Vec::new();
    let mut remainder = s;
    loop {
        let next_char = remainder.chars().next();
        let (next_token, rem) = match next_char {
            None => {break;},
            Some(c) => {match c {
                ' '|'\t' => read_whitespace(s),
                '#' => read_comment(s),
                _ => panic!("Unexpected char")
            }}
        };
        tokens.push(next_token);
        remainder = rem;
    }
}

#[test]
fn it_works() {
    use toml::Value;
    let tv = Value::String("Hello".to_string());
    assert_eq!(tv.to_string(), "\"Hello\"");
    let tv2 = Value::Integer(123);
    assert_eq!(tv2.to_string(), "123");
}

#[test]
fn test_read_whitespace() {
    let res = read_whitespace("  \t b");
    assert_eq!(res, (Token{kind: TokenType::Whitespace, text: String::from("  \t ")}, "b"))
}

#[test]
fn test_read_comment() {
    assert_eq!(read_comment("# This is a comment\nfoo"),
            (Token{kind: TokenType::Comment, text: String::from("# This is a comment")}, "\nfoo"))
}

// #[test]
// fn test_tokenise() {
//     tokenise("abc".chars());
// }

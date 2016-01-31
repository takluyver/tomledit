extern crate toml;

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Punctutation,
    Whitespace,
    Newline,
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

macro_rules! chars_while {
    ($s:expr, $($pattern:pat),+ ) => {{
        let mut ends_at = $s.len();
        for (i, c) in $s.char_indices() {
            match c {
                $($pattern)|+ => (),
                _ => {
                    ends_at = i;
                    break;
                }
            }
        }
        $s.split_at(ends_at)
    }}
}

macro_rules! chars_until {
    ($s:expr, $($pattern:pat),+ ) => {{
        let mut ends_at = $s.len();
        for (i, c) in $s.char_indices() {
            match c {
                $($pattern)|+ => {
                    ends_at = i;
                    break;
                },
                _ => ()
            }
        }
        $s.split_at(ends_at)
    }}
}

fn read_whitespace(s: &str) -> (Token, &str) {
    let (tok, remainder) = chars_while!(s, ' ', '\t');
    return (Token{kind: TokenType::Whitespace, text:String::from(tok)}, remainder)
}

fn read_newline(s: &str) -> (Token, &str) {
    let (tok, remainder) = chars_while!(s, '\n', '\r');
    return (Token{kind: TokenType::Newline, text:String::from(tok)}, remainder)
}

fn read_comment(s: &str) -> (Token, &str) {
    let (tok, remainder) = chars_until!(s, '\n', '\r');
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
                ' '|'\t' => read_whitespace(remainder),
                '\n'|'\r' => read_newline(remainder),
                '#' => read_comment(remainder),
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
fn test_read_newline() {
    let res = read_newline("\n\r\na");
    assert_eq!(res, (Token{kind: TokenType::Newline, text: String::from("\n\r\n")}, "a"))
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

extern crate toml;

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Punctuation,
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
    (Token{kind: TokenType::Whitespace, text:String::from(tok)}, remainder)
}

fn read_newline(s: &str) -> (Token, &str) {
    let (tok, remainder) = chars_while!(s, '\n', '\r');
    (Token{kind: TokenType::Newline, text:String::from(tok)}, remainder)
}

fn read_comment(s: &str) -> (Token, &str) {
    let (tok, remainder) = chars_until!(s, '\n', '\r');
    (Token{kind: TokenType::Comment, text:String::from(tok)}, remainder)
}

fn read_punctuation(s: &str) -> (Token, &str) {
    // Punctuation is always 1 character (and 1 byte in UTF-8)
    (Token{kind: TokenType::Punctuation, text:String::from(&s[..1])}, &s[1..])
}

fn read_number_or_datetime(s: &str) -> (Token, &str) {
    let (tok,  remainder) = chars_until!(s, ' ', '\t', '\n', '\r');
    let kind  = if tok.contains('e') || tok.contains('E') {
        TokenType::Float
    } else if tok.contains('-') && !tok.starts_with('-') {
        TokenType::Datetime
    } else if tok.contains('.') {
        TokenType::Float
    } else {
        TokenType::Integer
    };
    (Token{kind: kind, text:String::from(tok)}, remainder)
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
                '['|']'|'.'|','|'=' => read_punctuation(remainder),
                '+'|'-' => read_number_or_datetime(remainder),
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

#[test]
fn test_read_punctuation() {
    assert_eq!(read_punctuation("[foo]"),
            (Token{kind: TokenType::Punctuation, text: String::from("[")}, "foo]"));
}

#[test]
fn test_read_number_or_datetime() {
    assert_eq!(read_number_or_datetime("6.626e-34 "),
            (Token{kind: TokenType::Float, text: String::from("6.626e-34")}, " "));
    assert_eq!(read_number_or_datetime("-12\n"),
            (Token{kind: TokenType::Integer, text: String::from("-12")}, "\n"));
    assert_eq!(read_number_or_datetime("1979-05-27 "),
            (Token{kind: TokenType::Datetime, text: String::from("1979-05-27")}, " "));
}

// #[test]
// fn test_tokenise() {
//     tokenise("abc".chars());
// }

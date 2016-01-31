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
    pub kind: TokenType,
    pub text: String,
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
    let (tok,  remainder) = chars_until!(s, ' ', '\t', '\n', '\r', '#');
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

fn read_boolean(s: &str) -> (Token, &str) {
    let (tok,  remainder) = chars_until!(s, ' ', '\t', '\n', '\r', '#');
    (Token{kind: TokenType::Boolean, text:String::from(tok)}, remainder)
}

fn read_bare_key(s: &str) -> (Token, &str) {
    let (tok, remainder) = chars_while!(s, 'A'...'Z', 'a'...'z', '0'...'9', '_', '-');
    (Token{kind: TokenType::BareKey, text:String::from(tok)}, remainder)
}

fn read_literal_string(s: &str) -> (Token, &str) {
    let mut ends_at = s.len();
    let (offset, kind) = if s.starts_with("'''") {
        (3, TokenType::MultilineLiteralString)
    } else {
        (1, TokenType::LiteralString)
    };
    for (i, c) in s[offset..].char_indices() {
        match c {
            '\'' => {
                if kind == TokenType::MultilineLiteralString {
                    if s[i+offset..].starts_with("'''") {
                        ends_at = i+6; // +6 for 2x triple quotes
                        break;
                    }
                } else {
                    ends_at = i+2; // +2 for 2x single quotes
                    break;
                }
            },
            _ => ()
        }
    }
    let (tok,  remainder) = s.split_at(ends_at);
    (Token{kind: kind, text: String::from(tok)}, remainder)
}

fn read_basic_string(s: &str) -> (Token, &str) {
    let mut ends_at = s.len();
    let (offset, kind) = if s.starts_with("\"\"\"") {
        (3, TokenType::MultilineBasicString)
    } else {
        (1, TokenType::BasicString)
    };
    let mut escape = false;
    for (i, c) in s[offset..].char_indices() {
        match c {
            '\\' => escape = !escape,
            '"' => {
                if escape {
                    escape = false;
                    continue;
                }
                if kind == TokenType::MultilineBasicString {
                    if s[i+offset..].starts_with("\"\"\"") {
                        ends_at = i+6; // +6 for 2x triple quotes
                        break;
                    }
                } else {
                    ends_at = i+2; // +2 for 2x single quotes
                    break;
                }
            },
            _ => escape = false
        }
    }
    let (tok,  remainder) = s.split_at(ends_at);
    (Token{kind: kind, text: String::from(tok)}, remainder)
}

fn key_context(in_rhs: bool, bracket_stack: &Vec<char>, tokens: &Vec<Token>) -> bool {
    if in_rhs {
        if bracket_stack.last() == Some(&'{') {
            for token in tokens.iter().rev() {
                match token.kind {
                    TokenType::Whitespace => continue,
                    TokenType::Punctuation => {
                        if token.text == "," || token.text == "{" {
                            return true
                        } else {
                            return false
                        }
                    },
                    _ => {return false}
                }
            }
            panic!()
        } else {
            false
        }
    } else {
        true
    }
}

pub fn tokenise(s: &str) {
    let mut tokens = Vec::new();
    let mut remainder = s;
    let mut in_rhs = false;
    let mut bracket_stack = Vec::new();
    loop {
        let next_char = remainder.chars().next();
        let (next_token, rem) = match next_char {
            None => {break;},
            Some(c) => {match c {
                ' '|'\t' => read_whitespace(remainder),
                '\n'|'\r' => {
                    if bracket_stack.is_empty() {
                        in_rhs = false;
                    }
                    read_newline(remainder)
                },
                '#' => read_comment(remainder),
                '['|'{' => {
                    bracket_stack.push(c);
                    read_punctuation(remainder)
                },
                ']'|'}' => {
                    bracket_stack.pop().unwrap();
                    read_punctuation(remainder)
                },
                '=' => {
                    in_rhs = true;
                    read_punctuation(remainder)
                }
                '.'|',' => {
                    read_punctuation(remainder)
                },
                '+' => read_number_or_datetime(remainder),
                '0'...'9'|'-' => {
                    if key_context(in_rhs, &bracket_stack, &tokens) {
                        read_bare_key(remainder)
                    } else {
                        read_number_or_datetime(remainder)
                    }
                },
                't'|'f' => {
                    if key_context(in_rhs, &bracket_stack, &tokens) {
                        read_bare_key(remainder)
                    } else {
                        read_boolean(remainder)
                    }
                },
                'A'...'Z'|'a'...'z'|'_' => {
                    assert!(key_context(in_rhs, &bracket_stack, &tokens));
                    read_bare_key(remainder)
                },
                '\'' => read_literal_string(remainder),
                '"' => read_basic_string(remainder),
                _ => panic!("Unexpected char")

            }}
        };
        tokens.push(next_token);
        remainder = rem;
    }
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

#[test]
fn test_read_bare_key() {
    assert_eq!(read_bare_key("bare-key ="),
            (Token{kind: TokenType::BareKey, text: String::from("bare-key")}, " ="));
    assert_eq!(read_bare_key("1234="),
            (Token{kind: TokenType::BareKey, text: String::from("1234")}, "="));
}

#[test]
fn test_read_literal_string() {
    assert_eq!(read_literal_string("'foo' "),
            (Token{kind: TokenType::LiteralString, text: String::from("'foo'")}, " "));
    assert_eq!(read_literal_string("'''foo'\nbar''' "),
            (Token{kind: TokenType::MultilineLiteralString, text: String::from("'''foo'\nbar'''")}, " "));
}

#[test]
fn test_read_basic_string() {
    assert_eq!(read_basic_string(r#""foo\"\nbar" "#),
            (Token{kind: TokenType::BasicString, text: String::from(r#""foo\"\nbar""#)}, " "));
    assert_eq!(read_basic_string(r#""""foo"\nbar\"""" "#),
            (Token{kind: TokenType::MultilineBasicString, text: String::from(r#""""foo"\nbar\"""""#)}, " "));
}

// #[test]
// fn test_tokenise() {
//     tokenise("abc".chars());
// }

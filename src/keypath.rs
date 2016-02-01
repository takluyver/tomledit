use super::tokenise;
use super::tokenise::{Token, TokenType};

#[derive(Debug,PartialEq,Clone)]
pub enum KeyPath {
    Root,
    Key(Box<KeyPath>, String),
    Ix(Box<KeyPath>, i64),
}

impl KeyPath {
    fn append_key(self, k: String) -> KeyPath {
        KeyPath::Key(Box::new(self.clone()), k)
    }

    fn append_index(self, i: i64) -> KeyPath {
        KeyPath::Ix(Box::new(self.clone()), i)
    }

    pub fn stringify(&self) -> String {
        match *self {
            KeyPath::Root => format!(""),
            KeyPath::Key(ref head, ref tail) => {
                format!("{}.{}", head.stringify(), tail)
            },
            KeyPath::Ix(ref head, tail) => {
                format!("{}[{}]", head.stringify(), tail)
            }
        }
    }

    pub fn from_string(s: &str) -> KeyPath {
        let mut path = KeyPath::Root;
        let mut remainder = s;
        loop {
            let next_char = remainder.chars().next();
            let (next_token, rem) = match next_char {
                None => break,
                Some(c) => {match c {
                    '.' => {
                        tokenise::read_punctuation(remainder)
                    },
                    '[' => {
                        let (t, r) = tokenise::read_number_or_datetime(&remainder[1..]);
                        (t, &r[1..])
                    },
                    'A'...'Z'|'a'...'z'|'0'...'9'|'_'|'-' => {
                        tokenise::read_bare_key(remainder)
                    },
                    _ => panic!("Unexpected char")
                }}
            };
            match next_token {
                Token{kind: TokenType::BareKey, text: s} => {
                    path = path.append_key(s);
                }
                Token{kind: TokenType::Integer, text: s} => {
                    path = path.append_index(s.parse::<i64>().unwrap());
                }
                _ => (),
            };
            remainder = rem;
        }
        path
    }
}


#[test]
fn test_stringify_keypath() {
    let kp = KeyPath::Root.append_key(String::from("foo")).append_index(2);
    assert_eq!(kp.stringify(), String::from(".foo[2]"))
}

#[test]
fn test_keypath_from_string() {
    let s = "1";
    assert_eq!(&s[1..], "");
    let expected = KeyPath::Root.append_key(String::from("foo"))
                                .append_key(String::from("bar"))
                                .append_index(2);
    assert_eq!(KeyPath::from_string("foo.bar[2]"), expected);
}
use super::tokenise;
use super::tokenise::{Token, TokenType};

#[derive(Debug,PartialEq,Eq,Hash,Clone)]
pub enum KeyPathComponent {
    Key(String),
    Ix(usize),
}

#[derive(Debug,PartialEq,Clone,Eq,Hash)]
pub struct KeyPath {
    parts: Vec<KeyPathComponent>
}
// pub enum KeyPath {
//     Root,
//     Key(Box<KeyPath>, String),
//     Ix(Box<KeyPath>, usize),
// }

impl KeyPath {
    pub fn new() -> KeyPath {
        KeyPath{parts: Vec::new()}
    }

    pub fn append_key(self, k: String) -> KeyPath {
        // KeyPath::Key(Box::new(self.clone()), k)
        let mut new = self.clone();
        new.parts.push(KeyPathComponent::Key(k));
        new
    }

    pub fn append_index(self, i: usize) -> KeyPath {
        //KeyPath::Ix(Box::new(self.clone()), i)
        let mut new = self.clone();
        new.parts.push(KeyPathComponent::Ix(i));
        new
    }

    pub fn stringify(&self) -> String {
        // match *self {
        //     KeyPath::Root => format!(""),
        //     KeyPath::Key(ref head, ref tail) => {
        //         format!("{}.{}", head.stringify(), tail)
        //     },
        //     KeyPath::Ix(ref head, tail) => {
        //         format!("{}[{}]", head.stringify(), tail)
        //     }
        // }
        let mut res = String::new();
        for ref p in &self.parts {
            match *p {
                &KeyPathComponent::Key(ref s) => res.push_str(&format!(".{}", s)),
                &KeyPathComponent::Ix(ref i) => res.push_str(&format!("[{}]", i))
            }
        }
        res
    }

    pub fn from_string(s: &str) -> KeyPath {
        let mut path = KeyPath::new();
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
                    path.parts.push(KeyPathComponent::Key(s));
                    // path = path.append_key(s);
                }
                Token{kind: TokenType::Integer, text: s} => {
                    path.parts.push(KeyPathComponent::Ix(s.parse::<usize>().unwrap()));
                    // path = path.append_index(s.parse::<usize>().unwrap());
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
    let kp = KeyPath::new().append_key(String::from("foo")).append_index(2);
    assert_eq!(kp.stringify(), String::from(".foo[2]"))
}

#[test]
fn test_keypath_from_string() {
    let s = "1";
    assert_eq!(&s[1..], "");
    let expected = KeyPath::new().append_key(String::from("foo"))
                                .append_key(String::from("bar"))
                                .append_index(2);
    assert_eq!(KeyPath::from_string("foo.bar[2]"), expected);
}

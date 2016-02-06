extern crate toml;

mod tokenise;
mod keypath;

use std::str::FromStr;
pub use tokenise::{Token, TokenType, tokenise};
pub use keypath::KeyPath;

enum ValueFormat {
    Table,
    InlineTable,
    Array,
}

fn select_significant(tokens: &Vec<Token>, start: usize, n: usize) -> Vec<(&Token, usize)> {
    let mut res = Vec::new();
    let mut pos = start;
    while (pos < tokens.len()) && (res.len() < n) {
        let ref token = tokens[pos];
        match token.kind {
            TokenType::Whitespace | TokenType::Newline | TokenType::Comment => (),
            _ => {
                res.push((token, pos))
            }
        }
        pos += 1;
    }
    res
}

fn key_token_to_string(tok: &Token) -> String {
    match tok.kind {
        TokenType::BareKey => tok.text.clone(),
        TokenType::BasicString | TokenType::LiteralString => {
            let val = toml::Value::from_str(&tok.text).unwrap();
            match val {
                toml::Value::String(s) => s,
                _ => panic!("Unexpected key value {:?}", val)
            }
        },
        _ => panic!("Unexpected key token {:?}", tok)
    }
}

fn is_atomic_tok(tok: &Token) -> bool {
    match tok.kind {
        TokenType::Punctuation => false,
        _ => true
    }
}

struct KeyTokenIter<'a> {
    tokens: &'a Vec<Token>,
    stack: Vec<(KeyPath, ValueFormat, usize)>,
    pos: usize,
}

impl<'a> KeyTokenIter<'a> {
    fn new(tokens: &Vec<Token>) -> KeyTokenIter {
        KeyTokenIter{tokens: tokens, stack: Vec::new(), pos:0}
    }

    fn current_keypath(&self) -> KeyPath {
        match self.stack.last() {
            Some(trip) => trip.0.clone(),
            None => KeyPath::Root
        }
    }
}

impl<'a> Iterator for KeyTokenIter<'a> {
    type Item = (KeyPath, usize, usize);

    fn next(&mut self) -> Option<(KeyPath, usize, usize)> {
        let next_3 = select_significant(self.tokens, self.pos, 3);
        if next_3.len() < 3 {
            return None;
        }
        if (next_3[1].0 == &Token{kind: TokenType::Punctuation, text: String::from("=")}) {
            // Key value pair
            let key = key_token_to_string(next_3[0].0);
            if is_atomic_tok(next_3[2].0) {
                let (_, pos) = next_3[2];
                self.pos = pos + 1;
                return Some((self.current_keypath().append_key(key), pos, pos+1))
            } else if next_3[2].0.text == String::from("[") {
                // Start an array
                let pos = next_3[2].1;
                let keypath = self.current_keypath().append_key(key);
                self.stack.push((keypath, ValueFormat::Array, pos));
                self.pos = pos + 1;
                return self.next();
            } else if next_3[2].0.text == String::from("{") {
                let pos = next_3[2].1;
                let keypath = self.current_keypath().append_key(key);
                self.stack.push((keypath, ValueFormat::InlineTable, pos));
                self.pos = pos + 1;
                return self.next();
            } else {
                panic!("Unexpected token after = : {:?}", next_3[2]);
            }
        } else if next_3[0].0.text == String::from("[") {
            // New table
            let prev_table = self.stack.pop();
            if next_3[1].0.text == String::from("[") {
                // [[foo]] : table in array
            }
            // TODO: there may be >1 token - [a.b.c]
            let key = key_token_to_string(next_3[1].0);
            self.pos = next_3[2].1+1;
            let new_keypath = self.current_keypath().append_key(key);
            self.stack.push((new_keypath, ValueFormat::Table, self.pos));
            return match prev_table {
                Some((key, _, start)) => Some((key, start, next_3[0].1 - 1)),
                None => self.next()
            }
        } else {
            return None
        }
    }
}

#[test]
fn test_keytokeniter() {
    let inp = vec![
        Token{kind: TokenType::BareKey, text: String::from("a")},
        Token{kind: TokenType::Punctuation, text: String::from("=")},
        Token{kind: TokenType::Integer, text: String::from("1")},
        Token{kind: TokenType::Newline, text: String::from("\n")},
        Token{kind: TokenType::BareKey, text: String::from("b")},
        Token{kind: TokenType::Punctuation, text: String::from("=")},
        Token{kind: TokenType::Integer, text: String::from("2")},
    ];
    let mut kti = KeyTokenIter::new(&inp);
    assert_eq!(kti.next(), Some((KeyPath::from_string("a"), 2, 3)));
    assert_eq!(kti.next(), Some((KeyPath::from_string("b"), 6, 7)));
    assert_eq!(kti.next(), None);
}

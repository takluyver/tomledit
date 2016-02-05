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
}

impl<'a> Iterator for KeyTokenIter<'a> {
    type Item = (KeyPath, usize, usize);

    fn next(&mut self) -> Option<(KeyPath, usize, usize)> {
        let next_3 = select_significant(self.tokens, self.pos, 3);
        if (next_3[1].0 == &Token{kind: TokenType::Punctuation, text: String::from("=")}) {
            let key = key_token_to_string(next_3[0].0);
            if is_atomic_tok(next_3[2].0) {
                let (tok, pos) = next_3[2];
                let keypath = match self.stack.last() {
                    Some(trip) => {
                        trip.0.clone()
                    }
                    None => KeyPath::Root
                };
                return Some((keypath.append_key(key), pos, pos+1))
            } else {
                return None
            }
        } else {
            return None
        }
    }
}

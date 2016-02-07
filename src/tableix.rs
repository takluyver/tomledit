extern crate toml;

use std::collections::HashMap;

use super::tokenise::{Token, TokenType, tokenise};
use super::keypath::{KeyPath, KeyPathComponent};

// use std::boxed::Box;

#[derive(Debug,PartialEq)]
pub struct TablePos {
    key: KeyPath,
    start: usize,
    end: usize,
}

fn read_table_name(tokens: &Vec<Token>, pos: usize, table_arrays: &mut HashMap<KeyPath, usize>) -> (KeyPath, usize) {
    let mut my_pos = pos+1;
    let table_in_array = tokens[my_pos] == Token::from("[");
    if table_in_array {
        my_pos += 1;
    };
    let mut s = String::new();
    while my_pos < tokens.len() && tokens[my_pos] != Token::from("]") {
        s.push_str(&tokens[my_pos].text);
        my_pos += 1;
    }
    let name = KeyPath::from_string(&s);
    let key = {
        let mut res = KeyPath::new();
        let (tail, head) = name.parts.split_last().unwrap();
        for part in head {
            res.parts.push(part.clone());
            match table_arrays.get(&res) {
                None => (),
                Some(length) => res.parts.push(KeyPathComponent::Ix(length-1))
            }
        }
        res.parts.push(tail.clone());
        res
    };
    if table_in_array {
        let count = table_arrays.entry(key.clone()).or_insert(0);
        *count += 1;
        (key.append_index(*count-1), my_pos+2)
    } else {
        (key, my_pos+1)
    }
}

pub fn find_tables(tokens: &Vec<Token>) -> Vec<TablePos> {
    let mut res = Vec::new();
    let mut table_arrays = HashMap::new();
    let mut prev_table = (KeyPath::new(), 0);
    let mut prev_token = &Token::from("."); // Just has to be not '='
    let mut array_depth = 0;
    let mut pos = 0;
    while pos < tokens.len()  {
        let ref tok = tokens[pos];
        match tok.kind {
            TokenType::Whitespace | TokenType::Newline | TokenType::Comment => {
                pos += 1;
                continue;
            },
            _ => ()
        }
        if *tok == Token::from("[") {
            if *prev_token == Token::from("=") || array_depth > 0 {
                array_depth += 1;
            } else {
                // New table
                let (new_key, new_start) = read_table_name(&tokens, pos, &mut table_arrays);
                let (prev_key, prev_start) = prev_table;
                res.push(TablePos{key: prev_key, start: prev_start, end: pos});
                prev_table = (new_key, new_start);
                prev_token = &tokens[new_start-1];
                pos = new_start;
            }
        } else {
            if *tok == Token::from("]") && array_depth > 0 {
                array_depth -= 1;
            }
            prev_token = tok;
            pos += 1;
        }
    }
    {
        let (final_key, final_start) = prev_table;
        res.push(TablePos{key: final_key, start: final_start, end: pos})
    }
    res
}

pub fn find_table(tokens: &Vec<Token>, key: &KeyPath) -> Option<TablePos> {
    for candidate in find_tables(tokens) {
        if candidate.key == *key {
            return Some(candidate)
        }
    }
    None
}

fn make_key_token(key: &str) -> Token {
    if key.chars().any(|c| { match c {
        'a'...'z' | 'A'...'Z' | '-' | '_' => false,
        _ => true
    }}) {
        // Key needs quoting
        let val = toml::Value::String(String::from(key));
        Token{kind: TokenType::BasicString, text: val.to_string()}
    } else {
        // Bare key
        Token{kind: TokenType::BareKey, text: String::from(key)}
    }
}

pub fn insert_kv(tokens: &Vec<Token>, key: &KeyPath, value: toml::Value) -> Vec<Token> {
    let table_pos = find_table(tokens, &key.parent().unwrap()).unwrap();
    // Find insertion point
    let mut pos = table_pos.end - 1;
    while pos >= table_pos.start && (tokens[pos].kind == TokenType::Whitespace ||
                                     tokens[pos].kind == TokenType::Newline) {
        pos -= 1;
    }
    pos += 1; // Cut just after the non-whitespace token we found

    let mut res = Vec::new();
    // Copy tokens before insertion point
    for tok in &tokens[..pos] {
        res.push((*tok).clone());
    }
    // Insert new key-value pair
    res.push(Token::from("\n"));
    let key_tail = match key.parts.last().unwrap() {
        &KeyPathComponent::Key(ref s) => s,
        _ => panic!("Key must end with a string part")
    };
    res.push(make_key_token(&key_tail));
    for tok in [" ", "=", " "].iter() {
        res.push(Token::from(tok));
    }
    res.push(Token::from(&value.to_string()));
    // Copy tokens after insertion point
    for tok in &tokens[pos..] {
        res.push((*tok).clone());
    }
    res
}

#[test]
fn test_find_tables() {
    let inp = vec![
        Token::from("a"), Token::from("="), Token::from("1"),
        Token::from("\n"),
        Token::from("["), Token::from("table1"), Token::from("]"),
        Token::from("\n"),
        Token::from("b"), Token::from("="), Token::from("2"),
        Token::from("\n"),
        Token::from("["), Token::from("["), Token::from("arraytable"), Token::from("]"), Token::from("]"),
        Token::from("\n"),
        Token::from("b"), Token::from("="), Token::from("2"),
        Token::from("\n"),
        Token::from("["), Token::from("["), Token::from("arraytable"), Token::from("]"), Token::from("]"),
        Token::from("\n"),
        Token::from("b"), Token::from("="), Token::from("3"),
        Token::from("\n"),
        Token::from("["), Token::from("arraytable.sub"), Token::from("]"),
        Token::from("\n"),
        Token::from("q"), Token::from("="), Token::from("7"),
    ];
    assert_eq!(find_tables(&inp), vec![
        TablePos{key: KeyPath::new(), start:0, end:4},
        TablePos{key: KeyPath::from_string("table1"), start:7, end:12},
        TablePos{key: KeyPath::from_string("arraytable[0]"), start:17, end:22},
        TablePos{key: KeyPath::from_string("arraytable[1]"), start:27, end:32},
        TablePos{key: KeyPath::from_string("arraytable[1].sub"), start:35, end:39},
    ])
}

#[test]
fn test_make_key_token() {
    assert_eq!(make_key_token("a_b"), Token{kind: TokenType::BareKey, text: String::from("a_b")});
    assert_eq!(make_key_token("a b"), Token{kind: TokenType::BasicString, text: String::from(r#""a b""#)});
}

#[test]
fn test_insert_kv() {
let inp = "\
[foo]
# a comment
a = 1 #?

[bar]
b = 2
";
let exp = "\
[foo]
# a comment
a = 1 #?
c = 3

[bar]
b = 2
";
    let tokens = tokenise(inp);
    let mut res = String::new();
    for tok in insert_kv(&tokens, &KeyPath::from_string("foo.c"), toml::Value::Integer(3)) {
        res.push_str(&tok.text);
    }
    assert_eq!(res, exp);
}

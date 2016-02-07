use std::collections::HashMap;

use super::tokenise::{Token, TokenType};
use super::keypath::KeyPath;

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
    if table_in_array {
        let count = table_arrays.entry(name.clone()).or_insert(0);
        *count += 1;
        (name.append_index(*count-1), my_pos+2)
    } else {
        (name, my_pos+1)
    }
}

pub fn find_tables(tokens: &Vec<Token>) -> Vec<TablePos> {
    let mut res = Vec::new();
    let mut table_arrays = HashMap::new();
    let mut prev_table = (KeyPath::Root, 0);
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
    ];
    assert_eq!(find_tables(&inp), vec![
        TablePos{key: KeyPath::Root, start:0, end:4},
        TablePos{key: KeyPath::from_string("table1"), start:7, end:12},
        TablePos{key: KeyPath::from_string("arraytable[0]"), start:17, end:22},
        TablePos{key: KeyPath::from_string("arraytable[1]"), start:27, end:31},
    ])
}

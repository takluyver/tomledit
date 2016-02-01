mod tokenise;
mod keypath;

pub use tokenise::{Token, TokenType, tokenise};
pub use keypath::KeyPath;

enum ValueFormat {
    Table,
    InlineTable,
    Array,
}

fn select_significant(tokens: &Vec<Token>, start: usize, n: usize) -> (Vec<&Token>, usize) {
    let mut res = Vec::new();
    let mut pos = start;
    while (pos < tokens.len()) && (res.len() < n) {
        let token = tokens[pos];
        match token.kind {
            TokenType::Whitespace | TokenType::Newline | TokenType::Comment => (),
            _ => {
                res.push(&token)
            }
        }
    }
    (res, pos)
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
        next_3 = select_significant(self.tokens, self.pos, 3);
    }
}

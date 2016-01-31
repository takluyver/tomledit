extern crate tomledit;

use tomledit::{Token, TokenType, tokenise};

#[test]
fn test_tokenise() {
    let sample = r#"[table]
foo="bar"
12=34
"#;
    let res = tokenise(sample);
    println!("{:?}\n", res);
    assert_eq!(res[0], Token{kind: TokenType::Punctuation, text: String::from("[")});
    assert_eq!(res[1], Token{kind: TokenType::BareKey, text: String::from("table")});
    assert_eq!(res[3].kind, TokenType::Newline);
    assert_eq!(res[5], Token{kind: TokenType::Punctuation, text: String::from("=")});
    assert_eq!(res[6], Token{kind: TokenType::BasicString, text: String::from(r#""bar""#)});
    assert_eq!(res[8], Token{kind: TokenType::BareKey, text: String::from("12")});
    assert_eq!(res[10], Token{kind: TokenType::Integer, text: String::from("34")});
}

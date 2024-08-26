use std::iter::Peekable;
use std::str::Chars;
#[derive(Debug, PartialEq)]
pub enum Token {
    Word(String),
    Punctuation(char),
    Number(String),
}

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}
impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            chars: input.chars().peekable(),
        }
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        let next_char = self.chars.next()?;

        match next_char {
            c if c.is_alphabetic() => {
                let mut word = String::new();
                word.push(c);
                while let Some(&c) = self.chars.peek() {
                    if c.is_alphabetic() || c == '\'' {
                        word.push(self.chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                Some(Token::Word(word.to_lowercase()))
            }
            c if c.is_numeric() => {
                let mut number = String::new();
                number.push(c);
                while let Some(&c) = self.chars.peek() {
                    if c.is_alphabetic() || c == '.' {
                        number.push(self.chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                Some(Token::Number(number))
            }
            c if c.is_whitespace() => None,
            c => Some(Token::Punctuation(c)),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }
}
impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

pub fn tokenize(text: &str) -> Vec<String> {
    Lexer::new(text)
        .map(|token| match token {
            Token::Word(word) => word,
            Token::Number(num) => num,
            Token::Punctuation(p) => p.to_string(),
        })
        .collect()
}

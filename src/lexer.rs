#[derive(Debug, thiserror::Error, PartialEq)]
pub enum LexError {
    #[error("expected string termination for string at position {pos}")]
    UnterminatedString { pos: usize },

    #[error("unexpected dot number at position {pos}")]
    UnexpectedDot { pos: usize },
}

#[derive(Debug, PartialEq)]
pub enum NumberType {
    Integer,
    Float,
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Identifier,
    StringLiteral,
    RawString,
    Number(NumberType),
    Bar, // |
    Newline,

    DollarSign, // $

    OpenBrace,  // {
    CloseBrace, // }

    OpenParenthesis,  // (
    CloseParenthesis, // )

    Equal, // =
    Bang,  // !

    // Comparsion
    DoubleEqual,      // ==
    NotEqual,         // !=
    GreaterEqualThan, // >=
    LessEqualThan,    // <=
    LessThan,         // <
    GreaterThan,      // >
}

#[derive(Debug, PartialEq)]
pub struct Token<'src> {
    pub kind: TokenType,
    pub text: &'src str,
    pub span: Span,
}

impl<'src> Token<'src> {
    fn new(source: &'src str, kind: TokenType, start: usize, end: usize) -> Self {
        Token {
            kind,
            text: &source[start..end],
            span: Span { start, end },
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl From<Span> for miette::SourceSpan {
    fn from(s: Span) -> Self {
        (s.start, s.end - s.start).into()
    }
}

#[allow(clippy::too_many_lines)]
pub fn lex<'src>(source: &'_ str) -> Result<Vec<Token<'_>>, LexError> {
    let mut chars = source.char_indices().peekable();
    let mut tokens: Vec<Token> = Vec::new();

    while let Some((pos, ch)) = chars.next() {
        match ch {
            '=' => {
                if chars.peek().is_some_and(|(_, ch)| *ch == '=') {
                    tokens.push(Token::new(source, TokenType::DoubleEqual, pos, pos + 2));
                    chars.next();
                    continue;
                }

                tokens.push(Token::new(source, TokenType::Equal, pos, pos + 1));
            }
            '|' => tokens.push(Token::new(source, TokenType::Bar, pos, pos + 1)),
            '\n' => tokens.push(Token::new(source, TokenType::Newline, pos, pos + 1)),
            '$' => tokens.push(Token::new(source, TokenType::DollarSign, pos, pos + 1)),
            '{' => tokens.push(Token::new(source, TokenType::OpenBrace, pos, pos + 1)),
            '}' => tokens.push(Token::new(source, TokenType::CloseBrace, pos, pos + 1)),
            '(' => tokens.push(Token::new(source, TokenType::OpenParenthesis, pos, pos + 1)),
            ')' => tokens.push(Token::new(
                source,
                TokenType::CloseParenthesis,
                pos,
                pos + 1,
            )),
            '"' => {
                let mut escaped = false;
                let mut terminated = false;

                for (inner_pos, inner_char) in chars.by_ref() {
                    if inner_char == '\\' {
                        escaped = true;
                        continue;
                    }

                    // not escaped ending "
                    if !escaped && inner_char == '"' {
                        tokens.push(Token {
                            kind: TokenType::StringLiteral,
                            text: &source[pos + 1..inner_pos],
                            span: Span {
                                start: pos,
                                end: inner_pos + 1,
                            },
                        });
                        terminated = true;
                        break;
                    }

                    escaped = false;
                }

                if !terminated {
                    return Err(LexError::UnterminatedString { pos });
                }
            }
            '#' => {
                chars.find(|(_, c)| *c == '\n');
            }
            '>' => {
                if chars.peek().is_some_and(|(_, char)| *char == '=') {
                    chars.next();
                    tokens.push(Token::new(
                        source,
                        TokenType::GreaterEqualThan,
                        pos,
                        pos + 2,
                    ));
                } else {
                    tokens.push(Token::new(source, TokenType::GreaterThan, pos, pos + 1));
                }
            }
            '<' => {
                if chars.peek().is_some_and(|(_, char)| *char == '=') {
                    chars.next();
                    tokens.push(Token::new(source, TokenType::LessEqualThan, pos, pos + 2));
                } else {
                    tokens.push(Token::new(source, TokenType::LessThan, pos, pos + 1));
                }
            }
            '!' => {
                if chars.peek().is_some_and(|(_, char)| *char == '=') {
                    chars.next();
                    tokens.push(Token::new(source, TokenType::NotEqual, pos, pos + 2));
                } else {
                    tokens.push(Token::new(source, TokenType::Bang, pos, pos + 1));
                }
            }
            '\'' => {
                if let Some((end_pos, _)) = chars.find(|(_, c)| *c == '\'') {
                    tokens.push(Token {
                        kind: TokenType::RawString,
                        text: &source[pos + 1..end_pos],
                        span: Span {
                            start: pos,
                            end: end_pos + 1,
                        },
                    });
                } else {
                    return Err(LexError::UnterminatedString { pos });
                }
            }
            ch if ch.is_whitespace() => {}
            ch if ch.is_ascii_digit() => {
                // advance until no more numbers left
                let mut is_float = false;

                while let Some(&(next_pos, next_ch)) = chars.peek() {
                    if next_ch.is_ascii_digit() {
                        chars.next();
                    } else if next_ch == '.' {
                        if is_float {
                            return Err(LexError::UnexpectedDot { pos: next_pos });
                        }
                        is_float = true;
                        chars.next();
                    } else {
                        break;
                    }
                }

                let end = chars.peek().map_or(source.len(), |(p, _)| *p);

                tokens.push(Token::new(
                    source,
                    TokenType::Number(if is_float {
                        NumberType::Float
                    } else {
                        NumberType::Integer
                    }),
                    pos,
                    end,
                ));
            }
            _ => {
                while chars.peek().is_some_and(|(_, ch)| is_identifier_char(*ch)) {
                    chars.next();
                }

                let end = chars.peek().map_or(source.len(), |(next_pos, _)| *next_pos);
                tokens.push(Token::new(source, TokenType::Identifier, pos, end));
            }
        }
    }

    Ok(tokens)
}

fn is_identifier_char(ch: char) -> bool {
    !ch.is_whitespace()
        && !matches!(
            ch,
            '|' | '=' | '"' | '\'' | '\n' | '$' | '(' | ')' | '{' | '}' | '#' | '!' | '<' | '>'
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_command() {
        let tokens = lex("ls ./src | where ext == \"rs\"").unwrap();

        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        let texts: Vec<_> = tokens.iter().map(|t| t.text).collect();

        assert_eq!(
            kinds,
            vec![
                &TokenType::Identifier,
                &TokenType::Identifier,
                &TokenType::Bar,
                &TokenType::Identifier,
                &TokenType::Identifier,
                &TokenType::DoubleEqual,
                &TokenType::StringLiteral,
            ]
        );

        assert_eq!(texts, vec!["ls", "./src", "|", "where", "ext", "==", "rs"]);
    }

    #[test]
    fn unterminated_raw_string() {
        let res = lex("echo 'test");
        assert_eq!(res, Err(LexError::UnterminatedString { pos: 5 }));
    }

    #[test]
    fn unterminated_string() {
        let res = lex("echo \"test");
        assert_eq!(res, Err(LexError::UnterminatedString { pos: 5 }));
    }

    #[test]
    fn integer_number() {
        let res = lex("5050").unwrap();

        assert_eq!(TokenType::Number(NumberType::Integer), res[0].kind);
        assert_eq!("5050", res[0].text);
    }

    #[test]
    fn float_number() {
        let res = lex("5050.420").unwrap();

        assert_eq!(TokenType::Number(NumberType::Float), res[0].kind);
        assert_eq!("5050.420", res[0].text);
    }

    #[test]
    fn float_number_dot_end() {
        let res = lex("5050.").unwrap();

        assert_eq!(TokenType::Number(NumberType::Float), res[0].kind);
        assert_eq!("5050.", res[0].text);
    }

    #[test]
    fn number_double_dot() {
        let res = lex("5.32.1");

        assert_eq!(res, Err(LexError::UnexpectedDot { pos: 4 }));
    }

    #[test]
    fn escape_in_raw_string() {
        let res = lex("'test \'").unwrap();

        assert_eq!(TokenType::RawString, res[0].kind);
        assert_eq!("test ", res[0].text);
    }

    #[test]
    fn escape_in_string_literal() {
        let tokens = lex(r#""hello \"world\"""#).unwrap();

        assert_eq!(TokenType::StringLiteral, tokens[0].kind);
        assert_eq!(r#"hello \"world\""#, tokens[0].text);
    }

    #[test]
    fn single_equal() {
        let tokens = lex("VAR_NAME=\"test\"").unwrap();

        assert_eq!(TokenType::Identifier, tokens[0].kind);
        assert_eq!("VAR_NAME", tokens[0].text);

        assert_eq!(TokenType::Equal, tokens[1].kind);
        assert_eq!("=", tokens[1].text);

        assert_eq!(TokenType::StringLiteral, tokens[2].kind);
        assert_eq!("test", tokens[2].text);
    }

    #[test]
    fn mutliple_pipes() {
        let tokens = lex("ls | sort | take 5").unwrap();

        assert_eq!(TokenType::Identifier, tokens[0].kind);
        assert_eq!("ls", tokens[0].text);

        assert_eq!(TokenType::Bar, tokens[1].kind);

        assert_eq!(TokenType::Identifier, tokens[2].kind);
        assert_eq!("sort", tokens[2].text);

        assert_eq!(TokenType::Bar, tokens[3].kind);

        assert_eq!(TokenType::Identifier, tokens[4].kind);
        assert_eq!("take", tokens[4].text);

        assert_eq!(TokenType::Number(NumberType::Integer), tokens[5].kind);
        assert_eq!("5", tokens[5].text);
    }

    #[test]
    fn complex() {
        let input = r#"ls ./src | where ext == "rs" | sort-by size | take 5
# this is a comment
let count = 42 + 3.14
echo $name != 'hello world'
size >= 100 | each { process($item) }"#;
        let tokens = lex(input).unwrap();

        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        let texts: Vec<_> = tokens.iter().map(|t| t.text).collect();

        assert_eq!(
            kinds,
            vec![
                // ls ./src | where ext == "rs" | sort-by size | take 5
                &TokenType::Identifier,
                &TokenType::Identifier,
                &TokenType::Bar,
                &TokenType::Identifier,
                &TokenType::Identifier,
                &TokenType::DoubleEqual,
                &TokenType::StringLiteral,
                &TokenType::Bar,
                &TokenType::Identifier,
                &TokenType::Identifier,
                &TokenType::Bar,
                &TokenType::Identifier,
                &TokenType::Number(NumberType::Integer),
                &TokenType::Newline,
                // # this is a comment (consumed, no tokens)
                // let count = 42 + 3.14
                &TokenType::Identifier,
                &TokenType::Identifier,
                &TokenType::Equal,
                &TokenType::Number(NumberType::Integer),
                &TokenType::Identifier, // "+"
                &TokenType::Number(NumberType::Float),
                &TokenType::Newline,
                // echo $name != 'hello world'
                &TokenType::Identifier,
                &TokenType::DollarSign,
                &TokenType::Identifier,
                &TokenType::NotEqual,
                &TokenType::RawString,
                &TokenType::Newline,
                // size >= 100 | each { process($item) }
                &TokenType::Identifier,
                &TokenType::GreaterEqualThan,
                &TokenType::Number(NumberType::Integer),
                &TokenType::Bar,
                &TokenType::Identifier,
                &TokenType::OpenBrace,
                &TokenType::Identifier,
                &TokenType::OpenParenthesis,
                &TokenType::DollarSign,
                &TokenType::Identifier,
                &TokenType::CloseParenthesis,
                &TokenType::CloseBrace,
            ]
        );
        assert_eq!(
            texts,
            vec![
                "ls",
                "./src",
                "|",
                "where",
                "ext",
                "==",
                "rs",
                "|",
                "sort-by",
                "size",
                "|",
                "take",
                "5",
                "\n",
                "let",
                "count",
                "=",
                "42",
                "+",
                "3.14",
                "\n",
                "echo",
                "$",
                "name",
                "!=",
                "hello world",
                "\n",
                "size",
                ">=",
                "100",
                "|",
                "each",
                "{",
                "process",
                "(",
                "$",
                "item",
                ")",
                "}",
            ]
        );
    }
}

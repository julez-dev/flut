use crate::lexer::{self, NumberType, Token, TokenType};

#[derive(thiserror::Error, miette::Diagnostic, Debug, PartialEq)]
enum ParseError {
    #[error("received unexpected token")]
    #[diagnostic(help("got: {got} want: {want}"))]
    UnexpectedToken { got: String, want: String },

    #[error("missing {missing} token")]
    #[diagnostic(help("add missing token {missing}"))]
    MissingToken { missing: String },
}

#[derive(Debug, PartialEq)]
pub struct Command {
    name: String,
    args: Vec<Expression>,
}

#[derive(Debug, PartialEq)]
enum NumberLiteral {
    Integer(i64),
    Float(f64),
}

#[derive(Debug, PartialEq)]
enum Expression {
    StringLiteral(String),
    Number(NumberLiteral),
    Identifier(String),
    Variable(String),
    Command(Command),
    BinaryOperation {
        op: BinaryOperationKind,
        left: Box<Expression>,
        right: Box<Expression>,
    },
}

#[derive(Debug, PartialEq)]
enum BinaryOperationKind {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterEqual,
    LessThan,
    LessEqual,
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, PartialEq)]
enum Statement {
    Pipeline(Vec<Command>),
    LetBinding { name: String, value: Expression },
}

struct Parser<'src> {
    tokens: &'src [Token<'src>],
    pos: usize,
}

impl<'src> Parser<'src> {
    pub fn new(tokens: &'src [Token<'src>]) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>, ParseError> {
        Ok(Vec::new())
    }

    fn parse_command(&mut self) -> Result<Command, ParseError> {
        let command_name_token = self.expect(lexer::TokenType::Identifier)?.text.to_string();

        let mut args: Vec<Expression> = vec![];

        while let Some(token) = self.peek() {
            match &token.kind {
                TokenType::Newline | TokenType::Bar => break,
                TokenType::Identifier => {
                    args.push(Expression::Identifier(token.text.to_string()));
                    self.advance().unwrap();
                }
                TokenType::RawString | TokenType::StringLiteral => {
                    args.push(Expression::StringLiteral(token.text.to_string()));
                    self.advance().unwrap();
                }
                TokenType::Number(number_type) => {
                    let number = if *number_type == NumberType::Float {
                        NumberLiteral::Float(token.text.parse().unwrap())
                    } else {
                        NumberLiteral::Integer(token.text.parse().unwrap())
                    };

                    args.push(Expression::Number(number));
                    self.advance().unwrap();
                }
                TokenType::DollarSign => {
                    let exp = self.parse_variable()?;
                    args.push(exp);
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        got: token.kind.to_string(),
                        want: "valid arg token".into(),
                    });
                }
            }
        }

        Ok(Command {
            name: command_name_token,
            args,
        })
    }

    fn parse_variable(&mut self) -> Result<Expression, ParseError> {
        let dollar_end = self.expect(TokenType::DollarSign)?.span.end;
        let name = self.expect(TokenType::Identifier)?;

        // name of var needs to directly start after $ token
        if dollar_end != name.span.start {
            return Err(ParseError::MissingToken {
                missing: TokenType::Identifier.to_string(),
            });
        }

        Ok(Expression::Variable(name.text.to_string()))
    }

    fn peek(&self) -> Option<&Token<'src>> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token<'src>> {
        let token = self.tokens.get(self.pos);
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    fn expect(&mut self, expected: lexer::TokenType) -> Result<&Token<'src>, ParseError> {
        match self.peek() {
            Some(token) if token.kind == expected => Ok(self.advance().unwrap()),
            Some(token) => Err(ParseError::UnexpectedToken {
                got: token.kind.to_string(),
                want: expected.to_string(),
            }),
            None => Err(ParseError::MissingToken {
                missing: expected.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variable_parsing() {
        let tokens = lexer::lex("$varname").unwrap();
        let mut parser = Parser::new(&tokens);
        let res = parser.parse_variable().unwrap();

        match &res {
            Expression::Variable(name) => {
                assert_eq!(name, "varname");
            }
            _ => panic!("type should be variable"),
        }
    }

    #[test]
    fn variable_parsing_space() {
        let tokens = lexer::lex("$ varname").unwrap();
        let mut parser = Parser::new(&tokens);
        let err = parser.parse_variable().unwrap_err();

        assert_eq!(
            err,
            ParseError::MissingToken {
                missing: "identifier".into()
            }
        );
    }

    #[test]
    fn command_parsing() {
        let tokens = lexer::lex("ls ./src").unwrap();
        let mut parser = Parser::new(&tokens);
        let res = parser.parse_command().unwrap();

        assert_eq!(
            res,
            Command {
                name: "ls".into(),
                args: vec![Expression::Identifier("./src".to_string())]
            }
        );
    }

    #[test]
    fn command_parsing_with_var() {
        let tokens = lexer::lex("ls $var").expect("lexing should be ok");
        let mut parser = Parser::new(&tokens);
        let res = parser.parse_command().expect("is valid command");

        assert_eq!(
            res,
            Command {
                name: "ls".into(),
                args: vec![Expression::Variable("var".to_string())]
            }
        );
    }
}

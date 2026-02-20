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

impl TryFrom<&TokenType> for BinaryOperationKind {
    type Error = ParseError;
    fn try_from(value: &TokenType) -> Result<Self, Self::Error> {
        match value {
            TokenType::DoubleEqual => Ok(Self::Equal),
            TokenType::NotEqual => Ok(Self::NotEqual),
            TokenType::LessThan => Ok(Self::LessThan),
            TokenType::GreaterThan => Ok(Self::GreaterThan),
            TokenType::LessEqualThan => Ok(Self::LessEqual),
            TokenType::GreaterEqualThan => Ok(Self::GreaterEqual),
            TokenType::Multiply => Ok(Self::Multiply),
            TokenType::Plus => Ok(Self::Add),
            _ => Err(ParseError::UnexpectedToken {
                got: value.to_string(),
                want: "operator".into(),
            }),
        }
    }
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
        let mut statements = vec![];

        while let Some(token) = self.peek() {
            match &token.kind {
                TokenType::Newline => {
                    self.advance();
                }
                TokenType::Identifier if token.text == "let" => {
                    statements.push(self.parse_let()?);
                }
                _ => {
                    statements.push(self.parse_pipeline()?);
                }
            }
        }

        Ok(statements)
    }

    fn parse_pipeline(&mut self) -> Result<Statement, ParseError> {
        let mut commands: Vec<Command> = vec![self.parse_command()?];

        while self.peek().is_some_and(|t| t.kind == TokenType::Bar) {
            self.advance();
            commands.push(self.parse_command()?);
        }

        Ok(Statement::Pipeline(commands))
    }

    fn parse_let(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenType::Identifier)?;

        let var_name = self.expect(TokenType::Identifier)?.text.to_string();
        self.expect(TokenType::Equal)?;

        let expr = self.parse_expression(0)?;

        Ok(Statement::LetBinding {
            name: var_name,
            value: expr,
        })
    }

    fn parse_command(&mut self) -> Result<Command, ParseError> {
        let command_name_token = self.expect(lexer::TokenType::Identifier)?.text.to_string();

        let mut args: Vec<Expression> = vec![];

        while let Some(token) = self.peek() {
            match &token.kind {
                TokenType::Newline | TokenType::Bar => break,
                _ => {
                    let exp = self.parse_expression(0)?;
                    args.push(exp);
                }
            }
        }

        Ok(Command {
            name: command_name_token,
            args,
        })
    }

    fn parse_expression(&mut self, min_power: u8) -> Result<Expression, ParseError> {
        let mut left = self.parse_atom()?;

        while let Some(token) = self.peek() {
            let binding_power = binding_power(&token.kind);

            match binding_power {
                None => break,
                Some(power) if power < min_power => break,
                Some(power) => {
                    let op = BinaryOperationKind::try_from(&token.kind)?;
                    self.advance();
                    let right = self.parse_expression(power + 1)?;

                    left = Expression::BinaryOperation {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                    }
                }
            }
        }

        Ok(left)
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

    fn parse_atom(&mut self) -> Result<Expression, ParseError> {
        if self.peek().is_some_and(|t| t.kind == TokenType::DollarSign) {
            return self.parse_variable();
        }

        match self.advance() {
            None => {
                return Err(ParseError::MissingToken {
                    missing: "Identifier".into(),
                });
            }
            Some(token) => match &token.kind {
                TokenType::Identifier => {
                    return Ok(Expression::Identifier(token.text.to_string()));
                }
                TokenType::RawString | TokenType::StringLiteral => {
                    return Ok(Expression::StringLiteral(token.text.to_string()));
                }
                TokenType::Number(number_type) => {
                    let number = if *number_type == NumberType::Float {
                        NumberLiteral::Float(token.text.parse().unwrap())
                    } else {
                        NumberLiteral::Integer(token.text.parse().unwrap())
                    };

                    return Ok(Expression::Number(number));
                }
                TokenType::OpenParenthesis => {
                    let expr = self.parse_expression(0)?;
                    self.expect(TokenType::CloseParenthesis)?;
                    Ok(expr)
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        got: token.kind.to_string(),
                        want: "valid arg token".into(),
                    });
                }
            },
        }
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

fn binding_power(kind: &TokenType) -> Option<u8> {
    match kind {
        TokenType::DoubleEqual
        | TokenType::NotEqual
        | TokenType::GreaterThan
        | TokenType::GreaterEqualThan
        | TokenType::LessEqualThan
        | TokenType::LessThan => Some(1),
        TokenType::Plus => Some(2),
        TokenType::Multiply => Some(3),
        _ => None,
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
    fn pipeline_parsing() {
        let tokens = lexer::lex("ls ./src | where ext == \"rs\" | take 5").unwrap();

        let mut parser = Parser::new(&tokens);
        let res = parser.parse_pipeline().unwrap();
        dbg!(&res);

        assert_eq!(res, Statement::Pipeline(todo!()));
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

    #[test]
    fn simple_binop() {
        let tokens = lexer::lex("a == b != c").unwrap();
        let mut parser = Parser::new(&tokens);
        let expr = parser.parse_expression(0).unwrap();
        dbg!(&expr);
    }

    #[test]
    fn simple_binop_variable() {
        let tokens = lexer::lex("$x == 10").unwrap();
        let mut parser = Parser::new(&tokens);
        let expr = parser.parse_expression(0).unwrap();
        dbg!(&expr);
    }

    #[test]
    fn parse_full_program() {
        let input = "let x = 5\nls ./src | where ext == \"rs\" && size > 10 | take 3";
        let tokens = lexer::lex(input).unwrap();
        let mut parser = Parser::new(&tokens);
        let stmts = parser.parse().unwrap();

        // produces
        //
        /*
        * [src/parser.rs:369:9] &stmts = [
            LetBinding {
                name: "x",
                value: Number(
                    Integer(
                        5,
                    ),
                ),
            },
            Pipeline(
                [
                    Command {
                        name: "ls",
                        args: [
                            Identifier(
                                "./src",
                            ),
                        ],
                    },
                    Command {
                        name: "where",
                        args: [
                            BinaryOperation {
                                op: Equal,
                                left: Identifier(
                                    "ext",
                                ),
                                right: StringLiteral(
                                    "rs",
                                ),
                            },
                        ],
                    },
                ],
            ),
        ]
        *
        */

        dbg!(&stmts);
    }

    #[test]
    fn arithmetic_precedence() {
        // should parse as 1 + (2 * 3), not (1 + 2) * 3
        let tokens = lexer::lex("1 + 2 * 3").unwrap();
        let mut parser = Parser::new(&tokens);
        let expr = parser.parse_expression(0).unwrap();
        dbg!(&expr);
    }
}

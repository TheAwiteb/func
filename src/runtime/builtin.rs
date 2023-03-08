use crate::common::{
    ast::FunctionStatement,
    error::{Error, ErrorType},
    object::{Meta, Object},
    position::Position,
    token::{Token, TokenType},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Builtin {
    Len,
    First,
    Last,
    Write,
    WriteLn,
    Readln,
    Pop,
    Push,
}

impl Builtin {
    /// Returns the number of parameters the builtin function takes.
    pub fn parameters(&self) -> Vec<Token> {
        match self {
            Self::Len => vec![Token::new(
                TokenType::Identifier,
                "value".to_string(),
                None,
                Position::new("builtin".to_string(), 0),
            )],
            Self::First => vec![Token::new(
                TokenType::Identifier,
                "value".to_string(),
                None,
                Position::new("builtin".to_string(), 0),
            )],
            Self::Last => vec![Token::new(
                TokenType::Identifier,
                "value".to_string(),
                None,
                Position::new("builtin".to_string(), 0),
            )],
            Self::Write => vec![Token::new(
                TokenType::Identifier,
                "value".to_string(),
                None,
                Position::new("builtin".to_string(), 0),
            )],
            Self::WriteLn => vec![Token::new(
                TokenType::Identifier,
                "value".to_string(),
                None,
                Position::new("builtin".to_string(), 0),
            )],
            Self::Readln => vec![Token::new(
                TokenType::Identifier,
                "prompt".to_string(),
                None,
                Position::new("builtin".to_string(), 0),
            )],
            Self::Pop => vec![Token::new(
                TokenType::Identifier,
                "popable".to_string(),
                None,
                Position::new("builtin".to_string(), 0),
            )],
            Self::Push => vec![
                Token::new(
                    TokenType::Identifier,
                    "pushable".to_string(),
                    None,
                    Position::new("builtin".to_string(), 0),
                ),
                Token::new(
                    TokenType::Identifier,
                    "value".to_string(),
                    None,
                    Position::new("builtin".to_string(), 0),
                ),
            ],
        }
    }

    /// Initializes all builtin functions. Returns a vector of functions statements.
    pub fn init() -> Vec<FunctionStatement> {
        [
            Self::Len,
            Self::First,
            Self::Last,
            Self::Write,
            Self::WriteLn,
            Self::Readln,
            Self::Pop,
            Self::Push,
        ]
        .iter()
        .map(|builtin| {
            FunctionStatement::new(
                Token::new(
                    TokenType::Identifier,
                    builtin.to_string(),
                    None,
                    Position::new("builtin".to_string(), 0),
                ),
                builtin.parameters(),
                None,
            )
        })
        .collect()
    }

    pub fn execute(&self, args: Vec<Object>, position: Position) -> Result<Object, Error> {
        match self {
            Builtin::Len => match &args[0] {
                Object::String(string, ..) => {
                    Ok(Object::Number(string.len() as f64, Meta::default()))
                }
                Object::Array(array, ..) => Ok(Object::Number(array.len() as f64, Meta::default())),
                _ => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!("argument to `len` not supported, got {}", args[0]),
                    position,
                )),
            },
            Builtin::First => match &args[0] {
                Object::Array(array, ..) => {
                    if !array.is_empty() {
                        Ok(array[0].clone())
                    } else {
                        Ok(Object::Nil(Meta::default()))
                    }
                }
                _ => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!("argument to `first` not supported, got {}", args[0]),
                    position,
                )),
            },
            Builtin::Last => match &args[0] {
                Object::Array(array, ..) => {
                    if !array.is_empty() {
                        Ok(array[array.len() - 1].clone())
                    } else {
                        Ok(Object::Nil(Meta::default()))
                    }
                }
                _ => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!("argument to `last` not supported, got {}", args[0]),
                    position,
                )),
            },
            Builtin::Write => {
                print!("{}", args[0]);
                Ok(Object::Nil(Meta::default()))
            }
            Builtin::WriteLn => {
                println!("{}", args[0]);
                Ok(Object::Nil(Meta::default()))
            }
            Builtin::Readln => rustyline::DefaultEditor::new()
                .map_err(|_| {
                    Error::new(
                        ErrorType::RuntimeError,
                        "failed to initialize readline".to_string(),
                        position.clone(),
                    )
                })?
                .readline(&args[0].to_string())
                .map_err(|_| {
                    Error::new(
                        ErrorType::RuntimeError,
                        "failed to read line".to_string(),
                        position,
                    )
                })
                .map(|line| Object::String(line, Meta::default())),
            Builtin::Pop => match &args[0] {
                Object::Array(array, ..) => {
                    if !array.is_empty() {
                        Ok(array.clone().pop().unwrap())
                    } else {
                        Ok(Object::Nil(Meta::default()))
                    }
                }
                _ => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!("argument to `pop` not supported, got {}", args[0]),
                    position,
                )),
            },
            Builtin::Push => match &args[0] {
                Object::Array(array, ..) => {
                    let mut array = array.clone();
                    array.push(args[1].clone());
                    Ok(Object::Array(array, Meta::default()))
                }
                _ => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!("argument to `push` not supported, got {}", args[0]),
                    position,
                )),
            },
        }
    }
}

impl ToString for Builtin {
    fn to_string(&self) -> String {
        match self {
            Self::Len => "len".to_string(),
            Self::First => "first".to_string(),
            Self::Last => "last".to_string(),
            Self::Write => "write".to_string(),
            Self::WriteLn => "writeln".to_string(),
            Self::Readln => "readln".to_string(),
            Self::Pop => "pop".to_string(),
            Self::Push => "push".to_string(),
        }
    }
}

impl TryFrom<Token> for Builtin {
    type Error = Error;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.lexeme.as_str() {
            "len" => Ok(Self::Len),
            "first" => Ok(Self::First),
            "last" => Ok(Self::Last),
            "write" => Ok(Self::Write),
            "writeln" => Ok(Self::WriteLn),
            "readln" => Ok(Self::Readln),
            "pop" => Ok(Self::Pop),
            "push" => Ok(Self::Push),
            _ => Err(Error::new(
                ErrorType::RuntimeError,
                format!("unknown builtin function: {}", value.lexeme),
                value.position,
            )),
        }
    }
}

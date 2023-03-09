use crate::common::{
    ast::{FunctionStatement, Parameter},
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
    Format,
}

impl Builtin {
    /// Returns the number of parameters the builtin function takes.
    pub fn parameters(&self) -> Vec<Parameter> {
        match self {
            Self::Len => vec![Parameter::new(
                Token::new(
                    TokenType::Identifier,
                    "value".to_string(),
                    None,
                    Position::new("builtin".to_string(), 0),
                ),
                false,
            )],
            Self::First => vec![Parameter::new(
                Token::new(
                    TokenType::Identifier,
                    "value".to_string(),
                    None,
                    Position::new("builtin".to_string(), 0),
                ),
                false,
            )],
            Self::Last => vec![Parameter::new(
                Token::new(
                    TokenType::Identifier,
                    "value".to_string(),
                    None,
                    Position::new("builtin".to_string(), 0),
                ),
                false,
            )],
            Self::Write => vec![Parameter::new(
                Token::new(
                    TokenType::Identifier,
                    "value".to_string(),
                    None,
                    Position::new("builtin".to_string(), 0),
                ),
                true,
            )],
            Self::Format => vec![
                Parameter::new(
                    Token::new(
                        TokenType::Identifier,
                        "format".to_string(),
                        None,
                        Position::new("builtin".to_string(), 0),
                    ),
                    false,
                ),
                Parameter::new(
                    Token::new(
                        TokenType::Identifier,
                        "args".to_string(),
                        None,
                        Position::new("builtin".to_string(), 0),
                    ),
                    true,
                ),
            ],
            Self::WriteLn => vec![Parameter::new(
                Token::new(
                    TokenType::Identifier,
                    "value".to_string(),
                    None,
                    Position::new("builtin".to_string(), 0),
                ),
                true,
            )],
            Self::Readln => vec![Parameter::new(
                Token::new(
                    TokenType::Identifier,
                    "prompt".to_string(),
                    None,
                    Position::new("builtin".to_string(), 0),
                ),
                false,
            )],
            Self::Pop => vec![Parameter::new(
                Token::new(
                    TokenType::Identifier,
                    "popable".to_string(),
                    None,
                    Position::new("builtin".to_string(), 0),
                ),
                false,
            )],
            Self::Push => vec![
                Parameter::new(
                    Token::new(
                        TokenType::Identifier,
                        "pushable".to_string(),
                        None,
                        Position::new("builtin".to_string(), 0),
                    ),
                    false,
                ),
                Parameter::new(
                    Token::new(
                        TokenType::Identifier,
                        "value".to_string(),
                        None,
                        Position::new("builtin".to_string(), 0),
                    ),
                    false,
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
            Self::Format,
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
            Builtin::Format => {
                let format = args[0].to_string();
                let args = match &args[1] {
                    Object::Array(array, ..) => array,
                    _ => {
                        return Err(Error::new(
                            ErrorType::RuntimeError,
                            format!(
                                "Expected array as second argument to `format`, got {}",
                                args[1]
                            ),
                            position,
                        ))
                    }
                };
                // Replace all {} with the corresponding argument, to escape use {{ and }}
                // If the placeholder contains a number, use that argument instead
                // If the placeholder empty, use the next argument
                // Else return an error
                // Note: All this will done with regex
                let mut result = String::new();
                let mut arg_index = 0;
                let mut placeholder = false;
                let mut close_placeholder = false;
                let mut placeholder_number = String::new();

                for c in format.chars() {
                    if placeholder {
                        if c == '}' {
                            if placeholder_number.is_empty() {
                                if arg_index < args.len() {
                                    result.push_str(&args[arg_index].to_string());
                                    arg_index += 1;
                                } else {
                                    return Err(Error::new(
                                        ErrorType::RuntimeError,
                                        "Not enough arguments for format string".to_string(),
                                        position,
                                    ));
                                }
                            } else {
                                let placeholder_number: usize =
                                    placeholder_number.parse().map_err(|_| {
                                        Error::new(
                                            ErrorType::RuntimeError,
                                            format!(
                                                "Invalid placeholder index: {}",
                                                placeholder_number
                                            ),
                                            position.clone(),
                                        )
                                    })?;
                                if placeholder_number < args.len() {
                                    result.push_str(&args[placeholder_number].to_string());
                                } else {
                                    return Err(Error::new(
                                        ErrorType::RuntimeError,
                                        format!(
                                            "Not enough arguments for format string, expected at least {}",
                                            placeholder_number
                                        ),
                                        position,
                                    ));
                                }
                            }
                            placeholder = false;
                            placeholder_number.clear();
                        } else if c == '{' {
                            result.push('{');
                            placeholder = false;
                        } else if c.is_numeric() {
                            placeholder_number.push(c);
                        } else {
                            return Err(Error::new(
                                ErrorType::RuntimeError,
                                format!("Invalid placeholder: {}", c),
                                position,
                            ));
                        }
                    } else if c == '{' {
                        placeholder = true;
                    } else if c == '}' && !close_placeholder {
                        close_placeholder = true;
                    } else if c == '}' && close_placeholder {
                        result.push('}');
                        close_placeholder = false;
                    } else if close_placeholder {
                        // Unclosed placeholder
                        return Err(Error::new(
                            ErrorType::RuntimeError,
                            "Unclosed placeholder".to_string(),
                            position,
                        ));
                    } else {
                        result.push(c);
                    }
                }

                Ok(Object::String(result.to_string(), Meta::default()))
            }
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
            Self::Format => "format".to_string(),
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
            "format" => Ok(Self::Format),
            _ => Err(Error::new(
                ErrorType::RuntimeError,
                format!("unknown builtin function: {}", value.lexeme),
                value.position,
            )),
        }
    }
}

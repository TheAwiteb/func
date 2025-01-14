use std::cmp::Ordering;

use crate::common::{
    ast::{
        AssignmentStatement, BinaryExpression, BlockExpression, CallExpression, ElseBlock,
        Expression, FunctionStatement, GroupExpression, IdentifierExpression, IfExpression,
        LetStatement, Program, Statement, UnaryExpression,
    },
    error::{Error, ErrorType},
    object::{Meta, Object},
    token::TokenType,
};

use super::{
    builtin::Builtin,
    environment::{FunctionBindings, VariableBindings},
};

#[derive(Default)]
pub struct Interpreter {
    variables: VariableBindings,
    functions: FunctionBindings,
}

impl Interpreter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn interpret(&mut self, program: Program) -> Result<(), Error> {
        for statement in program {
            self.execute_statement(statement)?;
        }
        Ok(())
    }

    fn execute_statement(&mut self, statement: Statement) -> Result<Object, Error> {
        match statement {
            Statement::Let(let_statement) => self.execute_let_statement(let_statement),

            Statement::Assignment(assignment_statement) => {
                self.execute_assignment_statement(assignment_statement)
            }

            Statement::Function(function_statement) => {
                self.define_function_statement(function_statement)
            }

            Statement::Expression(expression) => self.evaluate_expression(expression),
            _ => Ok(Object::Nil(Meta::default())),
        }
    }

    fn execute_let_statement(&mut self, let_statement: LetStatement) -> Result<Object, Error> {
        let identifier = let_statement.identifier;
        let value = self.evaluate_expression(let_statement.expression)?;
        self.variables.declare(identifier, value.clone());

        Ok(value)
    }

    fn execute_assignment_statement(
        &mut self,
        assignment_statement: AssignmentStatement,
    ) -> Result<Object, Error> {
        let identifier = assignment_statement.identifier;
        self.variables.get(identifier.clone())?;
        let value = self.evaluate_expression(assignment_statement.expression)?;
        self.variables.assign(identifier, value.clone())?;

        Ok(value)
    }

    fn define_function_statement(
        &mut self,
        function_statement: FunctionStatement,
    ) -> Result<Object, Error> {
        self.functions
            .put(function_statement.identifier.clone(), function_statement);
        Ok(Object::Nil(Meta::default()))
    }

    fn execute_function_statement(
        &mut self,
        arguments: Vec<Expression>,
        function_statement: FunctionStatement,
    ) -> Result<Object, Error> {
        let old_variables = self.variables.clone();
        for (param, argument) in function_statement.paramiters.iter().zip(arguments.iter()) {
            let value = self.evaluate_expression(argument.clone())?;
            self.variables.declare(param.identifier.clone(), value);
        }
        let return_value = if let Some(block_expression) = function_statement.block {
            self.evaluate_block_expression(block_expression)?
        } else {
            // If there is no block expression, that means the function is built-in.
            Builtin::try_from(function_statement.identifier.clone())?.execute(
                arguments
                    .iter()
                    .map(|expression| self.evaluate_expression(expression.clone()))
                    .collect::<Result<Vec<_>, _>>()?,
                function_statement.identifier.position,
            )?
        };

        self.variables = old_variables;
        Ok(return_value)
    }

    fn evaluate_if_expression(&mut self, if_statement: IfExpression) -> Result<Object, Error> {
        let condition = self.evaluate_expression(*if_statement.condition)?;
        if condition.is_true() {
            self.evaluate_block_expression(if_statement.if_block)
        } else if let Some(else_block) = *if_statement.else_block {
            match else_block {
                ElseBlock::Block(block_statment) => self.evaluate_block_expression(block_statment),
                ElseBlock::If(if_statement) => self.evaluate_if_expression(if_statement),
            }
        } else {
            Ok(Object::Nil(Meta::default()))
        }
    }

    fn evaluate_block_expression(
        &mut self,
        block_expression: BlockExpression,
    ) -> Result<Object, Error> {
        let old_variables = self.variables.clone();
        let mut return_value = Object::Nil(Meta::default());
        for statement in *block_expression.statements {
            if let Statement::Return(return_expression) = statement {
                return_value = self.evaluate_expression(return_expression)?;
                return_value.set_return();
                break;
            }
            return_value = self.execute_statement(statement.clone())?;
            if return_value.is_return() {
                break;
            }
        }
        self.variables = old_variables;
        Ok(return_value)
    }

    fn evaluate_expression(&mut self, expression: Expression) -> Result<Object, Error> {
        self.match_expression(expression)
    }

    fn evaluate_binary_expression(
        &mut self,
        binary_expression: BinaryExpression,
    ) -> Result<Object, Error> {
        let left = self.match_expression(*binary_expression.left)?;

        let right = self.match_expression(*binary_expression.right)?;

        match binary_expression.operator.ttype {
            TokenType::And => Ok(Object::Boolean(
                left.is_true() && right.is_true(),
                Meta::default(),
            )),

            TokenType::Or => Ok(Object::Boolean(
                left.is_true() || right.is_true(),
                Meta::default(),
            )),

            TokenType::EqualEqual => Ok(Object::Boolean(left == right, Meta::default())),

            TokenType::NotEqual => Ok(Object::Boolean(left != right, Meta::default())),

            TokenType::Greater => match (left, right) {
                (Object::Number(x, ..), Object::Number(y, ..)) => {
                    Ok(Object::Boolean(x > y, Meta::default()))
                }

                (Object::Boolean(..), Object::Boolean(..)) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` doesn't support `boolean` as it's operand",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),

                (Object::String(x, ..), Object::String(y, ..)) => {
                    Ok(Object::String(x + &y, Meta::default()))
                }

                (Object::Nil(..), Object::Nil(..)) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` doesn't support `nil` as it's operand",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),

                _ => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` expects same type on both side",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),
            },

            TokenType::GreaterEqual => match (left, right) {
                (Object::Number(x, ..), Object::Number(y, ..)) => {
                    Ok(Object::Boolean(x >= y, Meta::default()))
                }

                (Object::Boolean(..), Object::Boolean(..)) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` doesn't support `boolean` as it's operand",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),

                (Object::String(x, ..), Object::String(y, ..)) => {
                    Ok(Object::String(x + &y, Meta::default()))
                }

                (Object::Nil(..), Object::Nil(..)) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` doesn't support `nil` as it's operand",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),

                _ => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` expects same type on both side",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),
            },

            TokenType::Less => match (left, right) {
                (Object::Number(x, ..), Object::Number(y, ..)) => {
                    Ok(Object::Boolean(x < y, Meta::default()))
                }

                (Object::Boolean(..), Object::Boolean(..)) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` doesn't support `boolean` as it's operand",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),

                (Object::String(x, ..), Object::String(y, ..)) => {
                    Ok(Object::String(x + &y, Meta::default()))
                }

                (Object::Nil(..), Object::Nil(..)) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` doesn't support `nil` as it's operand",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),

                _ => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` expects same type on both side",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),
            },

            TokenType::LessEqual => match (left, right) {
                (Object::Number(x, ..), Object::Number(y, ..)) => {
                    Ok(Object::Boolean(x <= y, Meta::default()))
                }

                (Object::Boolean(..), Object::Boolean(..)) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` doesn't support `boolean` as it's operand",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),

                (Object::String(x, ..), Object::String(y, ..)) => {
                    Ok(Object::String(x + &y, Meta::default()))
                }

                (Object::Nil(..), Object::Nil(..)) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` doesn't support `nil` as it's operand",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),

                _ => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` expects same type on both side",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),
            },

            TokenType::Plus => match (left, right) {
                (Object::Number(x, ..), Object::Number(y, ..)) => {
                    Ok(Object::Number(x + y, Meta::default()))
                }

                (Object::Boolean(..), Object::Boolean(..)) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` doesn't support `boolean` as it's operand",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),

                (Object::String(x, ..), Object::String(y, ..)) => {
                    Ok(Object::String(x + &y, Meta::default()))
                }

                (Object::Nil(..), Object::Nil(..)) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` doesn't support `nil` as it's operand",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),

                _ => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` expects same type on both side",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),
            },

            TokenType::Minus => match (left, right) {
                (Object::Number(x, ..), Object::Number(y, ..)) => {
                    Ok(Object::Number(x - y, Meta::default()))
                }

                (Object::Boolean(..), Object::Boolean(..)) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` doesn't support `boolean` as it's operand",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),

                (Object::String(..), Object::String(..)) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` doesn't support `string` as it's operand",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),

                (Object::Nil(..), Object::Nil(..)) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` doesn't support `nil` as it's operand",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),

                _ => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` expects same type on both side",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),
            },

            TokenType::Star => match (left, right) {
                (Object::Number(x, ..), Object::Number(y, ..)) => {
                    Ok(Object::Number(x * y, Meta::default()))
                }

                (Object::Boolean(..), Object::Boolean(..)) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` doesn't support `boolean` as it's operand",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),

                (Object::String(..), Object::String(..)) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` doesn't support `string` as it's operand",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),

                (Object::Nil(..), Object::Nil(..)) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` doesn't support `nil` as it's operand",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),

                _ => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` expects same type on both side",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),
            },

            TokenType::Slash => match (left, right) {
                (Object::Number(x, ..), Object::Number(y, ..)) => {
                    Ok(Object::Number(x / y, Meta::default()))
                }

                (Object::Boolean(..), Object::Boolean(..)) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` doesn't support `boolean` as it's operand",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),

                (Object::String(..), Object::String(..)) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` doesn't support `string` as it's operand",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),

                (Object::Nil(..), Object::Nil(..)) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` doesn't support `nil` as it's operand",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),

                _ => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` expects same type on both side",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),
            },

            TokenType::Modulo => match (left, right) {
                (Object::Number(x, ..), Object::Number(y, ..)) => {
                    Ok(Object::Number(x % y, Meta::default()))
                }

                (Object::String(..), Object::String(..)) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` doesn't support `string` as it's operand",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),

                (Object::Nil(..), Object::Nil(..)) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` doesn't support `nil` as it's operand",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),

                _ => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` expects same type on both side",
                        binary_expression.operator.lexeme
                    ),
                    binary_expression.operator.position,
                )),
            },

            _ => Err(Error::new(
                ErrorType::RuntimeError,
                format!(
                    "`{}` is not a binary operator.",
                    binary_expression.operator.lexeme
                ),
                binary_expression.operator.position,
            )),
        }
    }

    fn evaluate_unary_expression(
        &mut self,
        unary_expression: UnaryExpression,
    ) -> Result<Object, Error> {
        let right = self.match_expression(*unary_expression.right)?;

        match unary_expression.operator.ttype {
            TokenType::Not => Ok(Object::Boolean(!right.is_true(), Meta::default())),

            TokenType::Minus => match right {
                Object::Number(x, ..) => Ok(Object::Number(x * -1., Meta::default())),

                Object::Boolean(..) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` does not support `boolean` as it's operand",
                        unary_expression.operator.lexeme
                    ),
                    unary_expression.operator.position,
                )),

                Object::String(..) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` does not support `string` as it's operand",
                        unary_expression.operator.lexeme
                    ),
                    unary_expression.operator.position,
                )),

                Object::Nil(..) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` does not support `nil` as it's operand",
                        unary_expression.operator.lexeme
                    ),
                    unary_expression.operator.position,
                )),

                Object::Array(..) => Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "Type mismatch, `{}` does not support `array` as it's operand",
                        unary_expression.operator.lexeme
                    ),
                    unary_expression.operator.position,
                )),
            },

            _ => Err(Error::new(
                ErrorType::RuntimeError,
                format!(
                    "`{}` is not a unary operator.",
                    unary_expression.operator.lexeme
                ),
                unary_expression.operator.position,
            )),
        }
    }

    fn evaluate_group_expression(
        &mut self,
        group_expression: GroupExpression,
    ) -> Result<Object, Error> {
        let value = self.evaluate_expression(*group_expression.child)?;
        Ok(value)
    }

    fn evaluate_call_expression(
        &mut self,
        call_expression: CallExpression,
    ) -> Result<Object, Error> {
        let function_statement = self.functions.get(call_expression.identifier.clone())?;
        let paramiters = function_statement.paramiters.clone();
        let arguments_length = call_expression.arguments.len();
        match arguments_length.cmp(&paramiters.len()) {
            Ordering::Less => {
                return Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "The `{}` expected {} arguments but got {}. Missing arguments are {}",
                        call_expression.identifier.lexeme,
                        paramiters.len(),
                        arguments_length,
                        paramiters[arguments_length..]
                            .iter()
                            .map(|p| format!("`{}`", p.identifier.lexeme))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    call_expression.identifier.position,
                ))
            }
            Ordering::Greater => {
                return Err(Error::new(
                    ErrorType::RuntimeError,
                    format!(
                        "too many arguments passed to `{}`. Expected {} but got {}",
                        call_expression.identifier.lexeme,
                        paramiters.len(),
                        arguments_length
                    ),
                    call_expression.identifier.position,
                ))
            }
            Ordering::Equal => {
                self.execute_function_statement(call_expression.arguments, function_statement)
            }
        }
    }

    fn evaluate_identifier_expression(
        &self,
        identifier_expression: IdentifierExpression,
    ) -> Result<Object, Error> {
        self.variables.get(identifier_expression.identifier)
    }

    fn match_expression(&mut self, expression: Expression) -> Result<Object, Error> {
        match expression {
            Expression::Binary(binary_expression) => {
                self.evaluate_binary_expression(binary_expression)
            }

            Expression::Unary(unary_expression) => self.evaluate_unary_expression(unary_expression),

            Expression::Group(group_expression) => self.evaluate_group_expression(group_expression),

            Expression::Call(call_expression) => self.evaluate_call_expression(call_expression),

            Expression::Identifier(identifier_expression) => {
                Ok(self.evaluate_identifier_expression(identifier_expression)?)
            }

            Expression::Block(block_expression) => self.evaluate_block_expression(block_expression),

            Expression::If(if_expression) => self.evaluate_if_expression(if_expression),

            Expression::Literal(literal_expression) => {
                if let Some(object) = literal_expression.object.literal {
                    Ok(object)
                } else {
                    Ok(Object::Nil(Meta::default()))
                }
            }

            Expression::Array(array_expression) => {
                let mut objects = Vec::new();
                for object in array_expression.objects {
                    if let Some(object) = object.literal {
                        objects.push(object)
                    } else if matches!(object.ttype, TokenType::Identifier) {
                        objects.push(self.variables.get(object)?)
                    }
                }
                Ok(Object::Array(objects, Meta::default()))
            }
        }
    }
}

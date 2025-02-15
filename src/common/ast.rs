use super::token::Token;

pub type Program = Vec<Statement>;

#[derive(Debug, Clone)]
pub enum Statement {
    Let(LetStatement),
    Assignment(AssignmentStatement),
    Function(FunctionStatement),
    Return(Expression),
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub enum ElseBlock {
    Block(BlockExpression),
    If(IfExpression),
}

#[derive(Debug, Clone)]
pub struct LetStatement {
    pub identifier: Token,
    pub expression: Expression,
}

impl LetStatement {
    pub fn new(identifier: Token, expression: Expression) -> Self {
        Self {
            identifier,
            expression,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssignmentStatement {
    pub identifier: Token,
    pub expression: Expression,
}

impl AssignmentStatement {
    pub fn new(identifier: Token, expression: Expression) -> Self {
        Self {
            identifier,
            expression,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockExpression {
    pub statements: Box<Vec<Statement>>,
}

impl BlockExpression {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self {
            statements: Box::new(statements),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IfExpression {
    pub condition: Box<Expression>,
    pub if_block: BlockExpression,
    pub else_block: Box<Option<ElseBlock>>,
}

impl IfExpression {
    pub fn new(
        condition: Expression,
        if_block: BlockExpression,
        else_block: Option<ElseBlock>,
    ) -> Self {
        Self {
            condition: Box::new(condition),
            if_block,
            else_block: Box::new(else_block),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub identifier: Token,
    pub is_pack: bool,
}

impl Parameter {
    pub fn new(identifier: Token, is_pack: bool) -> Self {
        Self {
            identifier,
            is_pack,
        }
    }
}

/// A function statement. The block will be None if the function is a builtin function.
#[derive(Debug, Clone)]
pub struct FunctionStatement {
    pub identifier: Token,
    pub paramiters: Vec<Parameter>,
    pub block: Option<BlockExpression>,
}

impl FunctionStatement {
    pub fn new(
        identifier: Token,
        paramiters: Vec<Parameter>,
        block: Option<BlockExpression>,
    ) -> Self {
        Self {
            identifier,
            paramiters,
            block,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Block(BlockExpression),
    If(IfExpression),
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Group(GroupExpression),
    Call(CallExpression),
    Identifier(IdentifierExpression),
    Literal(LiteralExpression),
    Array(ArrayExpression),
}

#[derive(Debug, Clone)]
pub struct BinaryExpression {
    pub left: Box<Expression>,
    pub operator: Token,
    pub right: Box<Expression>,
}

impl BinaryExpression {
    pub fn new(left: Expression, operator: Token, right: Expression) -> Self {
        Self {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnaryExpression {
    pub operator: Token,
    pub right: Box<Expression>,
}

impl UnaryExpression {
    pub fn new(operator: Token, right: Expression) -> Self {
        Self {
            operator,
            right: Box::new(right),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GroupExpression {
    pub child: Box<Expression>,
}

impl GroupExpression {
    pub fn new(child: Expression) -> Self {
        Self {
            child: Box::new(child),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CallExpression {
    pub identifier: Token,
    pub arguments: Vec<Expression>,
}

impl CallExpression {
    pub fn new(identifier: Token, arguments: Vec<Expression>) -> Self {
        Self {
            identifier,
            arguments,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IdentifierExpression {
    pub identifier: Token,
}

impl IdentifierExpression {
    pub fn new(identifier: Token) -> Self {
        Self { identifier }
    }
}

#[derive(Debug, Clone)]
pub struct LiteralExpression {
    pub object: Token,
}

impl LiteralExpression {
    pub fn new(object: Token) -> Self {
        Self { object }
    }
}

#[derive(Debug, Clone)]
pub struct ArrayExpression {
    pub objects: Vec<Token>,
}

impl ArrayExpression {
    pub fn new(objects: Vec<Token>) -> Self {
        Self { objects }
    }
}

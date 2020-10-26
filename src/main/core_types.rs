use std::collections::HashMap;

#[derive(PartialEq, Debug, Clone)]
pub enum BuiltinFunction {
    Equals,
    Plus,
    Minus,
    Times,
    First,
    Println,
}

pub type Environment = HashMap<String, Expression>;

#[derive(PartialEq, Debug, Clone)]
pub enum Frame {
    Start(Environment, Expression),
    AddToEnv(Environment, String),
    PushBranch(Environment, Expression, Expression),
    EvalFn(Environment, Vec<Expression>),
    EvalArgs(Environment, Expression, Vec<Expression>, Vec<Expression>),
    Stop(Environment, Expression),
}

pub type Stack = Vec<Frame>;

#[derive(PartialEq, Debug, Clone)]
pub enum Expression {
    True,
    False,
    Number(i32),
    Symbol(String),
    List(Vec<Expression>),
    Function(BuiltinFunction),
    Lambda(Environment, Vec<String>, Box<Expression>), // Why did I have the ref(env)?
    Continuation(Stack),
}

pub use Expression::*;

pub enum ReadResult {
    EndOfTokens(Vec<Expression>),
    EndOfExpression(Vec<Expression>, Vec<String>)
}

pub use ReadResult::*;

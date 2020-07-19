#![allow(dead_code)]

use ferris_says::say; // from the previous step
use std::io::{stdout, BufWriter};
use std::collections::HashMap;
use regex::Regex;

#[derive(PartialEq, Debug)]
enum BuiltinFunction {
    Equals,
    Plus,
    Minus,
    Times,
    First,
    Println,
}

type Environment = HashMap<String, Expression>;

enum Frame {
    Start(Environment, Expression),
    AddToEnv(Environment, String),
    PushBranch(Environment, Expression, Expression),
    EvalFn(Environment, Vec<Expression>),
    EvalArgs(Environment, Expression, Vec<Expression>, Vec<Expression>),
    Stop(Environment, Expression),
}

type Stack = Vec<Frame>;

enum Expression {
    True,
    False,
    Number(i32),
    Symbol(String), // Should this be &str instead?
    List(Vec<Expression>),
    Function(BuiltinFunction),
    Lambda(Environment, Vec<String>, Box<Expression>), // Why did I have the ref(env)?
    Continuation(Stack),
}

enum ReadResult {
    EndOfTokens(Vec<Expression>),
    EndOfExpression(Vec<Expression>, Vec<String>)
}

// exception ArgumentError(string);

// exception UnbalancedParens;

fn tokenize(before: &str) -> Vec<String> {
    // Should move these out of the function somewhere
    let parens = Regex::new(r"(?P<p>[\(\)])").unwrap();
    let whitespace = Regex::new(r"[ \n]+").unwrap();

    let after = parens.replace_all(before, " $p ");
    let tokens = whitespace
        .split(&after)
        .map(|s| s.to_string())
        .filter(|s| s != "")
        .collect();

    return tokens;
}

fn main() {
    let stdout  = stdout();
    let message = String::from("Hello fellow Rustaceans!");
    let width   = message.chars().count();

    let mut writer = BufWriter::new(stdout.lock());
    say(message.as_bytes(), width, &mut writer).unwrap();

    assert_eq!(BuiltinFunction::Equals, BuiltinFunction::Equals);
    assert_eq!(vec!["(", "hello", ")"], tokenize("(hello)"))
}

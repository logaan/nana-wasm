#![allow(dead_code)]

use ferris_says::say; // from the previous step
use std::io::{stdout, BufWriter};
use std::collections::HashMap;
use regex::Regex;

#[derive(PartialEq, Debug, Clone)]
enum BuiltinFunction {
    Equals,
    Plus,
    Minus,
    Times,
    First,
    Println,
}

type Environment = HashMap<String, Expression>;

#[derive(PartialEq, Debug, Clone)]
enum Frame {
    Start(Environment, Expression),
    AddToEnv(Environment, String),
    PushBranch(Environment, Expression, Expression),
    EvalFn(Environment, Vec<Expression>),
    EvalArgs(Environment, Expression, Vec<Expression>, Vec<Expression>),
    Stop(Environment, Expression),
}

type Stack = Vec<Frame>;

#[derive(PartialEq, Debug, Clone)]
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

fn tokenize(code: String) -> Vec<String> {
    // Should move these out of the function somewhere
    let parens = Regex::new(r"(?P<p>[\(\)])").unwrap();
    let whitespace = Regex::new(r"[ \n]+").unwrap();
    let spacious_code = parens.replace_all(&code, " $p ");
    let tokens = whitespace
        .split(&spacious_code)
        .filter(|s| !s.is_empty() )
        .map(|s| s.to_string())
        .collect();

    return tokens;
}

fn is_number(str: &str) -> bool {
    let digits = Regex::new(r"^[0-9]*$").unwrap();
    return digits.is_match(str);
}

// fn read(expressions: &mut Vec<Expression>, tokens: Vec<&str>) -> ReadResult {
//     return match tokens[..] {
//         [] => {
//             // So rather than mutating expressions should we be cloning and then reversing?
//             // Or returning a mutable something in EndOfTokens?
//             expressions.reverse();
//             return ReadResult::EndOfTokens(expressions.to_vec());
//         },
//         ["(", ..] => {
//             ReadResult::EndOfExpression(expressions.to_vec(), tokens)
//         }
//         // This is just to make the types stop complaining, should be deleted
//         _ => ReadResult::EndOfExpression(expressions.to_vec(), tokens)
//     };
// }

fn main() {
    let stdout  = stdout();
    let message = String::from("Hello fellow Rustaceans!");
    let width   = message.chars().count();

    let mut writer = BufWriter::new(stdout.lock());
    say(message.as_bytes(), width, &mut writer).unwrap();

    assert_eq!(BuiltinFunction::Equals, BuiltinFunction::Equals);
    let greeting = "(hello)".to_string();
    assert_eq!(vec!["(", "hello", ")"], tokenize(greeting))
}

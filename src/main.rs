#![allow(dead_code)]

use ferris_says::say;
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
    Symbol(String),
    List(Vec<Expression>),
    Function(BuiltinFunction),
    Lambda(Environment, Vec<String>, Box<Expression>), // Why did I have the ref(env)?
    Continuation(Stack),
}

enum ReadResult {
    EndOfTokens(Vec<Expression>),
    EndOfExpression(Vec<Expression>, Vec<String>)
}

fn tokenize(code: &str) -> Vec<String> {
    let parens = Regex::new(r"(?P<p>[\(\)])").unwrap();
    let whitespace = Regex::new(r"[ \n]+").unwrap();
    let spacious_code = parens.replace_all(code, " $p ");
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

fn read(mut expressions: Vec<Expression>, mut tokens: Vec<String>) -> ReadResult {
    let head = tokens.pop();

    match head {
        None => ReadResult::EndOfTokens(expressions),
        Some(head) => {
            match &head[..] {
                "(" => {
                    match read(vec![], tokens) {
                        ReadResult::EndOfTokens(_) => panic!("Unbalanced parens"),
                        ReadResult::EndOfExpression(nested_expressions, new_tokens) => {
                            expressions.push(Expression::List(nested_expressions));
                            return read(expressions, new_tokens)
                        }
                    }
                },
                ")" => return ReadResult::EndOfExpression(expressions, tokens),
                "true" => expressions.push(Expression::True),
                "false" => expressions.push(Expression::False),
                n if is_number(n) => {
                    let head_as_int: i32 = n.parse().unwrap();
                    expressions.push(Expression::Number(head_as_int));
                },
                _ => expressions.push(Expression::Symbol(head))
            }

            read(expressions, tokens)
        }
    }
}

fn read_tokens(mut tokens: Vec<String>) -> Vec<Expression> {
    tokens.reverse();

    match read(vec![], tokens) {
        ReadResult::EndOfTokens(result) => result,
        ReadResult::EndOfExpression(_, _) => panic!("Unbalanced parens")
    }
}

fn parse(code: &str) -> Vec<Expression> {
    read_tokens(tokenize(&code))
}

fn main() {
    let stdout  = stdout();
    let message = String::from("Hello fellow Rustaceans!");
    let width   = message.chars().count();

    let mut writer = BufWriter::new(stdout.lock());
    say(message.as_bytes(), width, &mut writer).unwrap();

    assert_eq!(BuiltinFunction::Equals, BuiltinFunction::Equals);
    assert_eq!(vec!["(", "hello", ")"], tokenize(&"(hello)"));

    assert_eq!(vec![Expression::True], parse("true"));
    assert_eq!(vec![Expression::False], parse("false"));
    assert_eq!(vec![Expression::Symbol("potato".to_string())], parse("potato"));
    assert_eq!(vec![Expression::Number(1)], parse("1"));
    assert_eq!(
        vec![
            Expression::List(vec![
                Expression::Symbol("+".to_string()),
                Expression::Number(1),
                Expression::Number(2),
            ])
        ],
        parse("(+ 1 2)")
    );

    assert_eq!(
        vec![
            Expression::List(vec![
                Expression::Symbol("+".to_string()),
                Expression::Number(1),
                Expression::Number(2),
                Expression::List(vec![
                    Expression::Symbol("+".to_string()),
                    Expression::Number(3),
                    Expression::Number(4),
                ]),
            ])
        ],
        parse("(+ 1 2 (+ 3 4))")
    );
}

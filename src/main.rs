#![allow(dead_code)]
#![allow(unused_imports)]

use ferris_says::say;
use std::io::{stdout, BufWriter};
use regex::Regex;

mod standard_library;
use standard_library::builtin_apply;

mod core_types;
use core_types::*;

use std::collections::HashMap;

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
        None => EndOfTokens(expressions),
        Some(head) => {
            match &head[..] {
                "(" => {
                    match read(vec![], tokens) {
                        EndOfTokens(_) => panic!("Unbalanced parens"),
                        EndOfExpression(nested_expressions, new_tokens) => {
                            expressions.push(List(nested_expressions));
                            return read(expressions, new_tokens)
                        }
                    }
                },
                ")" => return EndOfExpression(expressions, tokens),
                "true" => expressions.push(True),
                "false" => expressions.push(False),
                n if is_number(n) => {
                    let head_as_int: i32 = n.parse().unwrap();
                    expressions.push(Number(head_as_int));
                },
                _ => expressions.push(Symbol(head))
            }

            read(expressions, tokens)
        }
    }
}

fn read_tokens(mut tokens: Vec<String>) -> Vec<Expression> {
    tokens.reverse();

    match read(vec![], tokens) {
        EndOfTokens(result) => result,
        EndOfExpression(_, _) => panic!("Unbalanced parens")
    }
}

fn parse(code: &str) -> Vec<Expression> {
    read_tokens(tokenize(&code))
}

fn args_to_env(env: Environment, names: Vec<String>, values: Vec<Expression>) -> Environment {
    let new_env: Environment = names.into_iter().zip(values.into_iter()).collect();

    return env.into_iter().chain(new_env).collect();
}

// I wonder if I could do away with this function by using Symbols as keys in the env?
fn args_to_strings(exp: Expression) -> String {
    match exp {
        Symbol(name) => name,
        _ => panic!("All arguments must be symbols")
    }
}

fn not_special_form(word: Expression) -> bool {
    word != Symbol("def".to_string())
        && word != Symbol("if".to_string())
        && word != Symbol("quote".to_string())
        && word != Symbol("lambda".to_string())
        && word != Symbol("call/cc".to_string())
}

// TODO: Implement apply
fn apply(_env: Environment, fun: Expression, _args: Vec<Expression>, _stack: Stack) -> Stack {
    match fun {
        Function(_fun) => panic!("function application not implemented"),
        Lambda(_environment, _arg_names, _body) => panic!("args to env need to be implemented"),
        Continuation(_continuation_stack) => panic!("continuations not implemented"),
        _ => panic!("Lists must start with functions")
    }
}

// TODO: These block of functions all have to be done together. They refer to one another.
// TODO: eval_start(env: Environment, expr: Expression) -> Frame
// TODO: eval_frame(stack: Stack) -> Stack
// TODO: eval_stepper(stack: Stack) -> (Environment, Expression)

// TODO: eval(expr: Expression, env: Environment) -> Expression

// TODO: eval_expressions(env: Environment, code: String) -> (Environment, Expression)

// TODO: evalOnceOff(code: String) -> Expression


// TODO: Re-write main to be like Hello.re
fn main() {
    let stdout  = stdout();
    let message = String::from("Hello fellow Rustaceans!");
    let width   = message.chars().count();

    let mut writer = BufWriter::new(stdout.lock());
    say(message.as_bytes(), width, &mut writer).unwrap();

    assert_eq!(BuiltinFunction::Equals, BuiltinFunction::Equals);
    assert_eq!(vec!["(", "hello", ")"], tokenize(&"(hello)"));

    assert_eq!(vec![True], parse("true"));
    assert_eq!(vec![False], parse("false"));
    assert_eq!(vec![Symbol("potato".to_string())], parse("potato"));
    assert_eq!(vec![Number(1)], parse("1"));
    assert_eq!(
        vec![
            List(vec![
                Symbol("+".to_string()),
                Number(1),
                Number(2),
            ])
        ],
        parse("(+ 1 2)")
    );

    assert_eq!(
        vec![
            List(vec![
                Symbol("+".to_string()),
                Number(1),
                Number(2),
                List(vec![
                    Symbol("+".to_string()),
                    Number(3),
                    Number(4),
                ]),
            ])
        ],
        parse("(+ 1 2 (+ 3 4))")
    );
}

#![allow(dead_code)]
#![allow(unused_imports)]

use regex::Regex;
use std::env;
use std::io::{stdout, BufWriter};

mod standard_library;
use standard_library::builtin_apply;

mod core_types;
use core_types::*;
use Frame::*;

fn tokenize(code: &str) -> Vec<String> {
    let parens = Regex::new(r"(?P<p>[\(\)])").unwrap();
    let whitespace = Regex::new(r"[ \n]+").unwrap();
    let spacious_code = parens.replace_all(code, " $p ");
    let tokens = whitespace
        .split(&spacious_code)
        .filter(|s| !s.is_empty())
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
                "(" => match read(vec![], tokens) {
                    EndOfTokens(_) => panic!("Unbalanced parens"),
                    EndOfExpression(nested_expressions, new_tokens) => {
                        expressions.push(List(nested_expressions));
                        return read(expressions, new_tokens);
                    }
                },
                ")" => return EndOfExpression(expressions, tokens),
                "true" => expressions.push(True),
                "false" => expressions.push(False),
                n if is_number(n) => {
                    let head_as_int: i32 = n.parse().unwrap();
                    expressions.push(Number(head_as_int));
                }
                _ => expressions.push(Symbol(head)),
            }

            read(expressions, tokens)
        }
    }
}

fn read_tokens(mut tokens: Vec<String>) -> Vec<Expression> {
    tokens.reverse();

    match read(vec![], tokens) {
        EndOfTokens(result) => result,
        EndOfExpression(_, _) => panic!("Unbalanced parens"),
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
        _ => panic!("All arguments must be symbols"),
    }
}

fn not_special_form(word: Expression) -> bool {
    word != Symbol("def".to_string())
        && word != Symbol("if".to_string())
        && word != Symbol("quote".to_string())
        && word != Symbol("lambda".to_string())
        && word != Symbol("call/cc".to_string())
}

fn apply(env: Environment, fun: Expression, args: Vec<Expression>, mut stack: Stack) -> Stack {
    match fun {
        Function(fun) => {
            stack.push(Stop(env, builtin_apply(fun, args)));
            stack
        }
        Lambda(environment, arg_names, body) => {
            stack.push(Start(args_to_env(environment, arg_names, args), *body));
            stack
        }
        Continuation(mut continuation_stack) => {
            continuation_stack.push(Stop(env, args.into_iter().nth(0).expect("??")));
            continuation_stack
        }
        _ => panic!("Lists must start with functions"),
    }
}

fn eval_start(env: Environment, expr: Expression) -> Frame {
    match expr {
        True => Stop(env, True),
        False => Stop(env, False),
        Number(i) => Stop(env, Number(i)),
        Symbol(s) => {
            let value = env.get(&s).expect("Not found").clone();
            Stop(env, value)
        }
        // This whole block is a mess. Doesn't pattern match well.
        List(expressions) => {
            let first = &expressions[0];

            match first {
                Symbol(symbol_text) => match symbol_text.as_str() {
                    "quote" => Stop(env, expressions[1].clone()),
                    // Without good deep pattern matching this look pretty aweful.
                    "lambda" => match &expressions[1] {
                        List(args_exprs) => {
                            let args_as_strings: Vec<String> = args_exprs
                                .into_iter()
                                .map(|e| args_to_strings(e.clone()))
                                .collect();
                            Stop(
                                env.clone(),
                                Lambda(env, args_as_strings, Box::new(expressions[2].clone())),
                            )
                        }
                        _ => panic!("Lambda must be followed by arguments"),
                    },
                    _ => panic!("Lists must start with a fn"),
                },
                _ => panic!("Lists must start with a fn"),
            }
        }
        Function(_) => panic!("You can't eval a function"),
        Lambda(_, _, _) => panic!("You can't eval a lambda"),
        Continuation(_) => Stop(env, expr),
    }
}

// So... only some frames can be the root of the stack. What if we had two
// interfaces, one for things that can be at the root, and another for
// everything that can appear polymorphically after that. Then you could have
// each element point to the next one. And ensure that it's impossible to have
// nothing on the stack.
fn eval_frame(mut stack: Stack) -> Stack {
    match stack.pop().expect("Nothing on the stack.") {
        Start(env, expr) => {
            match expr.clone() {
                // This seems like a terrible way to get the first element from
                // the list
                List(mut exprs) => match exprs.remove(0) {
                    // Probably an unnecessary clone()
                    func if not_special_form(func.clone()) => {
                        // These clones might be ok... or could maybe be immutable refs?
                        stack.push(EvalFn(env.clone(), exprs));
                        stack.push(Start(env, func));
                    }
                    Symbol(name) => match name.as_str() {
                        "def" => match exprs.remove(0) {
                            Symbol(name) => {
                                let value_expr = exprs.remove(0);
                                stack.push(AddToEnv(env.clone(), name));
                                stack.push(Start(env, value_expr));
                            }
                            _ => panic!("def must be followed by a name"),
                        },
                        "if" => {
                            // I would maybe prefer to have a result here. So
                            // that I can expect with a helpful error message.
                            let conditional_expr = exprs.remove(0);
                            let then_expr = exprs.remove(0);
                            let else_expr = exprs.remove(0);
                            stack.push(PushBranch(env.clone(), then_expr, else_expr));
                            stack.push(Start(env, conditional_expr));
                        }
                        "call/cc" => {
                            let func = exprs.pop().expect("call/cc must be followed by a function");
                            stack.push(EvalFn(env.clone(), vec![Continuation(stack.clone())]));
                            stack.push(Start(env, func));
                        }
                        // These three identical cases aren't great. Can it be
                        // flattened somehow?
                        _ => stack.push(eval_start(env, expr)),
                    },
                    _ => stack.push(eval_start(env, expr)),
                },
                _ => stack.push(eval_start(env, expr)),
            }

            stack
        }

        Stop(stop_env, expr) => {
            let next = stack.pop().expect("Don't know how to handle this stop.");

            // Cloning here to avoid move issues when inserting into lambda env
            match (expr.clone(), next) {
                (True, PushBranch(env, then_expr, _else_expr)) => stack.push(Start(env, then_expr)),
                (False, PushBranch(env, _then_expr, else_expr)) => {
                    stack.push(Start(env, else_expr))
                }
                (Lambda(mut env_ref, args, body), AddToEnv(env, name)) => {
                    let mut new_env = env.clone();
                    // mutating the lambda's expression to add it.
                    // was a ref in reason.
                    // This is the second clone of expr.. feels extra bad.
                    env_ref.insert(name.clone(), expr.clone());
                    let lambda_with_self_reference = Lambda(env_ref, args, body);
                    new_env.insert(name, lambda_with_self_reference.clone());
                    stack.push(Stop(new_env, lambda_with_self_reference));
                }
                (result, AddToEnv(env, name)) => {
                    let mut new_env = env.clone();
                    new_env.insert(name, result.clone());
                    stack.push(Stop(new_env, result));
                }
                (result, EvalFn(env, arg_exprs)) => {
                    stack.push(EvalArgs(env, result, vec![], arg_exprs))
                }
                (result, EvalArgs(env, fun, mut evaluated, unevaluated)) => {
                    evaluated.push(result);
                    stack.push(EvalArgs(env, fun, evaluated, unevaluated));
                }
                (_result, Start(_, expr)) => stack.push(Start(stop_env, expr)),
                (_, PushBranch(_, _, _)) => panic!("if condition evaluated to non-boolean"),
                _result => panic!("Don't know how to handle this stop"),
            }

            stack
        }

        EvalArgs(env, fun, evaluated, mut unevaluated) => match unevaluated.pop() {
            Some(next) => {
                stack.push(EvalArgs(env.clone(), fun, evaluated, unevaluated));
                stack.push(Start(env, next));
                stack
            }
            // Maybe evaluated should be reversed
            None => apply(env, fun, evaluated, stack),
        },

        PushBranch(_, _, _) => panic!("PushBranch should never appear int he head of the stack."),
        AddToEnv(_, _) => panic!("AddToEnv should never appear int he head of the stack."),
        EvalFn(_, _) => panic!("EvalFn should never appear int he head of the stack."),
    }
}

fn eval_stepper(stack: Stack) -> (Environment, Expression) {
    if !env::var("VERBOSE").is_err() {
        println!("{:?}", stack);
    }

    match stack.len() {
        0 => panic!("Nothing on the stack."),
        1 => match stack.last().expect("literally unreachable") {
            // These are both references.... hmm. Cloning here feels super
            // unnecessary.
            Stop(env, expr) => (env.clone(), expr.clone()),
            _frame => eval_stepper(eval_frame(stack)),
        },
        _ => eval_stepper(eval_frame(stack)),
    }
}

// TODO: fn eval(expr: Expression, env: Environment) -> Expression

fn eval_expressions(env: Environment, code: String) -> (Environment, Expression) {
    // Some bad clones here
    let mut stack: Stack = parse(code.as_str())
        .iter()
        .map(|expression| Start(env.clone(), expression.clone()))
        .collect();

    stack.reverse();

    eval_stepper(stack)
}

fn eval_once_off(code: &str) -> Expression {
    let (_, result) = eval_expressions(standard_library::environment(), code.to_string());
    result
}

// TODO: Re-write main to be like Hello.re
fn main() {
    assert_eq!(BuiltinFunction::Equals, BuiltinFunction::Equals);
    assert_eq!(vec!["(", "hello", ")"], tokenize(&"(hello)"));

    assert_eq!(vec![True], parse("true"));
    assert_eq!(vec![False], parse("false"));
    assert_eq!(vec![Symbol("potato".to_string())], parse("potato"));
    assert_eq!(vec![Number(1)], parse("1"));
    assert_eq!(
        vec![List(vec![Symbol("+".to_string()), Number(1), Number(2),])],
        parse("(+ 1 2)")
    );

    assert_eq!(
        vec![List(vec![
            Symbol("+".to_string()),
            Number(1),
            Number(2),
            List(vec![Symbol("+".to_string()), Number(3), Number(4),]),
        ])],
        parse("(+ 1 2 (+ 3 4))")
    );

    assert_eq!(True, eval_once_off("true"));
    assert_eq!(False, eval_once_off("false"));
    assert_eq!(False, eval_once_off("true false"));
    assert_eq!(Number(1), eval_once_off("1"));
    assert_eq!(Number(1), eval_once_off("(if true 1 2)"));
    assert_eq!(Number(3), eval_once_off("(+ 1 2)"));
    assert_eq!(Number(1), eval_once_off("(first (quote (1 2 3)))"));
    assert_eq!(
        Symbol("foo".to_string()),
        eval_once_off("(println (quote foo))")
    );
    assert_eq!(Number(9), eval_once_off("((lambda (n) (* n n)) 3)"));
    assert_eq!(Number(4), eval_once_off("(def a 4) a"));
    assert_eq!(
        Number(9),
        eval_once_off("(def square (lambda (n) (* n n))) (square 3)")
    );
    assert_eq!(
        Symbol("done".to_string()),
        eval_once_off(
            "
(def repeat-once
  (lambda (should-repeat?)
    (if should-repeat?
      (repeat-once false)
      (quote done))))

(repeat-once true)
"
        )
    );
}

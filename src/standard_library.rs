// I think this is done. Seems to match StandardLibrary.re

use crate::core_types::*;
use std::collections::HashMap;
use BuiltinFunction::*;

pub fn builtin_apply(func: BuiltinFunction, args: Vec<Expression>) -> Expression {
    match (func, &args[..]) {
        (Equals, [a, b]) => {
            if a == b {
                True
            } else {
                False
            }
        }
        (Plus, [Number(a), Number(b)]) => Number(a + b),
        (Minus, [Number(a), Number(b)]) => Number(a - b),
        (Times, [Number(a), Number(b)]) => Number(a * b),
        // Is there a way of avoiding cloning here?
        (First, [List(v)]) => v.first().expect("Called first on an empty list").clone(),
        // There's got to be a way to avoid clone here. Implement Copy? Or Move?
        (Println, [value]) => {
            println!("{:?}", value);
            value.clone()
        }
        _ => panic!("ArgumentError on builtin function"),
    }
}

pub fn environment() -> Environment {
    let mut env = HashMap::new();

    env.insert("true".to_string(), True);
    env.insert("false".to_string(), False);
    env.insert("=".to_string(), Function(Equals));
    env.insert("+".to_string(), Function(Plus));
    env.insert("-".to_string(), Function(Minus));
    env.insert("*".to_string(), Function(Times));
    env.insert("first".to_string(), Function(First));
    env.insert("println".to_string(), Function(Println));

    env
}

use crate::core_types::*;
use BuiltinFunction::*;

pub fn builtin_apply(func: BuiltinFunction, args: Vec<Expression>) -> Expression {
    match (func, &args[..]) {
        (Equals, [a, b]) => if a == b {True} else {False} ,
        (Plus, [Number(a), Number(b)]) => Number(a + b),
        (Minus, [Number(a), Number(b)]) => Number(a - b),
        (Times, [Number(a), Number(b)]) => Number(a * b),
        // Is there a way of avoiding cloning here?
        (First, [List(v)]) => v.first().expect("Called first on an empty list").clone(),
        // There's got to be a way to avoid clone here. Implement Copy? Or Move?
        (Println, [value]) => { println!("{:?}", value); value.clone()},
        _ => panic!("ArgumentError on builtin function")
    }
}

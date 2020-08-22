use crate::core_types::*;

pub fn builtin_apply(func: BuiltinFunction, args: Vec<Expression>) -> Expression {
    match (func, &args[..]) {
        (BuiltinFunction::Equals, [a, b]) => {
            if a == b {True}
            else {False}
        },
        _ => panic!("ArgumentError on builtin function")
    }
}

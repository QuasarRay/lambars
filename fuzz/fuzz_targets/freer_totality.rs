#![no_main]

use std::any::Any;

use lambars::control::{Freer, InterpretError};
use libfuzzer_sys::fuzz_target;

#[derive(Clone, Copy)]
enum Op {
    Value,
}

fuzz_target!(|data: &[u8]| {
    let selector = data.first().copied().unwrap_or_default();
    let value = data.get(1).copied().unwrap_or_default();

    let program = Freer::<Op, ()>::lift_instruction(Op::Value, |result| {
        *result.downcast::<u8>().expect("Op::Value expects u8")
    })
    .map(|x| x.wrapping_add(1));

    let result = program.try_interpret(|_| -> Box<dyn Any> {
        if selector & 1 == 0 {
            Box::new(value)
        } else {
            Box::new("wrong type")
        }
    });

    if selector & 1 == 0 {
        assert_eq!(result, Ok(value.wrapping_add(1)));
    } else {
        assert!(matches!(
            result,
            Err(InterpretError::ContinuationPanicked)
                | Err(InterpretError::TypeMismatch { .. })
        ));
    }
});

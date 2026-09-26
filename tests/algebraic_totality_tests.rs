#![cfg(feature = "control")]

use lambars::effect::algebraic::{
    AlgebraicError, Eff, ErrorEffect, ErrorHandler, Handler, NoEffect, OperationTag,
    PureHandler, ReaderEffect, ReaderHandler, StateEffect, StateHandler, WriterEffect,
    WriterHandler,
};

fn all_standard_handlers_fail_closed() -> bool {
    let no_effect = Eff::<NoEffect, i32>::perform_raw::<i32>(OperationTag::new(999), ());
    let pure_ok = matches!(
        PureHandler.run(no_effect),
        Err(AlgebraicError::UnknownOperation {
            effect: "NoEffect",
            ..
        })
    );

    let reader_wrong_result =
        Eff::<ReaderEffect<i32>, String>::perform_raw::<String>(OperationTag::new(1), ());
    let reader_ok = ReaderHandler::new(42).run(reader_wrong_result)
        == Err(AlgebraicError::TypeMismatch {
            context: "Eff::perform_raw result",
        });

    let state_wrong_argument =
        Eff::<StateEffect<i32>, ()>::perform_raw::<()>(OperationTag::new(11), "wrong");
    let state_ok = StateHandler::new(0).run(state_wrong_argument)
        == Err(AlgebraicError::TypeMismatch {
            context: "State::put argument",
        });

    let writer_wrong_argument =
        Eff::<WriterEffect<String>, ()>::perform_raw::<()>(OperationTag::new(20), 7_i32);
    let writer_ok = WriterHandler::<String>::new().run(writer_wrong_argument)
        == Err(AlgebraicError::TypeMismatch {
            context: "Writer::tell argument",
        });

    let error_wrong_argument =
        Eff::<ErrorEffect<String>, i32>::perform_raw::<i32>(OperationTag::new(30), 7_i32);
    let error_ok = ErrorHandler::<String>::new().run(error_wrong_argument)
        == Err(AlgebraicError::TypeMismatch {
            context: "Error::throw argument",
        });

    let continuation = Eff::<NoEffect, i32>::pure(1).fmap(|_| -> i32 {
        panic!("captured continuation panic");
    });
    let continuation_ok =
        PureHandler.run(continuation) == Err(AlgebraicError::ContinuationPanicked);

    pure_ok && reader_ok && state_ok && writer_ok && error_ok && continuation_ok
}

#[test]
fn algebraic_totality_holds_on_calling_thread() {
    assert!(all_standard_handlers_fail_closed());
}

#[test]
fn algebraic_totality_holds_when_moved_to_os_thread() {
    assert!(std::thread::spawn(all_standard_handlers_fail_closed)
        .join()
        .expect("test worker should join"));
}

#[cfg(feature = "rayon")]
#[test]
fn algebraic_totality_holds_in_rayon_parallel_tasks() {
    let (left, right) = rayon::join(
        all_standard_handlers_fail_closed,
        all_standard_handlers_fail_closed,
    );
    assert!(left);
    assert!(right);
}

#[cfg(feature = "async")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn algebraic_totality_holds_in_async_tasks() {
    let left = tokio::spawn(async { all_standard_handlers_fail_closed() });
    let right = tokio::spawn(async { all_standard_handlers_fail_closed() });

    assert!(left.await.expect("left task should join"));
    assert!(right.await.expect("right task should join"));
}

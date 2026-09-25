use lambars::control::Either;

pub fn option_bool() -> Option<bool> {
    if kani::any::<bool>() {
        Some(kani::any::<bool>())
    } else {
        None
    }
}

pub fn result_bool() -> Result<bool, bool> {
    if kani::any::<bool>() {
        Ok(kani::any::<bool>())
    } else {
        Err(kani::any::<bool>())
    }
}

pub fn either_bool() -> Either<bool, bool> {
    if kani::any::<bool>() {
        Either::Left(kani::any::<bool>())
    } else {
        Either::Right(kani::any::<bool>())
    }
}

pub fn option_option_bool() -> Option<Option<bool>> {
    if kani::any::<bool>() {
        Some(option_bool())
    } else {
        None
    }
}

pub fn result_result_bool() -> Result<Result<bool, bool>, bool> {
    if kani::any::<bool>() {
        Ok(result_bool())
    } else {
        Err(kani::any::<bool>())
    }
}

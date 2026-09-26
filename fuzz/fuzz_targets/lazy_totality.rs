#![no_main]

use lambars::control::{ConcurrentLazy, Lazy};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let value = data.first().copied().unwrap_or_default();

    let lazy = Lazy::new(move || value);
    assert_eq!(lazy.try_force().copied(), Ok(value));
    assert_eq!(lazy.try_force().copied(), Ok(value));

    let concurrent = ConcurrentLazy::new(move || value);
    assert_eq!(concurrent.try_force().copied(), Ok(value));
    assert_eq!(concurrent.try_force().copied(), Ok(value));

    let owned = Lazy::new(move || value);
    assert_eq!(owned.into_inner(), Ok(value));

    let concurrent_owned = ConcurrentLazy::new(move || value);
    assert_eq!(concurrent_owned.into_inner(), Ok(value));
});

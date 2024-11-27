use std::{
    ops::{Range, RangeInclusive},
    time::{Duration, Instant},
};

pub mod backend;
#[cfg(test)]
mod fs_test;
pub mod parser;
pub mod raw_tracks;

pub fn rlen<T: std::ops::Sub<Output = T> + Copy + std::ops::Add<usize, Output = T>>(
    r: &RangeInclusive<T>,
) -> T {
    return *r.end() - *r.start() + 1;
}
pub fn rcontains<
    T: std::ops::Sub<Output = T> + Copy + std::ops::Add<usize, Output = T> + std::cmp::PartialOrd<T>,
>(
    r: &Range<T>,
    elem: T,
) -> bool {
    elem >= r.start && elem < r.end
}
pub fn ricontains<
    T: std::ops::Sub<Output = T> + Copy + std::ops::Add<usize, Output = T> + std::cmp::PartialOrd<T>,
>(
    r: &RangeInclusive<T>,
    elem: T,
) -> bool {
    return elem >= *r.start() && elem <= *r.end();
}

pub fn time<T, F: FnOnce() -> T>(f: F) -> (Duration, T) {
    let start = Instant::now();
    let val = f();
    (start.elapsed(), val)
}

pub fn collect_parse_error(x: &nom::Err<VerboseError<&str>>) -> String {
    let mut collected = String::new();
    collected += &format!(
        "{}",
        match x {
            nom::Err::Incomplete(_) => unreachable!(),
            nom::Err::Error(x) => x,
            nom::Err::Failure(x) => x,
        }
    );
    collected
}

pub const BOLD_YELLOW_FORMAT: &str = "\x1b[1;33m";
pub const GREEN_FORMAT: &str = "\x1b[32m";
pub const CLEAR_FORMAT: &str = "\x1b[0m";

use nom::error::VerboseError;
pub use nom::Err;

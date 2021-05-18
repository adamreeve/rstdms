#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timestamp {
    second_fractions: u64,
    seconds: i64,
}

impl Timestamp {
    pub fn new(seconds: i64, second_fractions: u64) -> Timestamp {
        Timestamp {
            seconds,
            second_fractions,
        }
    }
}

use chrono::{DateTime, Duration, TimeZone, Utc};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timestamp {
    second_fractions: u64,
    seconds: i64,
}

const FRACTIONS_PER_NS: u64 = 18446744073; // 2 ** 64 / 10 ** 9;

impl Timestamp {
    pub fn new(seconds: i64, second_fractions: u64) -> Timestamp {
        Timestamp {
            seconds,
            second_fractions,
        }
    }

    pub fn to_datetime(&self) -> Option<DateTime<Utc>> {
        let seconds_duration = Duration::seconds(self.seconds);
        let fractions_duration =
            Duration::nanoseconds((self.second_fractions / FRACTIONS_PER_NS) as i64);
        let epoch = Utc.ymd(1904, 1, 1).and_hms(0, 0, 0);
        epoch
            .checked_add_signed(seconds_duration)
            .and_then(|dt| dt.checked_add_signed(fractions_duration))
    }
}

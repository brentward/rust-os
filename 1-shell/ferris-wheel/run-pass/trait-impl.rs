// FIXME: Make me pass! Diff budget: 25 lines.
#[derive(Debug)]
enum Duration {
    MilliSeconds(u64),
    Seconds(u32),
    Minutes(u16)
}

impl PartialEq for Duration {
    fn eq(&self, other: &Self) -> bool {
        let self_ms = match self {
            &Duration::MilliSeconds(ms) => ms,
            &Duration::Seconds(s) => s as u64 * 1000,
            &Duration::Minutes(m) => m as u64 * 60000,
        };
        let other_ms = match other {
            &Duration::MilliSeconds(ms) => ms,
            &Duration::Seconds(s) => s as u64 * 1000,
            &Duration::Minutes(m) => m as u64 * 60000,
        };
        self_ms == other_ms
    }
}

fn MilliSeconds(ms: u64) -> Duration {
    Duration::MilliSeconds(ms)
}

fn Seconds(s: u32) -> Duration {
    Duration::Seconds(s)
}

fn Minutes(m: u16) -> Duration {
    Duration::Minutes(m)
}

fn main() {
    assert_eq!(Seconds(120), Minutes(2));
    assert_eq!(Seconds(420), Minutes(7));
    assert_eq!(MilliSeconds(420000), Minutes(7));
    assert_eq!(MilliSeconds(43000), Seconds(43));
}

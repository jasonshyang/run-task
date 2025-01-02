#[derive(Clone)]
pub enum TaskInterval {
    Micros(u64),
    Millis(u64),
    Seconds(u64),
    Minutes(u64),
}

impl TaskInterval {
    pub fn as_secs(&self) -> u64 {
        match self {
            TaskInterval::Micros(micros) => *micros / 1_000_000,
            TaskInterval::Millis(millis) => *millis / 1_000,
            TaskInterval::Seconds(secs) => *secs,
            TaskInterval::Minutes(mins) => mins * 60,
        }
    }

    pub fn as_millis(&self) -> u64 {
        match self {
            TaskInterval::Micros(micros) => *micros / 1_000,
            TaskInterval::Millis(millis) => *millis,
            TaskInterval::Seconds(secs) => *secs * 1_000,
            TaskInterval::Minutes(mins) => mins * 60 * 1_000,
        }
    }

    pub fn as_micros(&self) -> u64 {
        match self {
            TaskInterval::Micros(micros) => *micros,
            TaskInterval::Millis(millis) => millis * 1_000,
            TaskInterval::Seconds(secs) => secs * 1_000_000,
            TaskInterval::Minutes(mins) => mins * 60 * 1_000_000,
        }
    }

    pub fn as_u64(&self) -> u64 {
        match self {
            TaskInterval::Micros(micros) => *micros,
            TaskInterval::Millis(millis) => *millis,
            TaskInterval::Seconds(secs) => *secs,
            TaskInterval::Minutes(mins) => *mins,
        }
    }
}

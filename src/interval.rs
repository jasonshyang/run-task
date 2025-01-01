#[derive(Clone)]
pub enum TaskInterval {
    Seconds(u64),
    Minutes(u64),
    Hours(u64),
}

impl TaskInterval {
    pub fn as_secs(&self) -> u64 {
        match self {
            TaskInterval::Seconds(secs) => *secs,
            TaskInterval::Minutes(mins) => mins * 60,
            TaskInterval::Hours(hours) => hours * 3600,
        }
    }

    pub fn as_millis(&self) -> u64 {
        self.as_secs() * 1000
    }

    pub fn as_micros(&self) -> u64 {
        self.as_secs() * 1_000_000
    }
}

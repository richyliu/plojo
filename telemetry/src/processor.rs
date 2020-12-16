use crate::parsed::LogEntry;

/// A way to process log strokes to get some statistic
pub trait Processor {
    /// Process one additional log entry. This should happen chronologically after the previous one
    fn process(&mut self, entry: LogEntry);
}

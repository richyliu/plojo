use crate::parsed::LogEntry;

/// A way to process log strokes to get some statistic
pub trait Processor {
    /// Process log entries in order
    fn process(&mut self, entries: &[LogEntry]);
}

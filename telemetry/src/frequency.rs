use crate::parsed::Content;
use crate::parsed::{LogEntry, Stroke};
use crate::processor::Processor;
use std::collections::HashMap;

pub struct FrequencyAnalyzer {
    grams_1: HashMap<Stroke, u32>,
}

impl FrequencyAnalyzer {
    pub fn new() -> Self {
        Self {
            grams_1: HashMap::new(),
        }
    }

    /// Get list of 1 grams (stroke frequencies) with the more common at the start of the list.
    /// Only strokes that occur `thresholds` many times will be returned
    pub fn grams_1(&self, threshold: u32) -> Vec<(&Stroke, u32)> {
        let mut freqs = Vec::new();
        for (stroke, &count) in &self.grams_1 {
            if count >= threshold {
                freqs.push((stroke, count));
            }
        }

        // reverse sort
        freqs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        freqs
    }
}

impl Processor for FrequencyAnalyzer {
    /// Process an additional entry
    fn process(&mut self, entry: LogEntry) {
        // ignore commands
        if entry.content == Content::Command {
            return;
        }

        let stroke = entry.stroke;

        // ignore undo
        if stroke == "*" {
            return;
        }

        // increment the stroke's counter, or add one if it isn't in the map
        if let Some(count) = self.grams_1.get_mut(&stroke) {
            *count += 1;
        } else {
            self.grams_1.insert(stroke, 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(time: i64, stroke: &str, backspace_num: u32, text: &str) -> LogEntry {
        LogEntry {
            time,
            stroke: stroke.to_string(),
            content: Content::Replace {
                backspace_num,
                text: text.to_string(),
            },
        }
    }

    fn log_entries() -> Vec<LogEntry> {
        vec![
            entry(1607820695881, "-T", 0, "  the"),
            entry(1607820696136, "K-R", 0, " consider"),
            entry(1607820696286, "-T", 0, " the"),
            entry(1607820696540, "TO", 0, " to"),
            entry(1607820697320, "TPUL", 0, " full"),
            entry(1607820697605, "K-R", 0, " consider"),
            entry(1607820697808, "-T", 0, " the"),
        ]
    }

    #[test]
    fn test_1_gram_statistics() {
        let mut f = FrequencyAnalyzer::new();
        for e in log_entries() {
            f.process(e);
        }

        let freq = f.grams_1(2);
        assert_eq!(freq, vec![(&"-T".to_string(), 3), (&"K-R".to_string(), 2)])
    }

    #[test]
    fn test_ignore_commands_and_undo() {
        let mut f = FrequencyAnalyzer::new();
        f.process(entry(1607820695881, "*", 2, ""));
        f.process(entry(1607820695882, "*", 2, ""));
        f.process(entry(1607820695883, "*", 2, ""));
        f.process(entry(1607820695884, "*", 2, ""));
        f.process(LogEntry {
            time: 1607820697201,
            stroke: "SRO*PL".to_string(),
            content: Content::Command,
        });
        f.process(LogEntry {
            time: 1607820697202,
            stroke: "SRO*PL".to_string(),
            content: Content::Command,
        });
        f.process(LogEntry {
            time: 1607820697203,
            stroke: "SRO*PL".to_string(),
            content: Content::Command,
        });

        let freq = f.grams_1(2);
        assert_eq!(freq, vec![])
    }
}

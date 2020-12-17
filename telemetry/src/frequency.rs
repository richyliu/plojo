use crate::parsed::Content;
use crate::parsed::{LogEntry, Stroke};
use crate::processor::Processor;
use std::collections::HashMap;

pub struct FrequencyAnalyzer {
    grams_1: HashMap<Stroke, u32>,
    grams_2: HashMap<[Stroke; 2], u32>,
}

impl FrequencyAnalyzer {
    pub fn new() -> Self {
        Self {
            grams_1: HashMap::new(),
            grams_2: HashMap::new(),
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

    /// Get a list of bi-grams
    pub fn grams_2(&self, threshold: u32) -> Vec<(&[Stroke; 2], u32)> {
        let mut freqs = Vec::new();
        for (strokes, &count) in &self.grams_2 {
            if count >= threshold {
                freqs.push((strokes, count));
            }
        }

        // reverse sort
        freqs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        freqs
    }

    fn process_grams_1(&mut self, entries: &[&LogEntry]) {
        for entry in entries {
            let stroke = entry.stroke.clone();

            // increment the stroke's counter, or add one if it isn't in the map
            if let Some(count) = self.grams_1.get_mut(&stroke) {
                *count += 1;
            } else {
                self.grams_1.insert(stroke, 1);
            }
        }
    }

    fn process_grams_2(&mut self, entries: &[&LogEntry]) {
        // each stroke of the bi-gram must occur this frequently
        const THRESHOLD: u32 = 2;
        let mut prev: Option<Stroke> = None;

        for entry in entries {
            let stroke = entry.stroke.clone();

            // insert the bi-gram of the previous stroke and this stroke
            if let Some(prev) = prev {
                // only insert if both occur frequently enough on their own
                if self.grams_1.get(&prev).unwrap_or(&0) >= &THRESHOLD
                    && self.grams_1.get(&stroke).unwrap_or(&0) >= &THRESHOLD
                {
                    // insert the bi-gram into the map
                    if let Some(count) = self.grams_2.get_mut(&[prev.clone(), stroke.clone()]) {
                        *count += 1;
                    } else {
                        self.grams_2.insert([prev.clone(), stroke.clone()], 1);
                    }
                }
            }

            prev = Some(entry.stroke.clone());
        }
    }
}

impl Processor for FrequencyAnalyzer {
    /// Process a series of entries
    fn process(&mut self, entries: &[LogEntry]) {
        // ignore commands, undo stroke, and NoOp
        let cleaned: Vec<&LogEntry> = entries
            .iter()
            .filter(|l| {
                l.content != Content::NoOp && l.content != Content::Command && l.stroke != "*"
            })
            .collect();
        self.process_grams_1(&cleaned);
        self.process_grams_2(&cleaned);
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
        f.process(&log_entries());

        let freq = f.grams_1(2);
        assert_eq!(freq, vec![(&"-T".to_string(), 3), (&"K-R".to_string(), 2)])
    }

    #[test]
    fn test_ignore_commands_and_undo() {
        let mut f = FrequencyAnalyzer::new();
        f.process(&vec![
            entry(1607820695881, "*", 2, ""),
            entry(1607820695882, "*", 2, ""),
            entry(1607820695883, "*", 2, ""),
            entry(1607820695884, "*", 2, ""),
            LogEntry {
                time: 1607820697201,
                stroke: "SRO*PL".to_string(),
                content: Content::Command,
            },
            LogEntry {
                time: 1607820697202,
                stroke: "SRO*PL".to_string(),
                content: Content::Command,
            },
            LogEntry {
                time: 1607820697203,
                stroke: "SRO*PL".to_string(),
                content: Content::Command,
            },
            LogEntry {
                time: 1607820697423,
                stroke: "KPA*".to_string(),
                content: Content::NoOp,
            },
        ]);

        let freq = f.grams_1(2);
        assert_eq!(freq, vec![])
    }

    #[test]
    fn test_2_gram_statistics() {
        let mut f = FrequencyAnalyzer::new();
        f.process(&log_entries());

        let freq = f.grams_2(2);
        assert_eq!(freq, vec![(&["K-R".to_string(), "-T".to_string()], 2)])
    }
}

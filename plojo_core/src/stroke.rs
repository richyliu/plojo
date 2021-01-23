use serde::Deserialize;

/// A steno stroke. Can be a single stroke (ex: "H-L") or several strokes (ex: "H-L/WORLD")
#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize)]
pub struct Stroke(String);

impl Stroke {
    pub fn new(stroke: &str) -> Self {
        Self(String::from(stroke))
    }

    pub fn to_raw(self) -> String {
        self.0
    }

    pub fn is_undo(&self) -> bool {
        self.0.len() == 1 && self.0.clone() == "*"
    }

    pub fn is_valid(&self) -> bool {
        !self.0.is_empty()
    }
}

impl From<RawStroke> for Stroke {
    fn from(raw: RawStroke) -> Self {
        let mut stroke = String::from("");

        stroke.push_str(&raw.left_hand);
        stroke.push_str(&raw.center_left);
        if raw.star_key {
            stroke.push('*');
        } else {
            // the hyphen is only used when there are no center strokes nor star key, and there are
            // keys on the right hand
            if raw.center_left.is_empty()
                && raw.center_right.is_empty()
                && !raw.right_hand.is_empty()
            {
                stroke.push('-');
            }
        }
        stroke.push_str(&raw.center_right);
        stroke.push_str(&raw.right_hand);

        if raw.num_key {
            let number_stroke = to_number_stroke(&stroke);
            stroke = if number_stroke == stroke {
                // only add the "#" sign if the stroke is the same
                // (to distinguish it from a stroke without the number key)
                "#".to_owned() + &number_stroke
            } else {
                number_stroke
            };
        }

        Stroke::new(&stroke)
    }
}

/// Converts a stroke into a number stroke
/// The center dash ('-') will not be removed
fn to_number_stroke(stroke: &str) -> String {
    fn is_center_key(key: char) -> bool {
        ['A', 'O', 'E', 'U', '*', '-'].contains(&key)
    }

    // map keys to their corresponding number
    let mut result = String::from("");
    let mut first_half = true;
    for key in stroke.chars() {
        if is_center_key(key) {
            first_half = false;
        }

        result.push(if first_half {
            match key {
                'S' => '1',
                'T' => '2',
                'P' => '3',
                'H' => '4',
                _k => _k,
            }
        } else {
            match key {
                'A' => '5',
                'O' => '0',
                'F' => '6',
                'P' => '7',
                'L' => '8',
                'T' => '9',
                '-' => '-',
                _k => _k,
            }
        });
    }

    result
}

/// Raw stroke representation that can be converted to a stroke
#[derive(Debug, PartialEq, Default)]
pub struct RawStroke {
    pub num_key: bool,
    pub left_hand: String,
    pub center_left: String,
    pub star_key: bool,
    pub center_right: String,
    pub right_hand: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_number_stroke() {
        assert_eq!(to_number_stroke("STPH"), String::from("1234"));
        assert_eq!(to_number_stroke("P-FP"), String::from("3-67"));
        assert_eq!(to_number_stroke("-S"), String::from("-S"));
        assert_eq!(to_number_stroke("PWHO"), String::from("3W40"));
    }

    #[test]
    fn test_from_raw_stroke() {
        assert_eq!(
            Stroke::from(RawStroke {
                num_key: false,
                left_hand: "STP".to_string(),
                center_left: "".to_string(),
                star_key: true,
                center_right: "".to_string(),
                right_hand: "T".to_string(),
            }),
            Stroke::new("STP*T")
        );
        assert_eq!(
            Stroke::from(RawStroke {
                num_key: true,
                left_hand: "".to_string(),
                center_left: "".to_string(),
                star_key: false,
                center_right: "".to_string(),
                right_hand: "G".to_string(),
            }),
            Stroke::new("#-G")
        );
        assert_eq!(
            Stroke::from(RawStroke {
                num_key: false,
                left_hand: "KP".to_string(),
                center_left: "AO".to_string(),
                star_key: false,
                center_right: "EU".to_string(),
                right_hand: "DZ".to_string(),
            }),
            Stroke::new("KPAOEUDZ")
        );
    }
}

use plojo_core::Stroke;

pub trait RawStroke {
    // left hand should not include S key
    fn get_left_hand(&self) -> &String;
    fn get_right_hand(&self) -> &String;
    // only vowels A and O, if any
    fn get_center_left(&self) -> &String;
    // only vowels E and U, if any
    fn get_center_right(&self) -> &String;
    fn parse_raw(raw: &Vec<u8>) -> Self;

    // The s, star, and num keys can correspond to multiple physical keys, represented by each bool
    // in the vector. If there is only one physical key, the vector should only have 1 item
    fn get_s_keys(&self) -> &Vec<bool>;
    fn get_star_keys(&self) -> &Vec<bool>;
    fn get_num_keys(&self) -> &Vec<bool>;

    fn to_stroke(&self) -> Stroke {
        let mut stroke = String::from("");

        if self.get_s_keys().iter().any(|x| *x) {
            stroke.push('S');
        }
        stroke.push_str(&self.get_left_hand());
        stroke.push_str(&self.get_center_left());
        if self.get_star_keys().iter().any(|x| *x) {
            stroke.push('*');
        } else {
            // the hyphen is only used when there are no center strokes nor star key, and there are
            // keys on the right hand
            if self.get_center_left().is_empty()
                && self.get_center_right().is_empty()
                && !self.get_right_hand().is_empty()
            {
                stroke.push('-')
            }
        }
        stroke.push_str(&self.get_center_right());
        stroke.push_str(&self.get_right_hand());

        if self.get_num_keys().iter().any(|x| *x) {
            let mut number_stroke = String::from("");
            let new_stroke = &to_number(&stroke);
            // only add the "#" sign if the stroke is the same (to distinguish it from a stroke without number key)
            if new_stroke == &stroke {
                number_stroke.push('#');
            }
            number_stroke.push_str(new_stroke);
            return Stroke::new(&number_stroke);
        }

        if stroke == "-" {
            Stroke::new("")
        } else {
            Stroke::new(&stroke)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct RawStrokeGeminipr {
    // left hand keys excluding S key
    left_hand: String,
    // keys that could replace the `-` in the middle
    center_left: String,
    center_right: String,
    right_hand: String,
    s_keys: Vec<bool>,
    star_keys: Vec<bool>,
    num_keys: Vec<bool>,
}

// for reference
/*
const STENO_KEY_CHART: [[&str; 7]; 6] = [
    ["Fn", "#1", "#2", "#3", "#4", "#5", "#6"],
    ["S1-", "S2-", "T-", "K-", "P-", "W-", "H-"],
    ["R-", "A-", "O-", "*1", "*2", "res1", "res2"],
    ["pwr", "*3", "*4", "-E", "-U", "-F", "-R"],
    ["-P", "-B", "-L", "-G", "-T", "-S", "-D"],
    ["#7", "#8", "#9", "#A", "#B", "#C", "-Z"],
];
*/
impl RawStroke for RawStrokeGeminipr {
    /// Parse a raw byte vector into the raw stroke representation
    ///
    /// # Panics
    ///
    /// Panics if the vector passed in does not have a length of 6
    #[rustfmt::skip]
    fn parse_raw(raw: &Vec<u8>) -> Self {
        assert_eq!(raw.len(), 6);
        // checks if the most significant bit is positive
        fn msb_pos(byte: u8) -> bool {
            byte > 127
        }

        let mut bytes = raw.iter();
        let mut s_keys: Vec<bool> = vec![];
        let mut star_keys: Vec<bool> = vec![];
        let mut num_keys: Vec<bool> = vec![];
        let mut left_hand = String::from("");
        let mut center_left = String::from("");
        let mut center_right = String::from("");
        let mut right_hand = String::from("");

        // first row: number keys 1-6
        if let Some(row) = bytes.next() {
            // always discard the first bit
            let mut row = row << 1;

            // discard the next bit, which is the "Fn" key
            row = row << 1;

            // the next 6 bits are number keys
            for _ in 0..6 {
                num_keys.push(msb_pos(row));
                row = row << 1;
            }
        }

        // second row: left hand S- to H-
        if let Some(row) = bytes.next() {
            // always discard the first bit
            let mut row = row << 1;

            // first and second S- key
            s_keys.push(msb_pos(row)); row = row << 1;
            s_keys.push(msb_pos(row)); row = row << 1;

            // the next are T, K, P, W, and H
            if msb_pos(row) { left_hand.push('T'); } row = row << 1;
            if msb_pos(row) { left_hand.push('K'); } row = row << 1;
            if msb_pos(row) { left_hand.push('P'); } row = row << 1;
            if msb_pos(row) { left_hand.push('W'); } row = row << 1;
            if msb_pos(row) { left_hand.push('H'); }
        }

        // third row: R, A, O, 1*, 2*, and 2 useless keys
        if let Some(row) = bytes.next() {
            // always discard the first bit
            let mut row = row << 1;

            // R, A, and O
            if msb_pos(row) { left_hand.push('R'); } row = row << 1;
            if msb_pos(row) { center_left.push('A'); } row = row << 1;
            if msb_pos(row) { center_left.push('O'); } row = row << 1;

            // first and second star key
            star_keys.push(msb_pos(row)); row = row << 1;
            star_keys.push(msb_pos(row));
        }

        // fourth row: useless key, 3rd and 4th star, and E, U, F, R
        if let Some(row) = bytes.next() {
            // always discard the first bit
            let mut row = row << 1;

            // discard useless power key
            row = row << 1;

            // 3rd and 4th star
            star_keys.push(msb_pos(row)); row = row << 1;
            star_keys.push(msb_pos(row)); row = row << 1;

            // E, U, F, R
            if msb_pos(row) { center_right.push('E'); } row = row << 1;
            if msb_pos(row) { center_right.push('U'); } row = row << 1;
            if msb_pos(row) { right_hand.push('F'); } row = row << 1;
            if msb_pos(row) { right_hand.push('R'); }
        }

        // fifth row: P, B, L, G, T, S, D
        if let Some(row) = bytes.next() {
            // always discard the first bit
            let mut row = row << 1;

            if msb_pos(row) { right_hand.push('P'); } row = row << 1;
            if msb_pos(row) { right_hand.push('B'); } row = row << 1;
            if msb_pos(row) { right_hand.push('L'); } row = row << 1;
            if msb_pos(row) { right_hand.push('G'); } row = row << 1;
            if msb_pos(row) { right_hand.push('T'); } row = row << 1;
            if msb_pos(row) { right_hand.push('S'); } row = row << 1;
            if msb_pos(row) { right_hand.push('D'); }
        }

        // sixth row: number keys 7-9, A-C (?), and -Z key
        if let Some(row) = bytes.next() {
            // always discard the first bit
            let mut row = row << 1;

            // number keys 7-9
            for _ in 0..3 {
                num_keys.push(msb_pos(row));
                row = row << 1;
            }

            // ignore A-C number keys
            for _ in 0..3 {
                row = row << 1;
            }

            // Z key
            if msb_pos(row) { right_hand.push('Z'); }
        }

        RawStrokeGeminipr {
            left_hand,
            center_left,
            center_right,
            right_hand,
            s_keys,
            star_keys,
            num_keys,
        }
    }

    fn get_left_hand(&self) -> &String {
        &self.left_hand
    }
    fn get_right_hand(&self) -> &String {
        &self.right_hand
    }
    fn get_center_left(&self) -> &String {
        &self.center_left
    }
    fn get_center_right(&self) -> &String {
        &self.center_right
    }

    fn get_s_keys(&self) -> &Vec<bool> {
        &self.s_keys
    }
    fn get_star_keys(&self) -> &Vec<bool> {
        &self.star_keys
    }
    fn get_num_keys(&self) -> &Vec<bool> {
        &self.num_keys
    }
}

fn to_number(stroke: &str) -> String {
    fn is_center_key(key: char) -> bool {
        match key {
            'A' | 'O' | 'E' | 'U' | '*' | '-' => true,
            _ => false,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let raw = &vec![128, 48, 36, 0, 2, 0];
        let raw_state = RawStrokeGeminipr::parse_raw(raw);

        assert_eq!(
            raw_state,
            RawStrokeGeminipr {
                left_hand: String::from("T"),
                center_left: String::from("A"),
                center_right: String::from(""),
                right_hand: String::from("S"),
                s_keys: vec![false, true],
                star_keys: vec![false, true, false, false],
                num_keys: vec![false, false, false, false, false, false, false, false, false,],
            }
        );
    }

    #[test]
    fn test_basic_stroke_parsing() {
        assert_eq!(
            RawStrokeGeminipr::parse_raw(&vec![128, 48, 36, 0, 2, 0]).to_stroke(),
            Stroke::new("STA*S")
        );
        assert_eq!(
            RawStrokeGeminipr::parse_raw(&vec![160, 2, 0, 0, 32, 64]).to_stroke(),
            Stroke::new("#W-B")
        );
        assert_eq!(
            RawStrokeGeminipr::parse_raw(&vec![160, 127, 124, 63, 127, 65]).to_stroke(),
            Stroke::new("12K3W4R50*EU6R7B8G9SDZ")
        );
        assert_eq!(
            RawStrokeGeminipr::parse_raw(&vec![128, 21, 0, 0, 0, 0]).to_stroke(),
            Stroke::new("TPH")
        );
        assert_eq!(
            RawStrokeGeminipr::parse_raw(&vec![128, 0, 64, 0, 64, 0]).to_stroke(),
            Stroke::new("R-P")
        );
        assert_eq!(
            RawStrokeGeminipr::parse_raw(&vec![128, 1, 0, 2, 0, 64]).to_stroke(),
            Stroke::new("4-6")
        );
        assert_eq!(
            RawStrokeGeminipr::parse_raw(&vec![128, 1, 32, 2, 0, 64]).to_stroke(),
            Stroke::new("456")
        );
        assert_eq!(
            RawStrokeGeminipr::parse_raw(&vec![128, 68, 0, 0, 4, 64]).to_stroke(),
            Stroke::new("13-9")
        );
    }

    #[test]
    fn test_number_key_parsing() {
        assert_eq!(
            RawStrokeGeminipr::parse_raw(&vec![160, 127, 124, 63, 127, 65]).get_num_keys(),
            &vec![true, false, false, false, false, false, true, false, false,]
        );
        assert_eq!(
            RawStrokeGeminipr::parse_raw(&vec![128, 48, 36, 0, 2, 0]).get_num_keys(),
            &vec![false, false, false, false, false, false, false, false, false,]
        );
    }
}

use plojo_core::{RawStroke, Stroke};

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

/// Parse a raw byte vector into a stroke
///
/// # Panics
///
/// Panics if the vector passed in does not have a length of 6
#[rustfmt::skip]
pub fn parse_raw(raw: &Vec<u8>) -> Stroke {
    assert_eq!(raw.len(), 6);
    // checks if the most significant bit is positive
    fn msb_pos(byte: u8) -> bool {
        byte > 127
    }

    let mut raw_stroke: RawStroke = Default::default();
    let mut bytes = raw.iter();

    // first row: number keys 1-6
    if let Some(row) = bytes.next() {
        // always discard the first bit
        let mut row = row << 1;

        // discard the next bit, which is the "Fn" key
        row = row << 1;

        // the next 6 bits are number keys
        for _ in 0..6 {
            // any of them can trigger a number stroke
            if msb_pos(row) {
                raw_stroke.num_key = true;
                break;
            }
            row = row << 1;
        }
    }

    // second row: left hand S- to H-
    if let Some(row) = bytes.next() {
        // always discard the first bit
        let mut row = row << 1;

        // first and second S- key
        let mut s_key = false;
        if msb_pos(row) { s_key = true; } row = row << 1;
        if msb_pos(row) { s_key = true; } row = row << 1;
        if s_key {
            raw_stroke.left_hand.push('S');
        }

        // the next are T, K, P, W, and H
        if msb_pos(row) { raw_stroke.left_hand.push('T'); } row = row << 1;
        if msb_pos(row) { raw_stroke.left_hand.push('K'); } row = row << 1;
        if msb_pos(row) { raw_stroke.left_hand.push('P'); } row = row << 1;
        if msb_pos(row) { raw_stroke.left_hand.push('W'); } row = row << 1;
        if msb_pos(row) { raw_stroke.left_hand.push('H'); }
    }

    // third row: R, A, O, 1*, 2*, and 2 useless keys
    if let Some(row) = bytes.next() {
        // always discard the first bit
        let mut row = row << 1;

        // R, A, and O
        if msb_pos(row) { raw_stroke.left_hand.push('R'); } row = row << 1;
        if msb_pos(row) { raw_stroke.center_left.push('A'); } row = row << 1;
        if msb_pos(row) { raw_stroke.center_left.push('O'); } row = row << 1;

        // first and second star key
        if msb_pos(row) { raw_stroke.star_key = true; } row = row << 1;
        if msb_pos(row) { raw_stroke.star_key = true; }
    }

    // fourth row: useless key, 3rd and 4th star, and E, U, F, R
    if let Some(row) = bytes.next() {
        // always discard the first bit
        let mut row = row << 1;

        // discard useless power key
        row = row << 1;

        // 3rd and 4th star
        if msb_pos(row) { raw_stroke.star_key = true; } row = row << 1;
        if msb_pos(row) { raw_stroke.star_key = true; } row = row << 1;

        // E, U, F, R
        if msb_pos(row) { raw_stroke.center_right.push('E'); } row = row << 1;
        if msb_pos(row) { raw_stroke.center_right.push('U'); } row = row << 1;
        if msb_pos(row) { raw_stroke.right_hand.push('F'); } row = row << 1;
        if msb_pos(row) { raw_stroke.right_hand.push('R'); }
    }

    // fifth row: P, B, L, G, T, S, D
    if let Some(row) = bytes.next() {
        // always discard the first bit
        let mut row = row << 1;

        if msb_pos(row) { raw_stroke.right_hand.push('P'); } row = row << 1;
        if msb_pos(row) { raw_stroke.right_hand.push('B'); } row = row << 1;
        if msb_pos(row) { raw_stroke.right_hand.push('L'); } row = row << 1;
        if msb_pos(row) { raw_stroke.right_hand.push('G'); } row = row << 1;
        if msb_pos(row) { raw_stroke.right_hand.push('T'); } row = row << 1;
        if msb_pos(row) { raw_stroke.right_hand.push('S'); } row = row << 1;
        if msb_pos(row) { raw_stroke.right_hand.push('D'); }
    }

    // sixth row: number keys 7-9, A-C (?), and -Z key
    if let Some(row) = bytes.next() {
        // always discard the first bit
        let mut row = row << 1;

        // number keys 7-9
        for _ in 0..3 {
            if msb_pos(row) {
                raw_stroke.num_key = true;
            }
            row = row << 1;
        }

        // ignore A-C number keys
        for _ in 0..3 {
            row = row << 1;
        }

        // Z key
        if msb_pos(row) { raw_stroke.right_hand.push('Z'); }
    }

    // convert raw stroke to stroke
    raw_stroke.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_stroke_parsing() {
        assert_eq!(parse_raw(&vec![128, 48, 36, 0, 2, 0]), Stroke::new("STA*S"));
        assert_eq!(parse_raw(&vec![160, 2, 0, 0, 32, 64]), Stroke::new("#W-B"));
        assert_eq!(
            parse_raw(&vec![160, 127, 124, 63, 127, 65]),
            Stroke::new("12K3W4R50*EU6R7B8G9SDZ")
        );
        assert_eq!(parse_raw(&vec![128, 21, 0, 0, 0, 0]), Stroke::new("TPH"));
        assert_eq!(parse_raw(&vec![128, 0, 64, 0, 64, 0]), Stroke::new("R-P"));
        assert_eq!(parse_raw(&vec![128, 1, 0, 2, 0, 64]), Stroke::new("4-6"));
        assert_eq!(parse_raw(&vec![128, 1, 32, 2, 0, 64]), Stroke::new("456"));
        assert_eq!(parse_raw(&vec![128, 68, 0, 0, 4, 64]), Stroke::new("13-9"));
    }
}

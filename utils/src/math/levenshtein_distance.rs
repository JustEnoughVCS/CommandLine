use std::cmp::min;

pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_len = a.chars().count();
    let b_len = b.chars().count();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut prev_row: Vec<usize> = (0..=b_len).collect();
    let mut curr_row = vec![0; b_len + 1];

    let mut a_chars = a.chars();

    for i in 1..=a_len {
        let a_char = a_chars.next().unwrap();
        curr_row[0] = i;

        let mut b_chars = b.chars();
        for j in 1..=b_len {
            let b_char = b_chars.next().unwrap();

            let cost = if a_char == b_char { 0 } else { 1 };
            curr_row[j] = min(
                prev_row[j] + 1,
                min(curr_row[j - 1] + 1, prev_row[j - 1] + cost),
            );
        }

        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[b_len]
}

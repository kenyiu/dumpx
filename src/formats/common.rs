//! Shared helpers for format writers.

pub fn csv_escape(input: &str) -> String {
    input.replace('"', "\"\"")
}

pub fn pdf_escape(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

pub fn rows_for_size(target_size: u64, row_size: u64) -> usize {
    ((target_size / row_size) + 1) as usize
}

pub fn repeated_payload(seed: u64, len: usize) -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    (0..len)
        .map(|idx| {
            let pos = (seed as usize + idx * 17 + idx / 3) % ALPHABET.len();
            ALPHABET[pos] as char
        })
        .collect()
}

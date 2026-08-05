#![forbid(unsafe_code)]

use memchr::memmem::Finder;

fn digest_word(digest: u64, value: u64) -> u64 {
    (digest ^ value).wrapping_mul(1_099_511_628_211)
}

fn finish_digest(digest: u64, match_count: u64, haystack_length: u64, needle_length: u64) -> u64 {
    let digest = digest_word(digest, match_count);
    let digest = digest_word(digest, haystack_length);
    digest_word(digest, needle_length)
}

#[inline(never)]
pub fn literal_line(finder: &Finder<'_>, haystack: &[u8], needle: &[u8]) -> u64 {
    let mut digest = 14_695_981_039_346_656_037_u64;
    let mut match_count = 0_u64;
    let mut line_number = 1_u64;
    let mut line_start = 0_usize;

    if needle.is_empty() || needle.iter().any(|byte| *byte == b'\n') {
        return finish_digest(
            digest,
            match_count,
            haystack.len() as u64,
            needle.len() as u64,
        );
    }
    let mut line_cursor = 0_usize;
    let mut search_cursor = 0_usize;
    loop {
        let Some(relative_match) = finder.find(&haystack[search_cursor..]) else {
            break;
        };
        let match_start = search_cursor + relative_match;
        while line_cursor < match_start {
            if line_cursor >= haystack.len() {
                break;
            }
            if haystack[line_cursor] == b'\n' {
                line_start = line_cursor + 1;
                line_number = line_number.wrapping_add(1);
            }
            line_cursor += 1;
        }

        let mut line_end = match_start;
        while line_end < haystack.len() && haystack[line_end] != b'\n' {
            line_end += 1;
        }

        let mut candidate = line_start;
        while needle.len() <= line_end.saturating_sub(candidate) {
            let Some(relative) = finder.find(&haystack[candidate..line_end]) else {
                break;
            };
            let start = candidate + relative;
            let end = start + needle.len();
            digest = digest_word(digest, match_count);
            digest = digest_word(digest, start as u64);
            digest = digest_word(digest, end as u64);
            digest = digest_word(digest, line_start as u64);
            digest = digest_word(digest, line_end as u64);
            digest = digest_word(digest, line_number);
            match_count = match_count.wrapping_add(1);
            candidate = end;
        }

        if line_end < haystack.len() {
            search_cursor = line_end + 1;
            line_cursor = search_cursor;
            line_start = search_cursor;
            line_number = line_number.wrapping_add(1);
        } else {
            search_cursor = haystack.len();
            line_cursor = haystack.len();
        }
    }
    finish_digest(
        digest,
        match_count,
        haystack.len() as u64,
        needle.len() as u64,
    )
}

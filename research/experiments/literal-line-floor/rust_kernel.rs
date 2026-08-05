#![forbid(unsafe_code)]

fn digest_word(digest: u64, value: u64) -> u64 {
    (digest ^ value).wrapping_mul(1_099_511_628_211)
}

fn finish_digest(digest: u64, match_count: u64, haystack_length: u64, needle_length: u64) -> u64 {
    let digest = digest_word(digest, match_count);
    let digest = digest_word(digest, haystack_length);
    digest_word(digest, needle_length)
}

fn find_scalar(haystack: &[u8], needle: &[u8], search_start: usize, search_end: usize) -> usize {
    let range_length = search_end - search_start;
    if range_length < needle.len() {
        return search_end;
    }
    let needle_first = needle[0];
    let last_candidate = search_end - needle.len();
    let mut candidate = search_start;
    while candidate <= last_candidate {
        if candidate >= search_end {
            break;
        }
        let mut equal = haystack[candidate] == needle_first;
        if equal {
            let mut needle_index = 1_usize;
            while needle_index < needle.len() {
                let haystack_index = candidate + needle_index;
                if haystack_index >= search_end || haystack[haystack_index] != needle[needle_index]
                {
                    equal = false;
                    break;
                }
                needle_index += 1;
            }
        }
        if equal {
            return candidate;
        }
        candidate += 1;
    }
    search_end
}

#[inline(never)]
pub fn literal_line(haystack: &[u8], needle: &[u8]) -> u64 {
    let mut digest = 14_695_981_039_346_656_037_u64;
    let mut match_count = 0_u64;
    let mut line_number = 1_u64;
    let mut line_start = 0_usize;
    let mut line_cursor = 0_usize;
    let mut search_cursor = 0_usize;

    if needle.is_empty() || needle.iter().any(|byte| *byte == b'\n') {
        return finish_digest(
            digest,
            match_count,
            haystack.len() as u64,
            needle.len() as u64,
        );
    }
    loop {
        let match_start = find_scalar(haystack, needle, search_cursor, haystack.len());
        if match_start == haystack.len() {
            break;
        }
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
        loop {
            let found = find_scalar(haystack, needle, candidate, line_end);
            if found == line_end {
                break;
            }
            let match_end = found + needle.len();
            digest = digest_word(digest, match_count);
            digest = digest_word(digest, found as u64);
            digest = digest_word(digest, match_end as u64);
            digest = digest_word(digest, line_start as u64);
            digest = digest_word(digest, line_end as u64);
            digest = digest_word(digest, line_number);
            match_count = match_count.wrapping_add(1);
            candidate = match_end;
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

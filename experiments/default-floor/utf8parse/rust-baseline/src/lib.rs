//! Frozen shipped-Rust baseline for the utf8parse default-floor study.
//!
//! The UTF-8 state machine is not reimplemented here. [`parse_into`] feeds
//! every source byte to the public [`utf8parse::Parser`] API and records the
//! resulting [`utf8parse::Receiver`] events in caller-owned storage.

use utf8parse::{Parser, Receiver};

/// Event value used when [`Receiver::invalid_sequence`] is called.
///
/// This is the first integer above Unicode's scalar-value range, so it cannot
/// collide with any value emitted by [`Receiver::codepoint`].
pub const INVALID_EVENT: u32 = 0x0011_0000;

/// The output buffer cannot hold the parser's worst-case one-event-per-byte
/// result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutputTooSmall;

struct EventSink<'a> {
    out: &'a mut [u32],
    produced: usize,
}

impl Receiver for EventSink<'_> {
    fn codepoint(&mut self, codepoint: char) {
        self.out[self.produced] = codepoint as u32;
        self.produced += 1;
    }

    fn invalid_sequence(&mut self) {
        self.out[self.produced] = INVALID_EVENT;
        self.produced += 1;
    }
}

/// Parse `src` from the parser's initial ground state into caller-owned event
/// storage.
///
/// A successful UTF-8 sequence produces its Unicode scalar value. An invalid
/// sequence produces [`INVALID_EVENT`]. Incomplete sequences at end of input
/// produce no event, matching `utf8parse`'s streaming API (which has no EOF or
/// finish operation). The byte which detects an invalid continuation is
/// consumed by the invalid event and is not processed again from ground state.
///
/// The wrapper requires `out.len() >= src.len()` before writing. The upstream
/// parser emits at most one event per input byte, so this is sufficient for all
/// inputs. The returned count identifies the written prefix; the remaining
/// suffix is left untouched.
pub fn parse_into(out: &mut [u32], src: &[u8]) -> Result<usize, OutputTooSmall> {
    if out.len() < src.len() {
        return Err(OutputTooSmall);
    }

    let mut parser = Parser::new();
    let mut sink = EventSink { out, produced: 0 };
    for &byte in src {
        parser.advance(&mut sink, byte);
    }
    Ok(sink.produced)
}

#[cfg(test)]
mod tests {
    use super::*;

    const GUARD: u32 = 0xDEAD_BEEF;

    fn parsed(src: &[u8]) -> Vec<u32> {
        let mut out = vec![GUARD; src.len()];
        let produced = parse_into(&mut out, src).expect("input-sized output is sufficient");
        out.truncate(produced);
        out
    }

    #[test]
    fn emits_ascii_and_valid_multibyte_scalars() {
        let source = "\0A\u{80}\u{7ff}\u{800}\u{d7ff}\u{e000}\u{ffff}\u{10000}\u{10ffff}";
        assert_eq!(
            parsed(source.as_bytes()),
            source.chars().map(u32::from).collect::<Vec<_>>()
        );
    }

    #[test]
    fn rejects_invalid_leads_and_unicode_boundary_violations() {
        let cases: &[&[u8]] = &[
            &[0x80],
            &[0xbf],
            &[0xc0],
            &[0xc1],
            &[0xf5],
            &[0xff],
            &[0xe0, 0x80],
            &[0xed, 0xa0],
            &[0xf0, 0x80],
            &[0xf4, 0x90],
        ];
        for &source in cases {
            assert_eq!(parsed(source), [INVALID_EVENT], "source {source:02x?}");
        }
    }

    #[test]
    fn consumes_the_byte_which_detects_a_broken_sequence() {
        assert_eq!(parsed(&[0xc2, b'A']), [INVALID_EVENT]);
        assert_eq!(parsed(&[0xe2, 0x82, b'A']), [INVALID_EVENT]);
        assert_eq!(parsed(&[0xe0, 0x9f, b'A']), [INVALID_EVENT, b'A' as u32]);
        assert_eq!(parsed(&[0xc2, 0xc2, 0x80]), [INVALID_EVENT, INVALID_EVENT]);
    }

    #[test]
    fn incomplete_end_of_input_produces_no_event() {
        assert!(parsed(&[0xc2]).is_empty());
        assert!(parsed(&[0xe2, 0x82]).is_empty());
        assert!(parsed(&[0xf0, 0x90, 0x80]).is_empty());
        assert_eq!(parsed(&[b'A', 0xf0, 0x90, 0x80]), [b'A' as u32]);
    }

    #[test]
    fn leaves_unused_output_suffix_and_source_untouched() {
        let source = b"A\xf0\x9f\x92\xa9Z".to_vec();
        let source_before = source.clone();
        let mut out = [GUARD; 16];
        let produced = parse_into(&mut out, &source).unwrap();

        assert_eq!(&out[..produced], &[b'A' as u32, 0x1f4a9, b'Z' as u32]);
        assert!(out[produced..].iter().all(|&event| event == GUARD));
        assert_eq!(source, source_before);
    }

    #[test]
    fn rejects_less_than_worst_case_capacity_before_writing() {
        // The particular valid input emits only one event, but the public
        // contract deliberately requires worst-case input-sized capacity.
        let source = "\u{10ffff}".as_bytes();
        let mut out = [GUARD; 3];
        assert_eq!(parse_into(&mut out, source), Err(OutputTooSmall));
        assert_eq!(out, [GUARD; 3]);
    }

    #[test]
    fn exact_capacity_handles_the_one_event_per_byte_case() {
        let source = [b'A', 0x80, b'B', 0xff];
        let mut out = [GUARD; 4];
        assert_eq!(parse_into(&mut out, &source), Ok(4));
        assert_eq!(
            out,
            [b'A' as u32, INVALID_EVENT, b'B' as u32, INVALID_EVENT]
        );
    }

    #[test]
    fn accepts_empty_input_and_output() {
        assert_eq!(parse_into(&mut [], &[]), Ok(0));
    }
}

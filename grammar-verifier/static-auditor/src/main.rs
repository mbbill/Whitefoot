use std::io::{self, Read, Write};

const HARD_FRAME_BYTES: usize = 48 + 8_192 + 2 * 1_048_576 + 2 * 262_144;

fn main() {
    let mut input = Vec::new();
    let mut block = [0_u8; 8192];
    loop {
        let count = match io::stdin().read(&mut block) {
            Ok(count) => count,
            Err(_) => std::process::exit(1),
        };
        if count == 0 {
            break;
        }
        let Some(total) = input.len().checked_add(count) else {
            std::process::exit(1);
        };
        if total > HARD_FRAME_BYTES || input.try_reserve(count).is_err() {
            let report = b"WFGRREPORT1\nENGINE\tstatic\nFAIL\tinput\tframe-outer-limit\nEND\n";
            if io::stdout().write_all(report).is_err() {
                std::process::exit(1);
            }
            return;
        }
        input.extend_from_slice(&block[..count]);
    }
    let output = whitefoot_static_grammar_auditor::process_frame(&input);
    if io::stdout().write_all(&output).is_err() {
        std::process::exit(1);
    }
}

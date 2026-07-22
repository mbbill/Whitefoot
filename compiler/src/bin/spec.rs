#![forbid(unsafe_code)]

use whitefoot::{KERNEL_SPEC_V0_10_HASH, SYNTAX_DATA_SPEC_V0_10, TERMINAL_CONTRACT_SPEC_V0_10};

const ACTIVE_SPEC: &[u8] = include_bytes!("../../../spec/kernel-spec-v0.10.md");
const APPROVED_CANDIDATE: &[u8] =
    include_bytes!("../../../governance/spec-evolution/kernel-spec-v0.10-candidate.md");

fn main() {
    if ACTIVE_SPEC != APPROVED_CANDIDATE {
        eprintln!("spec/kernel-spec-v0.10.md differs from the approved candidate");
        std::process::exit(1);
    }
    if SYNTAX_DATA_SPEC_V0_10 != KERNEL_SPEC_V0_10_HASH
        || TERMINAL_CONTRACT_SPEC_V0_10 != KERNEL_SPEC_V0_10_HASH
    {
        eprintln!("frontend data is not bound to the active v0.10 identity");
        std::process::exit(1);
    }
    println!("Whitefoot v0.10 frontend identity: {KERNEL_SPEC_V0_10_HASH}");
}

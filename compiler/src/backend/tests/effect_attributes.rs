//! Tripwire on the LLVM function attributes the backend is allowed to emit.
//!
//! The backend emits no function attributes today, so the assertion below is
//! currently satisfied without the emitter doing anything. That is the point:
//! it is a canary for the effect-attribute channel that a future change will
//! open when it starts translating Whitefoot effect rows (`pure`, `reads`,
//! `writes`, `traps`) into LLVM attributes. It must fail on the day anyone
//! emits `willreturn`, before the resulting miscompile reaches a program.
//!
//! Why `willreturn` specifically, and why not just the combination:
//!
//! [EFF-3] of the active kernel specification says `pure` "licenses
//! deduplication and reordering of calls with equal arguments. Elimination of
//! an unused pure call additionally requires a termination proof; v0 provides
//! no termination checker, so unused pure calls are not eliminated. `pure`
//! excludes traps and all reads/writes/allocates; it does not promise
//! termination." `willreturn` is exactly the termination promise the language
//! does not make, so emitting it asserts something v0 cannot prove.
//!
//! Measured hazard, on a function containing an executed claim path: with
//! `nounwind willreturn memory(argmem: read)`, LLVM DELETES the claim trap.
//! The program then exits 0 with empty stderr where it must abort with the
//! DIAG-3 record. Every other tested combination kept the trap and aborted
//! correctly: `nounwind`; `willreturn`; `memory(read)`; `nounwind
//! memory(read)`; `nounwind memory(argmem: read)`; `willreturn memory(read)`;
//! and `nounwind willreturn memory(read, inaccessiblemem: readwrite)`. The
//! claim trap vanishes only when `nounwind` and `willreturn` and a memory class that
//! excludes `inaccessiblemem` all coincide, in the shape where a caller
//! discards the callee's result - which is precisely how the emitted `main`
//! shim calls `wf_main`.
//!
//! The tripwire is on `willreturn` alone rather than on the triple because
//! `nounwind` and a narrow `memory(...)` class are each individually harmless
//! and are the attributes an effect row would most naturally justify, while
//! `willreturn` is the one member of the triple that no v0 effect row can
//! license. Guarding the single unlicensed attribute keeps the useful channel
//! open and closes the unsound one. Do not delete this test as vacuous: it is
//! meant to be silent until the channel opens.
//!
//! The check is on the module the backend emits, not on the module the host
//! optimizer returns. Do not "strengthen" it to the optimized module: the host
//! optimizer infers and writes its own attribute groups (an `-O2` run over
//! `tests/programs/recursive_tree.wf` yields eight `willreturn` occurrences,
//! including on declared allocation helpers), which is LLVM reasoning about its
//! own IR and is not this compiler making a promise. What this test controls is
//! what the backend asserts.

use super::compile;

/// Representative accepted programs from the corpus. Their claims are not
/// part of this test's contract: corpus migrations may legitimately remove
/// test-oracle claims. `recursive_tree` exercises a self-recursive call and
/// `generic_instances` exercises monomorphized instances, so the check still
/// covers the call shapes an effect-attribute pass would annotate.
const REPRESENTATIVE_PROGRAMS: &[(&str, &[u8])] = &[
    (
        "tests/programs/sha256_abc.wf",
        include_bytes!("../../../../tests/programs/sha256_abc.wf"),
    ),
    (
        "tests/programs/fir_filter.wf",
        include_bytes!("../../../../tests/programs/fir_filter.wf"),
    ),
    (
        "tests/programs/percent_decode.wf",
        include_bytes!("../../../../tests/programs/percent_decode.wf"),
    ),
    (
        "tests/programs/telemetry_packet.wf",
        include_bytes!("../../../../tests/programs/telemetry_packet.wf"),
    ),
    (
        "tests/programs/generic_instances.wf",
        include_bytes!("../../../../tests/programs/generic_instances.wf"),
    ),
    (
        "tests/programs/recursive_tree.wf",
        include_bytes!("../../../../tests/programs/recursive_tree.wf"),
    ),
];

/// A self-contained, genuinely residual theorem supplies the claim trap path
/// used by the measured-hazard tripwire. The theorem follows from the helper's
/// body, ENT cannot publish that uncontracted result bound, and the exact
/// addition consumes precisely the written predicate.
const RESIDUAL_CLAIM_PROGRAM: &[u8] = br#"fn clamp_zero(value: own u64) -> result: own u64 pure {
  return imin(value, 0_u64);
}

command fn main() -> status: own ExitStatus traps {
  let bounded = clamp_zero(value: 1_u64);
  claim bounded_below_one: ilt(bounded, 1_u64) because "premises: bounded is returned by clamp_zero, whose body computes imin(1_u64, 0_u64)\nderivation: imin(1_u64, 0_u64) is 0_u64, which is strictly less than 1_u64\nconclusion: ilt(bounded, 1_u64) is true\nchecker gap: ENT does not publish an uncontracted user-call result bound\nconsumers: the following bounded + 1_u64 exact addition requires this bound for its OP-2 domain obligation";
  let successor = bounded + 1_u64;
  return exit_status(code: 0_u8);
}
"#;

#[test]
fn no_emitted_module_promises_termination_with_the_willreturn_attribute() {
    for (path, source) in REPRESENTATIVE_PROGRAMS {
        let module = compile(source);
        // Guards against the corpus coverage going vacuous on a degenerate
        // module without coupling that coverage to its current claim count.
        assert!(
            module.contains("define "),
            "{path} must emit at least one function definition"
        );
        assert!(
            !module.contains("willreturn"),
            "{path} emitted the willreturn attribute; v0 has no termination \
             checker, so no effect row licenses that promise [EFF-3], and \
            `nounwind willreturn memory(argmem: read)` was measured to delete \
             a written claim trap"
        );
    }

    let residual = compile(RESIDUAL_CLAIM_PROGRAM);
    assert!(residual.contains("define "));
    assert!(
        residual.contains("@wf_trap"),
        "the self-contained residual fixture must emit its written claim trap path"
    );
    assert!(
        !residual.contains("willreturn"),
        "the residual fixture emitted the unlicensed willreturn attribute"
    );
}

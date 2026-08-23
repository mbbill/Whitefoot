//! The resource-exhaustion floor's translation unit and the module's own
//! standalone answer for it.
//!
//! This is the structural twin of [`super::parallel`]'s runtime wiring, with
//! one deliberate difference: the parallel runtime joins the link only when a
//! module hands work out, and the floor joins it always. Every program can run
//! out of stack, so every program carries the unit that reports it.

/// The floor runtime's source, carried inside the compiler.
///
/// Its bytes travel in the compiler binary and are written beside the module
/// at link time, so no installed path, build directory, or environment
/// variable decides which floor a program gets.
pub const FLOOR_RUNTIME_SOURCE: &str = include_str!("../wf_floor.c");

/// The module's own definition of the floor entry point.
///
/// An emitted module is a complete program on its own — `--emit-llvm` output
/// must link and run without the compiler's driver — so the module defines
/// every runtime symbol it names. `weak` linkage lets the real definition in
/// [`FLOOR_RUNTIME_SOURCE`] replace this one whenever that unit is linked,
/// which for the floor is every ordinary build.
///
/// The call carries `noinline` so that a small enough program is not pasted
/// into this definition as well, leaving every module with a second copy of
/// the whole program for the sake of a definition that only exists to keep an
/// unlinked module runnable. The restriction belongs here rather than on the
/// entry itself, where it would also suppress the optimizer's hot/cold
/// splitting of the program's own failure arms.
///
/// Without the runtime there is no second stack to move to, so the honest
/// answer is to run the entry on the thread the host started: the program
/// still runs and still means the same thing, with the ceiling and the bare
/// host signal it had before the floor existed.
pub(crate) const FLOOR_RUNTIME_FALLBACK: &str = "define weak i32 @wf__floor_run(i32 %argc, ptr %argv) {\nentry:\n  %status = call i32 @wf__main_body(i32 %argc, ptr %argv) noinline\n  ret i32 %status\n}\n\n";

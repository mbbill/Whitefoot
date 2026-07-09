use puremath::Config;

// A TRUE dynamic-library boundary. The caller links against this symbol; the
// body lives in a separate .dylib. LTO cannot cross it; the caller sees only an
// external `bl` to an opaque symbol with NO purity attributes. This is where a
// signature-level EFFECT ROW (xlang) would still let the caller CSE/hoist/DCE,
// and where safe Rust cannot — the compiler must assume it reads/writes all
// memory and may trap.
#[no_mangle]
pub extern "C" fn compute_gain_dyn(cfg: &Config) -> f64 {
    puremath::compute_gain(cfg)
}

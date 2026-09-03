use super::support::{compile_and_run, compile_program, compile_sources};

const GENERIC_LIBRARY: &[u8] = br#"struct Pair<T: Int> {
  value: T;
}

fn bundle_pair<T: Int>(value: own T) -> pair: own Pair<T> pure {
  return Pair<T>(value: value);
}
"#;
const GENERIC_CONSUMER: &[u8] = br#"fn forward<T: Int>(value: own T) -> pair: own Pair<T> pure {
  return bundle_pair::<T>(value: value);
}

command fn main() -> status: own ExitStatus pure {
  let small = forward::<u8>(value: 13_u8);
  let wide = forward::<i64>(value: -17_i64);
  let small_value = small.value;
  let wide_value = wide.value;
  if small_value == 13_u8 {
  } else {
    return exit_status(code: 1_u8);
  }
  if wide_value == -17_i64 {
  } else {
    return exit_status(code: 2_u8);
  }
  return exit_status(code: 0_u8);
}
"#;

#[test]
fn concrete_type_and_const_instances_have_distinct_symbols_and_execute() {
    let llvm = compile_program("generic_instances.wf");
    for name in ["maximum", "forward", "preserve"] {
        let symbol = format!("@wf_{name}$instance$");
        let definitions = llvm
            .lines()
            .filter(|line| line.starts_with("define internal") && line.contains(&symbol))
            .collect::<Vec<_>>();
        assert_eq!(definitions.len(), 2, "{name} definitions: {definitions:?}");
        assert_ne!(definitions[0], definitions[1]);
    }
    assert_eq!(
        llvm.lines()
            .filter(|line| {
                line.starts_with("define internal") && line.contains("@wf_filled_array$instance$")
            })
            .count(),
        2
    );
    assert_eq!(
        llvm.lines()
            .filter(|line| {
                line.starts_with("define internal") && line.contains("@wf_filled_buffer$instance$")
            })
            .count(),
        1
    );

    let output = compile_and_run(&llvm);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn concrete_generic_symbol_order_is_deterministic() {
    assert_eq!(
        compile_program("generic_instances.wf"),
        compile_program("generic_instances.wf")
    );
}

#[test]
fn concrete_generic_struct_enum_and_const_nominal_instances_execute() {
    let llvm = compile_program("generic_nominals.wf");
    for name in ["duplicate", "present", "checked_sum"] {
        let symbol = format!("@wf_{name}$instance$");
        assert_eq!(
            llvm.lines()
                .filter(|line| line.starts_with("define internal") && line.contains(&symbol))
                .count(),
            2,
            "{name} must have one definition per concrete type"
        );
    }

    let output = compile_and_run(&llvm);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn concrete_generic_nominal_order_is_deterministic() {
    assert_eq!(
        compile_program("generic_nominals.wf"),
        compile_program("generic_nominals.wf")
    );
}

#[test]
fn generic_instances_forward_across_ordered_source_records() {
    let llvm = compile_sources(&[
        ("library/generics.wf", GENERIC_LIBRARY),
        ("application/main.wf", GENERIC_CONSUMER),
    ]);
    for name in ["bundle_pair", "forward"] {
        assert_eq!(
            llvm.lines()
                .filter(|line| {
                    line.starts_with("define internal")
                        && line.contains(&format!("@wf_{name}$instance$"))
                })
                .count(),
            2
        );
    }
    let output = compile_and_run(&llvm);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

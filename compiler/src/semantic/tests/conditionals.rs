//! [GRAM-6] type-driven conditional control: a Bool condition takes `if`, an
//! enum scrutinee takes `match`, and each is the sole form for its class. The
//! two `else` spellings the rule refuses are checked here at the exact nodes
//! it names, together with the cases it deliberately leaves to [GIVE-1].

use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::{assert_rule, assert_rule_at, with_semantics};

fn assert_checks(source: &[u8]) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("conditional must check: {outcome:?}");
        };
        assert_eq!(checked.entry_function_name(), "main");
    });
}

#[test]
fn a_bool_scrutinee_match_is_a_gram6_rejection_at_the_scrutinee() {
    // migrate: keep — the Bool `match` is this test's whole subject, and the
    // migration rewrites it into the `if` [GRAM-6] demands, leaving a source
    // that checks clean and an assertion that no longer asserts anything.
    let source = br#"fn main() -> own unit pure {
  let flag = True();
  match flag {
    True() => {
      return unit;
    }
    False() => {
      return unit;
    }
  }
}
"#;
    assert_rule_at(source, SemanticRule::Gram6, "flag");
}

#[test]
fn an_enum_scrutinee_still_takes_match() {
    let source = br#"enum Signal {
  Stop();
  Go();
}

fn main() -> own unit pure {
  let signal = Go();
  match signal {
    Stop() => {
      return unit;
    }
    Go() => {
      return unit;
    }
  }
}
"#;
    assert_checks(source);
}

#[test]
fn an_empty_else_is_a_gram6_rejection_at_the_if() {
    let source = br#"fn main() -> own unit pure {
  let flag = True();
  if flag {
    return unit;
  } else {
  }
  return unit;
}
"#;
    assert_rule_at(
        source,
        SemanticRule::Gram6,
        "if flag {\n    return unit;\n  } else {\n  }",
    );
}

#[test]
fn an_unflattened_else_if_is_a_gram6_rejection_at_the_nested_if() {
    let source = br#"fn main() -> own unit pure {
  let flag = True();
  if flag {
    return unit;
  } else {
    if flag {
      return unit;
    } else {
      return unit;
    }
  }
}
"#;
    assert_rule_at(
        source,
        SemanticRule::Gram6,
        "if flag {\n      return unit;\n    } else {\n      return unit;\n    }",
    );
}

#[test]
fn a_flattened_else_if_chain_checks() {
    let source = br#"fn main() -> own unit pure {
  let flag = True();
  if flag {
    return unit;
  } else if flag {
    return unit;
  } else {
    return unit;
  }
}
"#;
    assert_checks(source);
}

#[test]
fn an_else_free_if_is_the_empty_alternative_form() {
    let source = br#"fn main() -> own unit pure {
  let flag = True();
  if flag {
    return unit;
  }
  return unit;
}
"#;
    assert_checks(source);
}

#[test]
fn an_empty_then_block_is_admitted_where_an_empty_else_is_not() {
    let source = br#"fn main() -> own unit pure {
  let flag = True();
  if flag {
  } else {
    return unit;
  }
  return unit;
}
"#;
    assert_checks(source);
}

#[test]
fn a_non_bool_condition_is_a_gram6_rejection_at_the_condition() {
    let source = br#"fn main() -> own unit pure {
  let count = 3_u64;
  if count {
    return unit;
  }
  return unit;
}
"#;
    assert_rule_at(source, SemanticRule::Gram6, "count");
}

#[test]
fn a_value_if_derives_its_binder_from_the_delivery_set() {
    let source = br#"fn main() -> own unit pure {
  let flag = True();
  let picked = if flag {
    give 1_i32;
  } else {
    give 2_i32;
  }
  return unit;
}
"#;
    assert_checks(source);
}

#[test]
fn a_value_if_holds_its_deliveries_to_one_exact_type() {
    let source = br#"fn main() -> own unit pure {
  let flag = True();
  let picked = if flag {
    give 1_i32;
  } else {
    give 2_u64;
  }
  return unit;
}
"#;
    assert_rule(source, SemanticRule::Give1, SemanticIssueKind::TypeMismatch);
}

/// [GIVE-1] owns the undelivering `value_if`, not GRAM-6: a `value_if`'s
/// `else` is mandatory by grammar, so an empty one is an empty delivery set
/// rather than the else-free form GRAM-6 asks for.
#[test]
fn an_empty_value_if_else_is_a_give1_rejection() {
    let source = br#"fn main() -> own unit pure {
  let flag = True();
  let picked = if flag {
    give 1_i32;
  } else {
  }
  return unit;
}
"#;
    assert_rule(source, SemanticRule::Give1, SemanticIssueKind::InvalidGive);
}

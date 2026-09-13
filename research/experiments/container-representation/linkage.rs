//! Linkage adaptation for the closed C measurement harnesses.
//!
//! Ordinary WF definitions are linkable in v0.58. The retained controls expose
//! only the measured bridge, as they did before the amendment; their helper
//! bodies stay local so the comparison has the same interprocedural scope.
//! No call, data layout, contract or instruction changes here.

pub fn closed_helpers(module: &str) -> String {
    let mut output = String::with_capacity(module.len());
    for line in module.split_inclusive('\n') {
        let definition = line.starts_with("define ")
            && !line.starts_with("define internal ")
            && !line.starts_with("define private ")
            && !line.starts_with("define weak ");
        let ordinary = line.split_once('@').is_some_and(|(_, name)| {
            let name = name.trim_start_matches('"');
            name.starts_with("wf_") && !name.starts_with("wf__")
        });
        if definition && ordinary {
            output.push_str("define internal ");
            output.push_str(&line["define ".len()..]);
        } else {
            output.push_str(line);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::closed_helpers;

    #[test]
    fn only_ordinary_definition_linkage_changes() {
        let source = "define i64 @wf_helper(i64 %x) {\n  ret i64 %x\n}\ndeclare void @wf_close_read(ptr, ptr)\ndefine i32 @wf__main_body() {\n  %v = call i64 @wf_helper(i64 1)\n  ret i32 0\n}\ndefine private void @wf_abort() {\n  unreachable\n}\n";
        let expected =
            source.replacen("define i64 @wf_helper", "define internal i64 @wf_helper", 1);
        assert_eq!(closed_helpers(source), expected);
        assert_eq!(closed_helpers(&expected), expected);
    }
}

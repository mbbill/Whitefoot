//! Diagnostic-only boundaries in the current scatter root's two worlds.
//! Every boundary joins its phase. Reject an unfamiliar lowering rather than
//! guessing where work completes; the ordinary timing image never uses this.
//! Retire with this consumer's phase experiment when its source shape changes.

use std::{env, fs};

fn instrument(source: &str) -> Result<String, String> {
    let mut output = String::with_capacity(source.len() + 1024);
    let mut active = false;
    let mut roots = 0;
    let (mut phase, mut slots, mut arrays, mut loops, mut packs, mut frees) = (0, 0, 0, 0, 0, 0);
    for line in source.lines() {
        if line.starts_with("define ") {
            active = line.contains(" @wf_radix_scatter(")
                || line.contains(" @wf__par_seq_radix_scatter(");
            if active {
                roots += 1;
                (phase, slots, arrays, loops, packs, frees) = (0, 0, 0, 0, 0, 0);
            }
        }
        let (mut before, mut after) = (None, None);
        if active {
            if line == "entry:" {
                after = Some(0);
            } else if line.contains(" = call ptr @wf_box_slots_new$") {
                slots += 1;
            } else if line.contains(" = call i64 @wf__par_split_budget(") {
                // The initialization loop has joined before map admission.
                before = Some(1);
            } else if line.contains(" = call i8 @wf__par_split_")
                || line.contains(" = call i8 @wf__par_seq__par_chunk_")
            {
                loops += 1;
                if phase == 1 {
                    before = Some(1); // The sequential world has no admission call.
                }
                after = Some(2);
            } else if line.contains(" = call ptr @wf_box_array_filled$") {
                arrays += 1;
                if arrays == 1 {
                    before = Some(3);
                } else if arrays == 3 {
                    after = Some(6);
                }
            } else if line.contains(" = call i64 @wf_pack_chunks(")
                || line.contains(" = call i64 @wf__par_seq_pack_chunks(")
            {
                packs += 1;
                before = Some(4);
                after = Some(5);
            } else if line.trim_start().starts_with("call void @free(") {
                frees += 1;
                if frees == 1 {
                    // Both final copies have joined before any temporary dies.
                    before = Some(7);
                }
            } else if line.trim_start().starts_with("ret ptr ") {
                before = Some(8);
            }
        }
        let emit = |out: &mut String, event: usize, next: &mut usize| {
            if event != *next {
                return Err(format!("unexpected phase {event}; expected {next}"));
            }
            out.push_str(&format!("  call void @wf_scatter_phase(i32 {event})\n"));
            *next += 1;
            Ok(())
        };
        if let Some(event) = before {
            emit(&mut output, event, &mut phase)?;
        }
        output.push_str(line);
        output.push('\n');
        if let Some(event) = after {
            emit(&mut output, event, &mut phase)?;
        }
        if active && line == "}" {
            if (phase, slots, arrays, loops, packs, frees) != (9, 1, 3, 1, 1, 3) {
                return Err(format!(
                    "incomplete scatter root: phases={phase} slots={slots} arrays={arrays} \
                     loops={loops} packs={packs} frees={frees}"
                ));
            }
            active = false;
        }
    }
    if roots != 2 || active {
        return Err(format!(
            "expected two complete scatter roots, found {roots}"
        ));
    }
    output.push_str("\ndeclare void @wf_scatter_phase(i32)\n");
    Ok(output)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        return Err("usage: radix_scatter_phases INPUT.ll OUTPUT.ll".into());
    }
    fs::write(&args[2], instrument(&fs::read_to_string(&args[1])?)?)?;
    Ok(())
}

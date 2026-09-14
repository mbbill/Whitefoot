//! Insert coarse diagnostic boundaries into this consumer's emitted module.
//! The ordinary timing image never reads this output. Fail closed when its
//! lowering no longer has the inspected shape; do not guess new boundaries.

use std::{env, fs};

fn instrument(source: &str) -> Result<String, String> {
    let mut output = String::with_capacity(source.len() + 1024);
    let mut active = false;
    let mut roots = 0;
    let mut phase = 0;
    let mut fills = 0;
    let mut loops = 0;
    let mut packs = 0;
    let mut frees = 0;
    for line in source.lines() {
        if line.starts_with("define ") {
            active = line.contains(" @wf_radix_scatter(")
                || line.contains(" @wf__par_seq_radix_scatter(");
            if active {
                roots += 1;
                phase = 0;
                fills = 0;
                loops = 0;
                packs = 0;
                frees = 0;
            }
        }
        let mut before = None;
        let mut after = None;
        if active {
            if line == "entry:" {
                after = Some(0);
            } else if line.starts_with("buffer.vacant.done.") {
                after = Some(1);
            } else if line.contains(" = call i8 @wf__par_split_")
                || line.contains(" = call i8 @wf__par_seq__par_chunk_")
            {
                loops += 1;
                after = Some(2);
            } else if line
                .trim_start()
                .starts_with("br label %buffer.fill.allocate.")
            {
                fills += 1;
                if fills == 1 {
                    before = Some(3);
                }
            } else if line.contains(" = call i64 @wf_pack_chunks(")
                || line.contains(" = call i64 @wf__par_seq_pack_chunks(")
            {
                packs += 1;
                before = Some(4);
                after = Some(5);
            } else if fills == 3 && line.starts_with("buffer.fill.done.") {
                after = Some(6);
            } else if line.trim_start().starts_with("call void @free(") {
                frees += 1;
                if frees == 1 {
                    before = Some(7);
                }
            } else if line.trim_start().starts_with("ret { ptr, i64 }") {
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
            if (phase, fills, loops, packs, frees) != (9, 3, 1, 1, 3) {
                return Err(format!(
                    "incomplete scatter root: phases={phase} fills={fills} loops={loops} packs={packs} frees={frees}"
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
    let source = fs::read_to_string(&args[1])?;
    let output = instrument(&source)?;
    fs::write(&args[2], output)?;
    Ok(())
}

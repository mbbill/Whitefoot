#!/bin/bash
# Generates one Whitefoot source for the paired benchmark.
# usage: gen_wf.sh SHAPE DEPTH WORDS REPS OUTPATH
#
# Two workload families share the four parameters, because they mean the same
# things in both: DEPTH is how deep the recursive pair descends, WORDS is how
# much arithmetic one leaf does, REPS is how many times the whole measurement
# repeats.
#
#   bal, skew  a box-tree layout fold. DEPTH is the tree depth, WORDS is the
#              length of the shared metric table each node measures.
#   grid       a Mandelbrot escape count over a 2^DEPTH index range, split by
#              recursive halving. WORDS is the orbit iteration cap. The pair is
#              the two halves of the split, so the parallelism comes from the
#              index range rather than from a data structure.
set -euo pipefail

SHAPE="$1"
DEPTH="$2"
WORDS="$3"
REPS="$4"
OUT="$5"

if [ "$SHAPE" = "grid" ]; then
  # The complex plane is fixed at x in [-2, 1.2) and y in [-1.2, 1.2), and the
  # grid is 1024 columns wide, so a linear index splits into row and column
  # with a mask and a shift and needs no division-domain discharge. DEPTH sets
  # the row count, and with it the vertical scale.
  if [ "$DEPTH" -lt 11 ]; then
    echo "gen_wf.sh: grid DEPTH must be at least 11 (1024 columns, 2+ rows)" >&2
    exit 2
  fi
  POINTS=$(( 1 << DEPTH ))
  Y_SCALE=$(perl -e "printf '%.17g', 2.4 / (1 << ($DEPTH - 10))")
  # [FORM-5] admits only the canonical spelling of a float, so a generated
  # literal can be rejected two ways: its shape can be outside the grammar,
  # which this checks here where the reason is visible, or it can be a
  # non-canonical spelling of a legal value, which the compiler reports as a
  # source error on the generated file. Both are loud; neither can produce a
  # benchmark that measures something other than what it says.
  if ! printf '%s' "$Y_SCALE" | grep -Eq '^(0|[1-9][0-9]*)\.[0-9]+(e-?(0|[1-9][0-9]*))?$'; then
    echo "gen_wf.sh: grid DEPTH $DEPTH needs vertical scale $Y_SCALE, which [FORM-5] does not spell" >&2
    exit 2
  fi
  cat > "$OUT" <<WFEOF
fn hex_digit(v: own u64) -> result: own u8 pure {
  let nibble = iand(v, 15_u64);
  let small = ilt(nibble, 10_u64);
  match cvt<u64, u8>(nibble) {
    Ok(value: byte) => {
      if small {
        return byte +wrap 48_u8;
      }
      return byte +wrap 87_u8;
    }
    Err(error: wide) => {
      return 63_u8;
    }
  }
}

fn spell_hex['d](destination: &uniq 'd buffer<u8>, at: own u64, value: own u64) -> result: own u64 reads(destination), writes(destination) {
  doc "Writes one 64-bit value as sixteen lowercase hexadecimal digits, most significant first.";
  let cursor = at;
  let rest = value;
  let limit = at +wrap 16_u64;
  loop @digits {
    let done = ige(cursor, limit);
    if done {
      break @digits;
    }
    let shifted = irotl(rest, 4_u32);
    let digit = hex_digit(v: shifted);
    let room = len(deref(destination));
    let writable = ilt(cursor, room);
    if writable {
      set deref(destination)[cursor] = digit;
    }
    set rest = shifted;
    set cursor = cursor +wrap 1_u64;
  }
  return limit;
}

fn mandelbrot_escapes(cx: own f64, cy: own f64) -> result: own Bool pure {
  doc "One orbit of the quadratic map, to a fixed iteration cap. Its operation domains are total or statically discharged, and its loop makes it the leaf work of the split rather than a bounded closure.";
  let real = 0.0_f64;
  let imaginary = 0.0_f64;
  let iteration = 0_u32;
  let escaped = False();
  loop @orbit {
    let exhausted = ieq(iteration, ${WORDS}_u32);
    if exhausted {
      break @orbit;
    }
    let real_squared = fmul.strict(real, real);
    let imaginary_squared = fmul.strict(imaginary, imaginary);
    let difference = fsub.strict(real_squared, imaginary_squared);
    let next_real = fadd.strict(difference, cx);
    let product = fmul.strict(real, imaginary);
    let doubled_product = fmul.strict(2.0_f64, product);
    let next_imaginary = fadd.strict(doubled_product, cy);
    set real = next_real;
    set imaginary = next_imaginary;
    let next_real_squared = fmul.strict(real, real);
    let next_imaginary_squared = fmul.strict(imaginary, imaginary);
    let magnitude_squared = fadd.strict(next_real_squared, next_imaginary_squared);
    let outside = fgt(magnitude_squared, 4.0_f64);
    if outside {
      set escaped = True();
      break @orbit;
    }
    set iteration = iteration +wrap 1_u32;
  }
  return escaped;
}

fn point_escapes(index: own u32, phase: own f64) -> result: own u32 pure {
  doc "Decodes one linear grid index into a complex coordinate. The grid is 1024 wide, so the row and column split with a mask and a shift and need no division-domain discharge.";
  let column = iand(index, 1023_u32);
  let row = ishr.wrap(index, 10_u32);
  let column_float = cvt<u32, f64>(column);
  let row_float = cvt<u32, f64>(row);
  let scaled_x = fmul.strict(column_float, 0.003125_f64);
  let scaled_y = fmul.strict(row_float, ${Y_SCALE}_f64);
  let cx = fsub.strict(scaled_x, 2.0_f64);
  let shifted_y = fsub.strict(scaled_y, 1.2_f64);
  let cy = fadd.strict(shifted_y, phase);
  let escaped = mandelbrot_escapes(cx: cx, cy: cy);
  if escaped {
    return 1_u32;
  }
  return 0_u32;
}

fn tile(lo: own u32, hi: own u32, phase: own f64) -> result: own u32 pure {
  doc "Counts escaping points over the half-open index range. The two recursive halves are the eligible pair: pure, every operation domain statically satisfied, no shared writable place at all, and no dataflow between them. The combine is +wrap, which is exactly associative, so no schedule can move a bit.";
  let width = hi -wrap lo;
  let single = ieq(width, 1_u32);
  if single {
    return point_escapes(index: lo, phase: phase);
  }
  let empty = ieq(width, 0_u32);
  if empty {
    return 0_u32;
  }
  let half = ishr.wrap(width, 1_u32);
  let mid = lo +wrap half;
  let a = tile(lo: lo, hi: mid, phase: phase);
  let b = tile(lo: mid, hi: hi, phase: phase);
  return a +wrap b;
}

command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  doc "Counts the escaping points of a ${POINTS}-point grid by recursive index split, once per repetition, and publishes the running total as hexadecimal.";
  let total = 0_u32;
  let phase = 0.0_f64;
  let i = 0_u64;
  loop @reps {
    let done = ieq(i, ${REPS}_u64);
    if done {
      break @reps;
    }
    let escaped = tile(lo: 0_u32, hi: ${POINTS}_u32, phase: phase);
    set total = total +wrap escaped;
    set phase = fadd.strict(phase, 0.0625_f64);
    set i = i +wrap 1_u64;
  }
  let wide = cvt<u32, u64>(total);
  let report = buffer_new(17_u64, 10_u8);
  region 'summary {
    let first = spell_hex<'summary>(destination: &uniq 'summary report, at: 0_u64, value: wide);
  }
  region 'o {
    region 's {
      match write_once<'o, 's>(output: &uniq 'o out, source: &'s report, start: 0_u64, end: 17_u64) {
        Ok(value: next) => {
          return exit_status(code: 0_u8);
        }
        Err(error: problem) => {
          return exit_status(code: 1_u8);
        }
      }
    }
  }
}
WFEOF
  exit 0
fi

if [ "$SHAPE" = "bal" ]; then
  BUILD_FN=$(cat <<'WFEOF'
fn build(depth: own u64, w: own f64) -> result: own box<LNode> allocates(heap) {
  doc "Builds the balanced binary box tree. The two child constructions are an eligible pair: disjoint fresh subtrees, no dataflow between them.";
  let done = ieq(depth, 0_u64);
  if done {
    return boxed_leaf(w: w);
  }
  let next = depth -wrap 1_u64;
  let wl = fmul.strict(w, 1.0009765625_f64);
  let wr = fmul.strict(w, 0.9990234375_f64);
  let l = build(depth: next, w: wl);
  let r = build(depth: next, w: wr);
  let branch = Branch(left: move l, right: move r, w: w, h: 0.25_f64, out: 0.0_f64);
  return box_new(move branch);
}
WFEOF
)
  BUILD_CALL="let tree = build(depth: ${DEPTH}_u64, w: 512.0_f64);"
else
  BUILD_FN=$(cat <<'WFEOF'
fn build(depth: own u64, w: own f64, phase: own u64) -> result: own box<LNode> allocates(heap) {
  doc "Builds the deterministically skewed binary box tree: at every other level the right subtree is three levels shallower than the left, saturating at a leaf.";
  let done = ieq(depth, 0_u64);
  if done {
    return boxed_leaf(w: w);
  }
  let next = depth -wrap 1_u64;
  let right_depth = next;
  let shallow = ieq(phase, 0_u64);
  if shallow {
    let deep = ige(next, 3_u64);
    if deep {
      set right_depth = next -wrap 3_u64;
    } else {
      set right_depth = 0_u64;
    }
  }
  let next_phase = 1_u64 -wrap phase;
  let wl = fmul.strict(w, 1.0009765625_f64);
  let wr = fmul.strict(w, 0.9990234375_f64);
  let l = build(depth: next, w: wl, phase: next_phase);
  let r = build(depth: right_depth, w: wr, phase: next_phase);
  let branch = Branch(left: move l, right: move r, w: w, h: 0.25_f64, out: 0.0_f64);
  return box_new(move branch);
}
WFEOF
)
  BUILD_CALL="let tree = build(depth: ${DEPTH}_u64, w: 512.0_f64, phase: 0_u64);"
fi

cat > "$OUT" <<WFEOF
enum LNode {
  Leaf(w: f64, h: f64, out: f64);
  Branch(left: box<LNode>, right: box<LNode>, w: f64, h: f64, out: f64);
}

fn boxed_leaf(w: own f64) -> result: own box<LNode> allocates(heap) {
  let leaf = Leaf(w: w, h: 0.5_f64, out: 0.0_f64);
  return box_new(move leaf);
}

${BUILD_FN}

fn cascade(inh: own f64, w: own f64, h: own f64) -> result: own f64 pure {
  doc "One node's float cascade: the per-node style and box arithmetic of the layout pass.";
  let pad_l = fmul.strict(w, 0.0625_f64);
  let pad_r = fmul.strict(w, 0.03125_f64);
  let border = fmul.strict(h, 0.125_f64);
  let lr = fadd.strict(pad_l, pad_r);
  let lrb = ffma.strict(border, 2.0_f64, lr);
  let content = fsub.strict(w, lrb);
  let clamped_lo = fmax(content, 0.0_f64);
  let font = ffma.strict(inh, 0.5_f64, 8.0_f64);
  let line = fmul.strict(font, 1.25_f64);
  let words = fdiv.strict(clamped_lo, font);
  let lines_raw = fdiv.strict(words, 16.0_f64);
  let lines = fceil(lines_raw);
  let height = fmul.strict(lines, line);
  let margin = fmul.strict(h, 0.5_f64);
  let outer = fadd.strict(height, margin);
  let scaled = ffma.strict(outer, 1.0625_f64, border);
  let abs_scaled = fabs(scaled);
  let root = fsqrt.strict(abs_scaled);
  let mixed = ffma.strict(root, 3.25_f64, clamped_lo);
  let capped = fmin(mixed, 4096.0_f64);
  let floored = fmax(capped, 0.0625_f64);
  let rounded = ffloor(floored);
  let frac = fsub.strict(floored, rounded);
  let adj = ffma.strict(frac, 0.5_f64, rounded);
  let final_h = fadd.strict(adj, lrb);
  return final_h;
}

fn measure_words['w](words: &'w buffer<f64>, font: own f64) -> result: own f64 reads(words) {
  doc "Measures every word of the shared metric table at one font size. The walk is bounded by the table's own length, so its index obligation and every other partial operation are discharged statically.";
  let widest = 0.0_f64;
  let total = 0.0_f64;
  let room = len(deref(words));
  let k = 0_u64;
  loop @scan {
    let done = ige(k, room);
    if done {
      break @scan;
    }
    let raw = deref(words)[k];
    let scaled = fmul.strict(raw, font);
    let above = fgt(scaled, widest);
    if above {
      set widest = scaled;
    }
    set total = fadd.strict(total, scaled);
    set k = k +wrap 1_u64;
  }
  return fadd.strict(total, widest);
}

fn layout['b, 'w](node: &uniq 'b box<LNode>, words: &'w buffer<f64>, inh: own f64) -> result: own f64 reads(node, words), writes(node) {
  doc "Folds the box tree, writing each node's resolved height into its own slot. The two child calls are the permitted and actualizable pair.";
  match deref(deref(node)) {
    Leaf(w: lw, h: lh, out: lslot) => {
      let v = cascade(inh: inh, w: deref(lw), h: deref(lh));
      let m = measure_words<'w>(words: words, font: v);
      let total = fadd.strict(v, m);
      set deref(lslot) = total;
      return total;
    }
    Branch(left: l, right: r, w: bw, h: bh, out: bslot) => {
      let own_h = cascade(inh: inh, w: deref(bw), h: deref(bh));
      let m = measure_words<'w>(words: words, font: own_h);
      let child_inh = fmul.strict(own_h, 0.5_f64);
      let a = layout<'b, 'w>(node: move l, words: words, inh: child_inh);
      let b = layout<'b, 'w>(node: move r, words: words, inh: child_inh);
      let kids = fadd.strict(a, b);
      let mine = fadd.strict(own_h, m);
      let total = fadd.strict(kids, mine);
      set deref(bslot) = total;
      return total;
    }
  }
}

fn hex_digit(v: own u64) -> result: own u8 pure {
  let nibble = iand(v, 15_u64);
  let small = ilt(nibble, 10_u64);
  match cvt<u64, u8>(nibble) {
    Ok(value: byte) => {
      if small {
        return byte +wrap 48_u8;
      }
      return byte +wrap 87_u8;
    }
    Err(error: wide) => {
      return 63_u8;
    }
  }
}

fn spell_hex['d](destination: &uniq 'd buffer<u8>, at: own u64, value: own u64) -> result: own u64 reads(destination), writes(destination) {
  doc "Writes one 64-bit value as sixteen lowercase hexadecimal digits, most significant first.";
  let cursor = at;
  let rest = value;
  let limit = at +wrap 16_u64;
  loop @digits {
    let done = ige(cursor, limit);
    if done {
      break @digits;
    }
    let shifted = irotl(rest, 4_u32);
    let digit = hex_digit(v: shifted);
    let room = len(deref(destination));
    let writable = ilt(cursor, room);
    if writable {
      set deref(destination)[cursor] = digit;
    }
    set rest = shifted;
    set cursor = cursor +wrap 1_u64;
  }
  return limit;
}

command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  doc "Lays out one box tree many times and publishes the exact bits of the fold as hexadecimal.";
  ${BUILD_CALL}
  let words = buffer_new(${WORDS}_u64, 0.25_f64);
  let room = len(words);
  let width = 6.5_f64;
  let w = 0_u64;
  loop @fill {
    let done = ige(w, room);
    if done {
      break @fill;
    }
    set words[w] = width;
    set width = fmul.strict(width, 1.0625_f64);
    let too_wide = fgt(width, 64.0_f64);
    if too_wide {
      set width = 6.5_f64;
    }
    set w = w +wrap 1_u64;
  }
  let i = 0_u64;
  let last = 0.0_f64;
  let seed = 16.0_f64;
  loop @reps {
    let done = ieq(i, ${REPS}_u64);
    if done {
      break @reps;
    }
    region 'tree {
      region 'words {
        let total = layout<'tree, 'words>(node: &uniq 'tree tree, words: &'words words, inh: seed);
        set last = total;
      }
    }
    set seed = fadd.strict(seed, 0.0625_f64);
    set i = i +wrap 1_u64;
  }
  let last_bits = reinterpret<f64, u64>(last);
  let report = buffer_new(17_u64, 10_u8);
  region 'summary {
    let first = spell_hex<'summary>(destination: &uniq 'summary report, at: 0_u64, value: last_bits);
  }
  region 'o {
    region 's {
      match write_once<'o, 's>(output: &uniq 'o out, source: &'s report, start: 0_u64, end: 17_u64) {
        Ok(value: next) => {
          return exit_status(code: 0_u8);
        }
        Err(error: problem) => {
          return exit_status(code: 1_u8);
        }
      }
    }
  }
}
WFEOF

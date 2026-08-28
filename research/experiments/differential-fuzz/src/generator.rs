//! The program generator.
//!
//! Every form it emits is one production of the [GRAM-4] statement fence or the
//! [GRAM-5] expression fence, chosen under a typing and ownership environment
//! the generator carries: which scalars and buffers are live, which entry
//! inputs exist, which regions are open, how deep the nesting is. That is what
//! "grammar-driven" has to mean for a language whose acceptance is a borrow and
//! effect judgment -- a free derivation over the fence produces well-formed
//! text that is rejected essentially always, and a fuzzer whose programs are
//! rejected tests the parser and nothing else.
//!
//! The environment is what makes the emitted program *canonical* rather than
//! merely parseable: subscripts are either constant against a known buffer
//! length or the binder of a `for` whose upper endpoint is that length,
//! divisors are nonzero literals, every borrow is taken inside a region the
//! generator opened, every affine value is consumed once, and the entry's
//! effect row is computed from what the body actually exhibited rather than
//! guessed. Acceptance is still verified by the compiler, never assumed: the
//! campaign counts every rejection by the rule its diagnostic cites, which is
//! how a bias in this file becomes visible instead of silent.
//!
//! The shape weights lean toward what [PAR-1], [PAR-2], and [PAR-3] can permit
//! and toward their exact boundaries, because a permission that is never
//! granted and a permission that is wrongly granted are both invisible to a
//! generator that only writes the easy middle.

use std::collections::BTreeSet;

use crate::rng::Rng;

/// The named shapes a program contains. The campaign reports the distribution,
/// so a shape that stopped being generated -- because a compiler change started
/// rejecting it -- shows up as a zero rather than as silence.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Shape {
    Arithmetic,
    Branch,
    CountedLoop,
    AccumulatorLoop,
    UnboundedLoop,
    NestedLoop,
    StdoutWrite,
    StderrWrite,
    BulkWrite,
    IndependentPair,
    SameOutputPair,
    PureCallPair,
    SharedSourcePair,
    ReadThenWriteBuffer,
    FileLoopIterationOwn,
    FileLoopHoistedScratch,
    FileLoopBreakAfterSubmission,
    FileLoopSharedScratch,
    DirectoryScan,
    Claim,
    GiveMatch,
    SliceView,
    TypedExit,
    ArgumentRead,
}

impl Shape {
    pub fn spelling(self) -> &'static str {
        match self {
            Shape::Arithmetic => "arithmetic",
            Shape::Branch => "branch",
            Shape::CountedLoop => "counted-loop",
            Shape::AccumulatorLoop => "accumulator-loop",
            Shape::UnboundedLoop => "unbounded-loop",
            Shape::NestedLoop => "nested-loop",
            Shape::StdoutWrite => "stdout-write",
            Shape::StderrWrite => "stderr-write",
            Shape::BulkWrite => "bulk-write",
            Shape::IndependentPair => "independent-pair",
            Shape::SameOutputPair => "same-output-pair",
            Shape::PureCallPair => "pure-call-pair",
            Shape::SharedSourcePair => "shared-source-pair",
            Shape::ReadThenWriteBuffer => "read-then-write-buffer",
            Shape::FileLoopIterationOwn => "file-loop-iteration-own",
            Shape::FileLoopHoistedScratch => "file-loop-hoisted-scratch",
            Shape::FileLoopBreakAfterSubmission => "file-loop-break-after-submission",
            Shape::FileLoopSharedScratch => "file-loop-shared-scratch",
            Shape::DirectoryScan => "directory-scan",
            Shape::Claim => "claim",
            Shape::GiveMatch => "give-match",
            Shape::SliceView => "slice-view",
            Shape::TypedExit => "typed-exit",
            Shape::ArgumentRead => "argument-read",
        }
    }
}

pub struct Program {
    pub source: String,
    pub shapes: Vec<Shape>,
    /// The program publishes more than one host pipe buffer of bytes, so a
    /// delayed FIFO reader makes its writes genuinely wait rather than merely
    /// arrive late.
    pub bulk_output: bool,
}

/// Text under construction, with the indentation the emitted source carries.
struct Emit {
    text: String,
    indent: usize,
}

impl Emit {
    fn new() -> Self {
        Self {
            text: String::new(),
            indent: 1,
        }
    }

    fn line(&mut self, text: &str) {
        for _ in 0..self.indent {
            self.text.push_str("  ");
        }
        self.text.push_str(text);
        self.text.push('\n');
    }

    fn open(&mut self, text: &str) {
        self.line(text);
        self.indent += 1;
    }

    fn close(&mut self) {
        self.indent -= 1;
        self.line("}");
    }
}

/// One live `u64` binding usable as an operand.
type Scalar = String;

/// One live `buffer<u8>` binding and its statically known length, which is what
/// makes a constant subscript provable without a guard.
struct Buffer {
    name: String,
    length: u64,
}

struct Gen {
    rng: Rng,
    body: Emit,
    next_name: u32,
    next_label: u32,
    next_region: u32,
    depth: usize,
    statements: usize,
    limit: usize,

    have_err: bool,
    have_files: bool,
    have_args: bool,

    used_err: bool,
    used_files: bool,
    used_args: bool,
    allocates: bool,
    traps: bool,

    need_fold: bool,
    need_render: bool,
    need_mix: bool,
    need_view: bool,

    scalars: Vec<Scalar>,
    buffers: Vec<Buffer>,
    shapes: BTreeSet<Shape>,
    bulk_output: bool,
}

pub fn generate(seed: u64) -> Program {
    let mut rng = Rng::new(seed);
    let have_err = rng.chance(55);
    let have_files = rng.chance(65);
    let have_args = rng.chance(30);
    let generator = Gen {
        rng,
        body: Emit::new(),
        next_name: 0,
        next_label: 0,
        next_region: 0,
        depth: 0,
        statements: 0,
        limit: 220,
        have_err,
        have_files,
        have_args,
        used_err: false,
        used_files: false,
        used_args: false,
        allocates: false,
        traps: false,
        need_fold: false,
        need_render: true,
        need_mix: false,
        need_view: false,
        scalars: Vec::new(),
        buffers: Vec::new(),
        shapes: BTreeSet::new(),
        bulk_output: false,
    };
    generator.program()
}

impl Gen {
    fn name(&mut self, stem: &str) -> String {
        self.next_name += 1;
        format!("{stem}_{}", self.next_name)
    }

    fn label(&mut self) -> String {
        self.next_label += 1;
        format!("@walk_{}", self.next_label)
    }

    fn region(&mut self) -> String {
        self.next_region += 1;
        format!("'zone_{}", self.next_region)
    }

    fn budget(&self) -> bool {
        self.statements < self.limit
    }

    fn spend(&mut self) {
        self.statements += 1;
    }

    /// A `u64` atom: a live scalar or a literal. Every operand position the
    /// generator fills is one `atom` of [GRAM-5], never a nested expression,
    /// because the fence admits none.
    fn scalar_atom(&mut self) -> String {
        if self.scalars.is_empty() || self.rng.chance(30) {
            format!("{}_u64", self.rng.between(0, 97))
        } else {
            let index = self.rng.below(self.scalars.len() as u64) as usize;
            self.scalars[index].clone()
        }
    }

    fn byte_literal(&mut self) -> String {
        format!("{}_u8", self.rng.between(32, 126))
    }

    /// A `Bool` expression: one comparison call over two atoms.
    fn condition(&mut self) -> String {
        let left = self.scalar_atom();
        let right = self.scalar_atom();
        let op = *self.rng.pick(&["ilt", "ile", "igt", "ige", "ieq", "ine"]);
        format!("{op}({left}, {right})")
    }

    fn program(mut self) -> Program {
        self.body.line("doc \"A generated differential-fuzz program: real I/O, real control flow, one published digest.\";");
        self.body.line("let total = 0_u64;");
        self.scalars.push("total".to_owned());

        let blocks = self.rng.between(3, 7);
        for _ in 0..blocks {
            if !self.budget() {
                break;
            }
            self.block();
        }
        self.publish_digest();

        let mut source = String::new();
        if self.need_mix {
            source.push_str(MIX_HELPER);
            source.push('\n');
        }
        if self.need_fold {
            source.push_str(FOLD_HELPER);
            source.push('\n');
        }
        if self.need_view {
            source.push_str(VIEW_HELPER);
            source.push('\n');
        }
        if self.need_render {
            source.push_str(RENDER_HELPER);
            source.push('\n');
        }
        source.push_str(&self.header());
        source.push_str(&self.body.text);
        source.push_str("}\n");

        Program {
            source,
            shapes: self.shapes.iter().copied().collect(),
            bulk_output: self.bulk_output,
        }
    }

    /// The entry header, declaring exactly the inputs the body used and the
    /// exact row it exhibited, in [EFF-1] canonical order. `command.cwd`
    /// carries `writes(cwd)` whenever it is declared, because the entry's
    /// normal return edge performs the directory state's compiler-derived
    /// close whether the body touched it or not.
    fn header(&self) -> String {
        let mut inputs = Vec::new();
        let mut reads = Vec::new();
        let mut writes = Vec::new();
        if self.used_args {
            inputs.push("command.args as args: own Args");
            reads.push("args");
        }
        if self.used_files {
            inputs.push("command.cwd as cwd: own DirectoryRead");
            reads.push("cwd");
            writes.push("cwd");
        }
        inputs.push("command.stdout as out: own Output");
        reads.push("out");
        writes.push("out");
        if self.used_err {
            inputs.push("command.stderr as err: own Output");
            reads.push("err");
            writes.push("err");
        }
        if self.used_files {
            inputs.push("command.files as files: own FileFactory");
            reads.push("files");
            writes.push("files");
        }
        let mut row = vec![
            format!("reads({})", reads.join(", ")),
            format!("writes({})", writes.join(", ")),
        ];
        if self.allocates {
            row.push("allocates(heap)".to_owned());
        }
        if self.traps {
            row.push("traps".to_owned());
        }
        format!(
            "command fn main({}) -> status: own ExitStatus {} {{\n",
            inputs.join(", "),
            row.join(", ")
        )
    }

    /// Renders the accumulated total as a fixed twenty-digit line and publishes
    /// it, then leaves through an exit status derived from the same total. Both
    /// observables therefore depend on every statement the program executed,
    /// which is what makes byte equality a real oracle rather than a check that
    /// two runs both printed nothing.
    fn publish_digest(&mut self) {
        let line = self.name("digest");
        let cursor = self.name("cursor");
        let render = self.region();
        let publish = self.region();
        self.allocates = true;
        self.body
            .line(&format!("let {line} = buffer_new(32_u64, 32_u8);"));
        self.body.line(&format!("let {cursor} = 0_u64;"));
        self.body.open(&format!("region {render} {{"));
        self.body.line(&format!(
            "set {cursor} = render_u64<{render}>(destination: &uniq {render} {line}, at: 0_u64, value: total);"
        ));
        self.body.close();
        self.body.line(&format!("set {line}[20_u64] = 10_u8;"));
        let outcome = self.name("published");
        self.body.open(&format!("region {publish} {{"));
        self.body.open(&format!(
            "match write_once<{publish}, {publish}>(output: &uniq {publish} out, source: &{publish} {line}, start: 0_u64, end: 21_u64) {{"
        ));
        self.body.open(&format!("Ok(value: {outcome}) => {{"));
        self.body.close();
        let failure = self.name("failed");
        self.body.open(&format!("Err(error: {failure}) => {{"));
        self.body.close();
        self.body.close();
        self.body.close();

        let wide = self.name("code_wide");
        let narrow = self.name("code_narrow");
        let code = self.name("code");
        let exact = self.name("code_exact");
        let failed = self.name("code_failed");
        self.body.line(&format!("let {wide} = total % 251_u64;"));
        self.body
            .line(&format!("let {narrow} = cvt<u64, u8>({wide});"));
        self.body.open(&format!("let {code} = match {narrow} {{"));
        self.body.open(&format!("Ok(value: {exact}) => {{"));
        self.body.line(&format!("give {exact};"));
        self.body.close();
        self.body.open(&format!("Err(error: {failed}) => {{"));
        self.body.line("give 9_u8;");
        self.body.close();
        self.body.close();
        self.body
            .line(&format!("return exit_status(code: {code});"));
    }
}

const MIX_HELPER: &str = r#"fn mix(a: own u64, b: own u64) -> result: own u64 pure {
  doc "Mixes two scalars into one with total operations and no effect of any kind.";
  let scaled = a *wrap 2654435761_u64;
  let added = scaled +wrap b;
  let folded = ixor(added, 1099511628211_u64);
  return folded;
}
"#;

const FOLD_HELPER: &str = r#"fn fold_prefix['s](source: &'s buffer<u8>, produced: own u64, seed: own u64) -> result: own u64 reads(source) {
  doc "Folds one prefix of a buffer into a running order-sensitive checksum.";
  let room = len(deref(source));
  let sum = seed;
  let at = 0_u64;
  loop @fold {
    let scanned = ige(at, produced);
    if scanned {
      break @fold;
    }
    let readable = ilt(at, room);
    if readable {
    } else {
      break @fold;
    }
    let byte = deref(source)[at];
    let widened = cvt<u8, u64>(byte);
    set sum = sum *wrap 31_u64;
    set sum = sum +wrap widened;
    set at = at +wrap 1_u64;
  }
  return sum;
}
"#;

const VIEW_HELPER: &str = r#"fn fold_view['r](view: own slice<'r, u8>, produced: own u64, seed: own u64) -> result: own u64 reads(view) {
  doc "Folds one prefix of a direct view into a running order-sensitive checksum.";
  let room = len(view);
  let sum = seed;
  let at = 0_u64;
  loop @scan {
    let scanned = ige(at, produced);
    if scanned {
      break @scan;
    }
    let readable = ilt(at, room);
    if readable {
    } else {
      break @scan;
    }
    let byte = view[at];
    let widened = cvt<u8, u64>(byte);
    set sum = sum *wrap 31_u64;
    set sum = sum +wrap widened;
    set at = at +wrap 1_u64;
  }
  return sum;
}
"#;

const RENDER_HELPER: &str = r#"fn render_u64['d](destination: &uniq 'd buffer<u8>, at: own u64, value: own u64) -> result: own u64 reads(destination), writes(destination) {
  doc "Renders one twenty-digit zero-padded decimal number and reports the position after it.";
  let room = len(deref(destination));
  let remaining = value;
  let position = at +wrap 20_u64;
  loop @digits {
    let done = ile(position, at);
    if done {
      break @digits;
    }
    set position = position -wrap 1_u64;
    let digit = remaining % 10_u64;
    let narrowed = cvt<u64, u8>(digit);
    let byte = match narrowed {
      Ok(value: exact) => {
        give 48_u8 +wrap exact;
      }
      Err(error: failure) => {
        give 48_u8;
      }
    }
    let writable = ilt(position, room);
    if writable {
      set deref(destination)[position] = byte;
    }
    set remaining = remaining / 10_u64;
  }
  return at +wrap 20_u64;
}
"#;

/// The shape catalog. Each method emits one complete, self-contained group of
/// statements and leaves the environment consistent: every binding it adds is
/// live, every affine value it creates is consumed, every region it opens is
/// closed, and every subscript it writes is provable from a length the
/// generator knows.
impl Gen {
    fn block(&mut self) {
        let mut options: Vec<(Shape, u64)> = vec![
            (Shape::Arithmetic, 9),
            (Shape::Branch, 9),
            (Shape::CountedLoop, 9),
            (Shape::AccumulatorLoop, 9),
            (Shape::UnboundedLoop, 7),
            (Shape::StdoutWrite, 9),
            (Shape::BulkWrite, 3),
            (Shape::SameOutputPair, 8),
            (Shape::PureCallPair, 7),
            (Shape::Claim, 7),
            (Shape::GiveMatch, 6),
            (Shape::SliceView, 6),
            (Shape::TypedExit, 3),
        ];
        if self.have_err {
            options.push((Shape::StderrWrite, 8));
            options.push((Shape::IndependentPair, 11));
            options.push((Shape::SharedSourcePair, 6));
        }
        if self.have_files {
            options.push((Shape::FileLoopIterationOwn, 20));
            options.push((Shape::FileLoopHoistedScratch, 6));
            options.push((Shape::FileLoopBreakAfterSubmission, 4));
            options.push((Shape::FileLoopSharedScratch, 4));
            options.push((Shape::ReadThenWriteBuffer, 6));
            options.push((Shape::DirectoryScan, 5));
        }
        if self.have_args {
            options.push((Shape::ArgumentRead, 5));
        }
        let total: u64 = options.iter().map(|(_, weight)| weight).sum();
        let mut draw = self.rng.below(total);
        let mut chosen = options[0].0;
        for (shape, weight) in &options {
            if draw < *weight {
                chosen = *shape;
                break;
            }
            draw -= weight;
        }
        self.shapes.insert(chosen);
        match chosen {
            Shape::Arithmetic => self.arithmetic_block(),
            Shape::Branch => self.branch_block(),
            Shape::CountedLoop => self.counted_loop_block(),
            Shape::AccumulatorLoop => self.accumulator_loop_block(),
            Shape::UnboundedLoop => self.unbounded_loop_block(),
            Shape::StdoutWrite => self.write_block(false),
            Shape::StderrWrite => self.write_block(true),
            Shape::BulkWrite => self.bulk_write_block(),
            Shape::IndependentPair => self.output_pair_block(true, false),
            Shape::SameOutputPair => self.output_pair_block(false, false),
            Shape::SharedSourcePair => self.output_pair_block(true, true),
            Shape::PureCallPair => self.pure_call_pair_block(),
            Shape::ReadThenWriteBuffer => self.read_then_write_block(),
            Shape::FileLoopIterationOwn => self.file_loop_block(FileLoop::IterationOwn),
            Shape::FileLoopHoistedScratch => self.file_loop_block(FileLoop::HoistedScratch),
            Shape::FileLoopBreakAfterSubmission => {
                self.file_loop_block(FileLoop::BreakAfterSubmission)
            }
            Shape::FileLoopSharedScratch => self.file_loop_block(FileLoop::SharedScratch),
            Shape::DirectoryScan => self.directory_block(),
            Shape::Claim => self.claim_block(),
            Shape::GiveMatch => self.give_match_block(),
            Shape::SliceView => self.slice_view_block(),
            Shape::TypedExit => self.typed_exit_block(),
            Shape::ArgumentRead => self.argument_block(),
            Shape::NestedLoop => self.counted_loop_block(),
        }
    }

    /// Saves the binding environment across a nested block, because a `let`
    /// inside a block is not in scope after it.
    fn scope<F: FnOnce(&mut Self)>(&mut self, inner: F) {
        let scalars = self.scalars.len();
        let buffers = self.buffers.len();
        self.depth += 1;
        inner(self);
        self.depth -= 1;
        self.scalars.truncate(scalars);
        self.buffers.truncate(buffers);
    }

    fn declare_buffer(&mut self, length: u64, fill: u64) -> String {
        let name = self.name("store");
        self.allocates = true;
        self.spend();
        self.body.line(&format!(
            "let {name} = buffer_new({length}_u64, {fill}_u8);"
        ));
        self.buffers.push(Buffer {
            name: name.clone(),
            length,
        });
        name
    }

    /// One fixture file name, `fNN.dat`, written byte by byte at constant
    /// positions of a length-8 buffer, so every subscript is provable and the
    /// name range is exactly seven bytes.
    fn declare_name(&mut self, digit: Option<&str>) -> String {
        let name = self.declare_buffer(8, 0);
        let fixed: u64 = self.rng.below(8);
        self.body.line(&format!("set {name}[0_u64] = 102_u8;"));
        self.body.line(&format!("set {name}[1_u64] = 48_u8;"));
        match digit {
            Some(index) => {
                let modulus = self.name("slot");
                let narrow = self.name("slot_narrow");
                let byte = self.name("slot_byte");
                let exact = self.name("slot_exact");
                let failed = self.name("slot_failed");
                self.body.line(&format!("let {modulus} = {index} % 8_u64;"));
                self.body
                    .line(&format!("let {narrow} = cvt<u64, u8>({modulus});"));
                self.body.open(&format!("let {byte} = match {narrow} {{"));
                self.body.open(&format!("Ok(value: {exact}) => {{"));
                self.body.line(&format!("give 48_u8 +wrap {exact};"));
                self.body.close();
                self.body.open(&format!("Err(error: {failed}) => {{"));
                self.body.line("give 48_u8;");
                self.body.close();
                self.body.close();
                self.body.line(&format!("set {name}[2_u64] = {byte};"));
            }
            None => {
                self.body
                    .line(&format!("set {name}[2_u64] = {}_u8;", 48 + fixed));
            }
        }
        self.body.line(&format!("set {name}[3_u64] = 46_u8;"));
        self.body.line(&format!("set {name}[4_u64] = 100_u8;"));
        self.body.line(&format!("set {name}[5_u64] = 97_u8;"));
        self.body.line(&format!("set {name}[6_u64] = 116_u8;"));
        name
    }

    fn arithmetic_block(&mut self) {
        let count = self.rng.between(2, 5);
        for _ in 0..count {
            self.arithmetic_statement();
        }
    }

    fn arithmetic_statement(&mut self) {
        self.spend();
        let name = self.name("value");
        let left = self.scalar_atom();
        let choice = self.rng.below(10);
        let text = match choice {
            0 => {
                let right = self.scalar_atom();
                format!("let {name} = {left} +wrap {right};")
            }
            1 => {
                let right = self.scalar_atom();
                format!("let {name} = {left} -wrap {right};")
            }
            2 => {
                let right = self.scalar_atom();
                format!("let {name} = {left} *wrap {right};")
            }
            3 => {
                let divisor = self.rng.between(2, 97);
                format!("let {name} = {left} % {divisor}_u64;")
            }
            4 => {
                let divisor = self.rng.between(2, 97);
                format!("let {name} = {left} / {divisor}_u64;")
            }
            5 => {
                let right = self.scalar_atom();
                format!("let {name} = iand({left}, {right});")
            }
            6 => {
                let right = self.scalar_atom();
                format!("let {name} = ior({left}, {right});")
            }
            7 => {
                let right = self.scalar_atom();
                format!("let {name} = ixor({left}, {right});")
            }
            8 => {
                let right = self.scalar_atom();
                format!("let {name} = imin({left}, {right});")
            }
            _ => {
                let right = self.scalar_atom();
                format!("let {name} = imax({left}, {right});")
            }
        };
        self.body.line(&text);
        self.scalars.push(name.clone());
        if self.rng.chance(70) {
            self.spend();
            self.body.line(&format!("set total = total +wrap {name};"));
        }
    }

    /// A handful of statements for a nested body: arithmetic, an accumulator
    /// update, a guarded subscript, or one more level of branching.
    fn inner_statements(&mut self, index: Option<String>) {
        let count = self.rng.between(1, 4);
        for _ in 0..count {
            if !self.budget() {
                return;
            }
            match self.rng.below(10) {
                0..=4 => self.arithmetic_statement(),
                5..=6 => {
                    self.spend();
                    let atom = self.scalar_atom();
                    self.body.line(&format!("set total = total +wrap {atom};"));
                }
                7..=8 => self.guarded_subscript(index.clone()),
                _ => {
                    if self.depth < 3 {
                        self.branch_block();
                    } else {
                        self.arithmetic_statement();
                    }
                }
            }
        }
    }

    /// A subscript whose bound is established by an explicit `len` comparison,
    /// which is the writer form [OP-4] admits when the index is not the binder
    /// of a `for` over exactly that length.
    fn guarded_subscript(&mut self, index: Option<String>) {
        if self.buffers.is_empty() {
            self.arithmetic_statement();
            return;
        }
        let slot = self.rng.below(self.buffers.len() as u64) as usize;
        let target = self.buffers[slot].name.clone();
        let length = self.buffers[slot].length;
        // A constant position below a length the generator knows needs no
        // guard: [OP-9] publishes the length `buffer_new` established and the
        // literal discharges the [OP-4] obligation on its own.
        if index.is_none() && length > 0 && self.rng.chance(40) {
            let position = self.rng.below(length);
            let byte = self.byte_literal();
            self.spend();
            self.body
                .line(&format!("set {target}[{position}_u64] = {byte};"));
            return;
        }
        let position = match index {
            Some(binder) => binder,
            None => self.scalar_atom(),
        };
        let room = self.name("room");
        let ok = self.name("writable");
        let byte = self.byte_literal();
        self.spend();
        self.body.line(&format!("let {room} = len({target});"));
        self.body
            .line(&format!("let {ok} = ilt({position}, {room});"));
        self.body.open(&format!("if {ok} {{"));
        self.body
            .line(&format!("set {target}[{position}] = {byte};"));
        self.body.close();
    }

    fn branch_block(&mut self) {
        let flag = self.name("flag");
        let condition = self.condition();
        self.spend();
        self.body.line(&format!("let {flag} = {condition};"));
        self.body.open(&format!("if {flag} {{"));
        self.scope(|generator| generator.inner_statements(None));
        if self.rng.chance(60) {
            self.body.indent -= 1;
            self.body.line("} else {");
            self.body.indent += 1;
            self.scope(|generator| generator.inner_statements(None));
        }
        self.body.close();
    }

    fn counted_loop_block(&mut self) {
        let length = *self.rng.pick(&[8_u64, 16, 24, 32, 48, 64]);
        let carry = self.rng.chance(60);
        let target = if carry {
            let fill = self.rng.between(48, 90);
            Some(self.declare_buffer(length, fill))
        } else {
            None
        };
        let label = self.label();
        let binder = self.name("step");
        self.spend();
        self.body
            .open(&format!("for {label} {binder} in 0_u64..{length}_u64 {{"));
        let inner = binder.clone();
        self.scope(|generator| {
            generator.scalars.push(inner.clone());
            if let Some(store) = &target {
                // The binder's upper endpoint is exactly this buffer's length,
                // so [ENT-3]'s structural fact discharges the subscript with no
                // written guard: the shape P11 names.
                let byte = generator.byte_literal();
                generator.spend();
                generator
                    .body
                    .line(&format!("set {store}[{inner}] = {byte};"));
            }
            generator.inner_statements(Some(inner.clone()));
            if generator.depth < 3 && generator.rng.chance(25) && generator.budget() {
                generator.shapes.insert(Shape::NestedLoop);
                generator.nested_counted_loop();
            }
            generator.spend();
            generator
                .body
                .line(&format!("set total = total +wrap {inner};"));
        });
        self.body.close();
        if let Some(store) = target {
            if self.rng.chance(55) {
                self.fold_buffer(&store, length);
            }
        }
    }

    /// The one loop shape [PAR-2] can permit: exactly one place rooted outside
    /// the loop, written by exactly one `set` under one fixed associative,
    /// commutative operation with an identity, and no other occurrence of that
    /// binding anywhere in the body. Every other statement reads only the
    /// binder and literals, so nothing else leaves the iteration.
    fn accumulator_loop_block(&mut self) {
        let bound = *self.rng.pick(&[8_u64, 16, 32, 64]);
        let label = self.label();
        let binder = self.name("counted");
        let infix = self.rng.chance(50);
        let operation = if infix {
            *self.rng.pick(&["+wrap", "*wrap"])
        } else {
            *self.rng.pick(&["iand", "ior", "ixor", "imin", "imax"])
        };
        self.spend();
        self.body
            .open(&format!("for {label} {binder} in 0_u64..{bound}_u64 {{"));
        let mut carrier = binder.clone();
        let steps = self.rng.between(1, 3);
        for _ in 0..steps {
            let name = self.name("term");
            let literal = self.rng.between(1, 97);
            let form = match self.rng.below(5) {
                0 => format!("let {name} = {carrier} +wrap {literal}_u64;"),
                1 => format!("let {name} = {carrier} *wrap {literal}_u64;"),
                2 => format!("let {name} = {carrier} % {literal}_u64;"),
                3 => format!("let {name} = ixor({carrier}, {literal}_u64);"),
                _ => format!("let {name} = imax({carrier}, {literal}_u64);"),
            };
            self.spend();
            self.body.line(&form);
            carrier = name;
        }
        if infix {
            self.body
                .line(&format!("set total = total {operation} {carrier};"));
        } else {
            self.body
                .line(&format!("set total = {operation}(total, {carrier});"));
        }
        self.body.close();
    }

    fn nested_counted_loop(&mut self) {
        let bound = *self.rng.pick(&[4_u64, 6, 8]);
        let label = self.label();
        let binder = self.name("inner");
        self.spend();
        self.body
            .open(&format!("for {label} {binder} in 0_u64..{bound}_u64 {{"));
        let inner = binder.clone();
        self.scope(|generator| {
            generator.scalars.push(inner.clone());
            generator.inner_statements(None);
            generator.spend();
            generator
                .body
                .line(&format!("set total = total +wrap {inner};"));
        });
        self.body.close();
    }

    fn unbounded_loop_block(&mut self) {
        let bound = *self.rng.pick(&[4_u64, 8, 12, 20]);
        let counter = self.name("tick");
        let label = self.label();
        let done = self.name("done");
        self.spend();
        self.body.line(&format!("let {counter} = 0_u64;"));
        self.body.open(&format!("loop {label} {{"));
        let inner = counter.clone();
        let stop = done.clone();
        let exit = label.clone();
        self.scope(|generator| {
            generator.scalars.push(inner.clone());
            generator
                .body
                .line(&format!("let {stop} = ige({inner}, {bound}_u64);"));
            generator.body.open(&format!("if {stop} {{"));
            generator.body.line(&format!("break {exit};"));
            generator.body.close();
            generator.inner_statements(None);
            generator.spend();
            generator
                .body
                .line(&format!("set total = total +wrap {inner};"));
            generator
                .body
                .line(&format!("set {inner} = {inner} +wrap 1_u64;"));
        });
        self.body.close();
    }

    fn fold_buffer(&mut self, store: &str, length: u64) {
        self.need_fold = true;
        let region = self.region();
        let digest = self.name("digest");
        self.spend();
        self.body.open(&format!("region {region} {{"));
        self.body.line(&format!(
            "let {digest} = fold_prefix<{region}>(source: &{region} {store}, produced: {length}_u64, seed: 7_u64);"
        ));
        self.body
            .line(&format!("set total = total +wrap {digest};"));
        self.body.close();
    }

    /// One publication through one `Output`, with both outcomes handled.
    fn write_block(&mut self, to_error: bool) {
        let length = *self.rng.pick(&[4_u64, 8, 16, 24, 32]);
        let fill = self.rng.between(48, 90);
        let store = self.declare_buffer(length, fill);
        let sink = if to_error {
            self.used_err = true;
            "err"
        } else {
            "out"
        };
        let region = self.region();
        let ok = self.name("wrote");
        let failed = self.name("write_failed");
        self.spend();
        self.body.open(&format!("region {region} {{"));
        self.body.open(&format!(
            "match write_once<{region}, {region}>(output: &uniq {region} {sink}, source: &{region} {store}, start: 0_u64, end: {length}_u64) {{"
        ));
        self.body.open(&format!("Ok(value: {ok}) => {{"));
        self.body.line(&format!("set total = total +wrap {ok};"));
        self.body.close();
        self.body.open(&format!("Err(error: {failed}) => {{"));
        self.body.line("set total = total +wrap 3_u64;");
        self.body.close();
        self.body.close();
        self.body.close();
    }

    /// Enough bytes to exceed a host pipe buffer, so a delayed reader on the
    /// other end makes the write genuinely wait.
    fn bulk_write_block(&mut self) {
        let length = *self.rng.pick(&[98304_u64, 131072, 196608]);
        let fill = self.rng.between(65, 90);
        let store = self.declare_buffer(length, fill);
        self.bulk_output = true;
        let region = self.region();
        let ok = self.name("bulk_wrote");
        let failed = self.name("bulk_failed");
        self.spend();
        self.body.open(&format!("region {region} {{"));
        self.body.open(&format!(
            "match write_once<{region}, {region}>(output: &uniq {region} out, source: &{region} {store}, start: 0_u64, end: {length}_u64) {{"
        ));
        self.body.open(&format!("Ok(value: {ok}) => {{"));
        self.body.line(&format!("set total = total +wrap {ok};"));
        self.body.close();
        self.body.open(&format!("Err(error: {failed}) => {{"));
        self.body.line("set total = total +wrap 5_u64;");
        self.body.close();
        self.body.close();
        self.body.close();
    }

    /// Two adjacent publication statements. `independent` sends them to the two
    /// distinct `Output` values, the pair [PAR-1] can permit; otherwise both go
    /// to stdout, where one exclusive loan stands against the other and the
    /// published order is the source order. `shared_source` additionally has
    /// both read one buffer, so two shared loans meet on one place.
    fn output_pair_block(&mut self, independent: bool, shared_source: bool) {
        let length = *self.rng.pick(&[8_u64, 16, 32, 48]);
        let left = self.declare_buffer(length, 65);
        let right = if shared_source {
            left.clone()
        } else {
            self.declare_buffer(length, 66)
        };
        let second_sink = if independent {
            self.used_err = true;
            "err"
        } else {
            "out"
        };
        let outer = self.region();
        let inner = self.region();
        let first = self.name("first");
        let second = self.name("second");
        self.spend();
        self.body.open(&format!("region {outer} {{"));
        self.body.open(&format!("region {inner} {{"));
        self.body.line(&format!(
            "let {first} = write_once<{outer}, {outer}>(output: &uniq {outer} out, source: &{outer} {left}, start: 0_u64, end: {length}_u64);"
        ));
        let source_region = if shared_source { &outer } else { &inner };
        self.body.line(&format!(
            "let {second} = write_once<{inner}, {source_region}>(output: &uniq {inner} {second_sink}, source: &{source_region} {right}, start: 0_u64, end: {length}_u64);"
        ));
        for binding in [first, second] {
            let ok = self.name("reached");
            let failed = self.name("pair_failed");
            self.body.open(&format!("match {binding} {{"));
            self.body.open(&format!("Ok(value: {ok}) => {{"));
            self.body.line(&format!("set total = total +wrap {ok};"));
            self.body.close();
            self.body.open(&format!("Err(error: {failed}) => {{"));
            self.body.line("set total = total +wrap 11_u64;");
            self.body.close();
            self.body.close();
        }
        self.body.close();
        self.body.close();
    }

    /// Two adjacent pure user calls: the cheapest pair [PAR-1] permits, and the
    /// one whose overlap can only be observed through a wrong result.
    fn pure_call_pair_block(&mut self) {
        self.need_mix = true;
        let first = self.name("mixed");
        let second = self.name("mixed");
        let a = self.scalar_atom();
        let b = self.scalar_atom();
        let c = self.scalar_atom();
        let d = self.scalar_atom();
        self.spend();
        self.body
            .line(&format!("let {first} = mix(a: {a}, b: {b});"));
        self.body
            .line(&format!("let {second} = mix(a: {c}, b: {d});"));
        self.body.line(&format!("set total = total +wrap {first};"));
        self.body
            .line(&format!("set total = total +wrap {second};"));
        self.scalars.push(first);
        self.scalars.push(second);
    }
}

/// The four staged-loop variants: one shape [PAR-3] grants and three the rule
/// denies for three different written reasons.
#[derive(Clone, Copy)]
enum FileLoop {
    IterationOwn,
    HoistedScratch,
    BreakAfterSubmission,
    SharedScratch,
}

impl Gen {
    /// A loop that opens and reads one fixture file per iteration. The variant
    /// decides which [PAR-3] condition the loop meets or breaks, and every
    /// variant publishes something that depends on what it read, so a wrongly
    /// granted permission shows up as a wrong digest rather than as nothing.
    fn file_loop_block(&mut self, variant: FileLoop) {
        self.used_files = true;
        self.need_fold = true;
        let rounds = *self.rng.pick(&[4_u64, 6, 8, 12]);
        let window = *self.rng.pick(&[64_u64, 256, 1024, 4096]);

        // The scratch the variant shares across iterations, declared above the
        // loop, is exactly what costs the loop its pipeline.
        let hoisted_data = match variant {
            FileLoop::HoistedScratch => Some(self.declare_buffer(window, 0)),
            _ => None,
        };
        let hoisted_name = match variant {
            FileLoop::SharedScratch => Some(self.declare_name(None)),
            _ => None,
        };

        let label = self.label();
        let binder = self.name("index");
        self.spend();
        self.body
            .open(&format!("for {label} {binder} in 0_u64..{rounds}_u64 {{"));
        let binder_name = binder.clone();
        let loop_label = label.clone();
        self.scope(|generator| {
            generator.scalars.push(binder_name.clone());
            let name = match &hoisted_name {
                Some(existing) => {
                    // The shared name buffer is written every iteration, which
                    // is a write to storage the body does not introduce.
                    let byte = generator.byte_literal();
                    generator
                        .body
                        .line(&format!("set {existing}[7_u64] = {byte};"));
                    existing.clone()
                }
                None => generator.declare_name(Some(&binder_name)),
            };
            let data = match &hoisted_data {
                Some(existing) => existing.clone(),
                None => generator.declare_buffer(window, 0),
            };
            let factory = generator.region();
            let name_region = generator.region();
            let permit = generator.name("permit");
            generator.body.open(&format!("region {factory} {{"));
            generator.body.line(&format!(
                "let {permit} = reserve_file<{factory}>(factory: &uniq {factory} files);"
            ));
            generator.body.open(&format!("region {name_region} {{"));
            generator.body.open(&format!(
                "match open_file<{factory}, {name_region}>(permit: move {permit}, root: &{factory} cwd, name: &{name_region} {name}, start: 0_u64, end: 7_u64) {{"
            ));
            let handle = generator.name("handle");
            generator.body.open(&format!("Ok(value: {handle}) => {{"));
            let file_region = generator.region();
            let data_region = generator.region();
            generator.body.open(&format!("region {file_region} {{"));
            generator.body.open(&format!("region {data_region} {{"));
            generator.body.open(&format!(
                "match read_at<{file_region}, {data_region}>(file: &{file_region} {handle}, destination: &uniq {data_region} {data}, file_offset: 0_u64, start: 0_u64, end: {window}_u64) {{"
            ));
            let produced = generator.name("produced");
            generator
                .body
                .open(&format!("ReadBytes(next: {produced}) => {{"));
            generator
                .body
                .line(&format!("set total = total +wrap {produced};"));
            // Folding the bytes is what makes a shared destination genuinely
            // order-dependent: after a short read the tail is the previous
            // iteration's bytes.
            let fold_region = generator.region();
            let digest = generator.name("read_digest");
            generator.body.open(&format!("region {fold_region} {{"));
            generator.body.line(&format!(
                "let {digest} = fold_prefix<{fold_region}>(source: &{fold_region} {data}, produced: {window}_u64, seed: {produced});"
            ));
            generator
                .body
                .line(&format!("set total = total +wrap {digest};"));
            generator.body.close();
            if matches!(variant, FileLoop::BreakAfterSubmission) {
                // An edge that leaves the loop from the remainder rather than
                // from the prologue.
                let stop = generator.name("enough");
                generator
                    .body
                    .line(&format!("let {stop} = ige(total, 1000000000_u64);"));
                generator.body.open(&format!("if {stop} {{"));
                generator.body.line(&format!("break {loop_label};"));
                generator.body.close();
            }
            generator.body.close();
            generator.body.open("ReadEnd() => {");
            generator.body.line("set total = total +wrap 13_u64;");
            generator.body.close();
            let problem = generator.name("read_problem");
            generator
                .body
                .open(&format!("ReadFailed(error: {problem}) => {{"));
            generator.body.line("set total = total +wrap 17_u64;");
            generator.body.close();
            generator.body.close();
            generator.body.close();
            generator.body.close();
            generator.body.close();
            let denied = generator.name("open_problem");
            generator.body.open(&format!("Err(error: {denied}) => {{"));
            generator.body.line("set total = total +wrap 19_u64;");
            generator.body.close();
            generator.body.close();
            generator.body.close();
            generator.body.close();
        });
        self.body.close();
    }

    /// Opens one file, reads it, and publishes what it read. The write reads
    /// the buffer the read wrote, so no permission may overlap the two, and the
    /// published bytes are the file's.
    fn read_then_write_block(&mut self) {
        self.used_files = true;
        let window = *self.rng.pick(&[32_u64, 64, 128, 256]);
        let name = self.declare_name(None);
        let data = self.declare_buffer(window, 46);
        let got = self.name("got");
        self.body.line(&format!("let {got} = 0_u64;"));
        self.scalars.push(got.clone());
        let factory = self.region();
        let name_region = self.region();
        let permit = self.name("permit");
        self.spend();
        self.body.open(&format!("region {factory} {{"));
        self.body.line(&format!(
            "let {permit} = reserve_file<{factory}>(factory: &uniq {factory} files);"
        ));
        self.body.open(&format!("region {name_region} {{"));
        self.body.open(&format!(
            "match open_file<{factory}, {name_region}>(permit: move {permit}, root: &{factory} cwd, name: &{name_region} {name}, start: 0_u64, end: 7_u64) {{"
        ));
        let handle = self.name("handle");
        self.body.open(&format!("Ok(value: {handle}) => {{"));
        let file_region = self.region();
        let data_region = self.region();
        self.body.open(&format!("region {file_region} {{"));
        self.body.open(&format!("region {data_region} {{"));
        self.body.open(&format!(
            "match read_at<{file_region}, {data_region}>(file: &{file_region} {handle}, destination: &uniq {data_region} {data}, file_offset: 0_u64, start: 0_u64, end: {window}_u64) {{"
        ));
        let produced = self.name("produced");
        self.body
            .open(&format!("ReadBytes(next: {produced}) => {{"));
        self.body.line(&format!("set {got} = {produced};"));
        self.body.close();
        self.body.open("ReadEnd() => {");
        self.body.close();
        let problem = self.name("read_problem");
        self.body
            .open(&format!("ReadFailed(error: {problem}) => {{"));
        self.body.close();
        self.body.close();
        self.body.close();
        self.body.close();
        self.body.close();
        let denied = self.name("open_problem");
        self.body.open(&format!("Err(error: {denied}) => {{"));
        self.body.close();
        self.body.close();
        self.body.close();
        self.body.close();

        // [SYS-8] wants `end` proved inside the buffer, and the read's reported
        // endpoint is a boundary result the checker publishes no range for, so
        // the relation is tested with a real branch before the publication --
        // the P12 form, not a claim.
        let room = self.name("room");
        let publishable = self.name("publishable");
        let publish = self.region();
        let ok = self.name("relayed");
        let failed = self.name("relay_failed");
        self.body.line(&format!("let {room} = len({data});"));
        self.body
            .line(&format!("let {publishable} = ile({got}, {room});"));
        self.body.open(&format!("if {publishable} {{"));
        self.body.open(&format!("region {publish} {{"));
        self.body.open(&format!(
            "match write_once<{publish}, {publish}>(output: &uniq {publish} out, source: &{publish} {data}, start: 0_u64, end: {got}) {{"
        ));
        self.body.open(&format!("Ok(value: {ok}) => {{"));
        self.body.line(&format!("set total = total +wrap {ok};"));
        self.body.close();
        self.body.open(&format!("Err(error: {failed}) => {{"));
        self.body.line("set total = total +wrap 23_u64;");
        self.body.close();
        self.body.close();
        self.body.close();
        self.body.close();
    }

    /// Directory enumeration. Only the reported counts reach the digest,
    /// because the byte order of one batch is the host's and folding it would
    /// make the program its own unstable oracle.
    fn directory_block(&mut self) {
        self.used_files = true;
        let entries = self.declare_buffer(4096, 0);
        let ended = self.name("ended");
        self.body.line(&format!("let {ended} = 0_u8;"));
        let factory = self.region();
        let permit = self.name("permit");
        self.spend();
        self.body.open(&format!("region {factory} {{"));
        self.body.line(&format!(
            "let {permit} = reserve_file<{factory}>(factory: &uniq {factory} files);"
        ));
        self.body.open(&format!(
            "match open_directory_source<{factory}>(permit: move {permit}, directory: &{factory} cwd) {{"
        ));
        let source = self.name("listing");
        self.body.open(&format!("Ok(value: {source}) => {{"));
        let rounds = self.name("rounds");
        let label = self.label();
        let stop = self.name("stop");
        let live = self.name("live");
        self.body.line(&format!("let {rounds} = 0_u64;"));
        self.body.open(&format!("loop {label} {{"));
        self.body
            .line(&format!("let {stop} = ige({rounds}, 8_u64);"));
        self.body.open(&format!("if {stop} {{"));
        self.body.line(&format!("break {label};"));
        self.body.close();
        let batch = self.region();
        self.body.open(&format!("region {batch} {{"));
        self.body.open(&format!(
            "match directory_next<{batch}, {batch}>(source: &uniq {batch} {source}, destination: &uniq {batch} {entries}, start: 0_u64, end: 4096_u64) {{"
        ));
        let endpoint = self.name("endpoint");
        let reported = self.name("reported");
        self.body.open(&format!(
            "ListBytes(next: {endpoint}, entries: {reported}) => {{"
        ));
        self.body
            .line(&format!("set total = total +wrap {reported};"));
        self.body.close();
        self.body.open("ListEnd() => {");
        self.body.line(&format!("set {ended} = 1_u8;"));
        self.body.close();
        let problem = self.name("list_problem");
        self.body
            .open(&format!("ListFailed(error: {problem}) => {{"));
        self.body.line(&format!("set {ended} = 2_u8;"));
        self.body.close();
        self.body.close();
        self.body.close();
        self.body.line(&format!("let {live} = ieq({ended}, 0_u8);"));
        self.body.open(&format!("if {live} {{"));
        self.body.indent -= 1;
        self.body.line("} else {");
        self.body.indent += 1;
        self.body.line(&format!("break {label};"));
        self.body.close();
        self.body
            .line(&format!("set {rounds} = {rounds} +wrap 1_u64;"));
        self.body.close();
        self.body.close();
        let denied = self.name("source_problem");
        self.body.open(&format!("Err(error: {denied}) => {{"));
        self.body.line("set total = total +wrap 29_u64;");
        self.body.close();
        self.body.close();
        self.body.close();
    }

    /// One always-true claim, in the residual shape [CLM-2] admits: the checker
    /// proves the remainder's domain but publishes no range for its result, so
    /// the following subscript has no other route to its bound. The predicate
    /// is true in every execution, so the trap path is never taken and the
    /// program's observables are the ordinary ones.
    fn claim_block(&mut self) {
        let length = *self.rng.pick(&[8_u64, 16, 32, 64]);
        let store = self.declare_buffer(length, 65);
        let seed = self.name("seed");
        let index = self.name("offset");
        let guard = self.name("guard");
        let literal = self.rng.between(1, 4096);
        self.traps = true;
        self.spend();
        self.body.line(&format!("let {seed} = {literal}_u64;"));
        self.body
            .line(&format!("let {index} = {seed} % {length}_u64;"));
        self.body.line(&format!(
            "claim {guard}: ilt({index}, {length}_u64) because \"premises: {index} is {seed} remainder {length}_u64 computed in this function and {store} has length {length}\\nderivation: an unsigned remainder by {length}_u64 is at most {} and is therefore strictly less than the buffer length\\nconclusion: ilt({index}, {length}_u64) is true\\nchecker gap: ENT proves the remainder operation domain but publishes no range for its result\\nconsumers: the following subscript of {store} needs this upper Range component\";",
            length - 1
        ));
        let byte = self.byte_literal();
        self.body.line(&format!("set {store}[{index}] = {byte};"));
        self.body.line(&format!("set total = total +wrap {index};"));
        self.scalars.push(index);
    }

    /// A direct view over a live buffer, moved into a helper that reads through
    /// it (P10). The slice descriptor carries a finite static origin set, so the
    /// footprint the permission judgment forms for the call is the origin's, not
    /// the descriptor's -- which is exactly the thing worth putting under an
    /// overlap oracle.
    fn slice_view_block(&mut self) {
        self.need_view = true;
        let (store, length) = match self.buffers.last() {
            Some(buffer) => (buffer.name.clone(), buffer.length),
            None => {
                let length = *self.rng.pick(&[8_u64, 16, 32, 64]);
                let fill = self.rng.between(48, 90);
                (self.declare_buffer(length, fill), length)
            }
        };
        let region = self.region();
        let view = self.name("view");
        let room = self.name("view_room");
        let digest = self.name("view_digest");
        let seed = self.rng.between(1, 97);
        self.spend();
        self.body.open(&format!("region {region} {{"));
        self.body
            .line(&format!("let {view} = slice_of(&{region} {store});"));
        self.body.line(&format!("let {room} = len({view});"));
        self.body.line(&format!(
            "let {digest} = fold_view<{region}>(view: move {view}, produced: {length}_u64, seed: {seed}_u64);"
        ));
        self.body
            .line(&format!("set total = total +wrap {digest};"));
        self.body.line(&format!("set total = total +wrap {room};"));
        self.body.close();
    }

    /// A conditional value: the `let`-initializer `match` of [GRAM-7], whose
    /// arms deliver through [GIVE-1].
    fn give_match_block(&mut self) {
        let source = self.scalar_atom();
        let narrow = self.name("narrowed");
        let picked = self.name("picked");
        let exact = self.name("exact");
        let widened = self.name("widened");
        let failed = self.name("narrow_failed");
        let fallback = self.rng.between(100, 4000);
        self.spend();
        self.body
            .line(&format!("let {narrow} = cvt<u64, u8>({source});"));
        self.body.open(&format!("let {picked} = match {narrow} {{"));
        self.body.open(&format!("Ok(value: {exact}) => {{"));
        self.body
            .line(&format!("let {widened} = cvt<u8, u64>({exact});"));
        self.body.line(&format!("give {widened};"));
        self.body.close();
        self.body.open(&format!("Err(error: {failed}) => {{"));
        self.body.line(&format!("give {fallback}_u64;"));
        self.body.close();
        self.body.close();
        self.body
            .line(&format!("set total = total +wrap {picked};"));
        self.scalars.push(picked);
    }

    /// A publication whose failure arm leaves the program through a typed exit
    /// status rather than through the digest, so an execution that diverges on
    /// the failure edge diverges on the exit status too.
    fn typed_exit_block(&mut self) {
        let length = *self.rng.pick(&[4_u64, 8, 16]);
        let fill = self.rng.between(48, 90);
        let store = self.declare_buffer(length, fill);
        let region = self.region();
        let ok = self.name("exit_wrote");
        let failed = self.name("exit_failed");
        let code = self.rng.between(20, 90);
        self.spend();
        self.body.open(&format!("region {region} {{"));
        self.body.open(&format!(
            "match write_once<{region}, {region}>(output: &uniq {region} out, source: &{region} {store}, start: 0_u64, end: {length}_u64) {{"
        ));
        self.body.open(&format!("Ok(value: {ok}) => {{"));
        self.body.line(&format!("set total = total +wrap {ok};"));
        self.body.close();
        self.body.open(&format!("Err(error: {failed}) => {{"));
        self.body
            .line(&format!("return exit_status(code: {code}_u8);"));
        self.body.close();
        self.body.close();
        self.body.close();
    }

    /// The invocation snapshot: a count, and one argument's byte length.
    ///
    /// Never position zero. The oracle runs the same program from three
    /// different files -- one per lowering -- so argument zero is the one
    /// invocation datum the harness cannot make identical across the runs it
    /// compares. A program that reads it publishes the harness's file name and
    /// reports a difference that is not the compiler's. Positions one and two
    /// are the two literal arguments the oracle passes, identical everywhere.
    fn argument_block(&mut self) {
        self.used_args = true;
        let region = self.region();
        let count = self.name("argument_count");
        self.spend();
        self.body.open(&format!("region {region} {{"));
        self.body.line(&format!(
            "let {count} = args_count<{region}>(args: &{region} args);"
        ));
        self.body.line(&format!("set total = total +wrap {count};"));
        if self.rng.chance(60) {
            let value = self.name("argument");
            let failed = self.name("argument_missing");
            let inner = self.region();
            let length = self.name("argument_length");
            let position = self.rng.between(1, 2);
            self.body.open(&format!(
                "match arg_get<{region}>(args: &{region} args, position: {position}_u64) {{"
            ));
            self.body.open(&format!("Ok(value: {value}) => {{"));
            self.body.open(&format!("region {inner} {{"));
            self.body.line(&format!(
                "let {length} = host_bytes_len<{inner}>(value: &{inner} {value});"
            ));
            self.body
                .line(&format!("set total = total +wrap {length};"));
            self.body.close();
            self.body.close();
            self.body.open(&format!("Err(error: {failed}) => {{"));
            self.body.line("set total = total +wrap 31_u64;");
            self.body.close();
            self.body.close();
        }
        self.body.close();
    }
}

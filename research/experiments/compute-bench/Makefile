# Serves compute-bench: the one place every compiler flag is written, and the
# only path from a Whitefoot source to a row in the table.
#
#   make deps            fetch (network) and build the pinned schedulers
#   make build           the compiler, the modules, the objects, one image
#                        per kernel, with the link-time assertions of 6.3
#   make verify          every form at every emitted width, correctness only
#   make compare         the timed passes and the table, into a fresh RESULTS
#   make programs-check  the cheap gate target: compile every program in the
#                        two modes the table uses and assert what --par emits
#
# This is a measurement bundle, not a gate. `programs-check` is the one target
# the repository's `make check` runs. Nothing here fails on an elapsed time,
# a ratio, a spread or an ordering: no band, no threshold, no timeout.
#
# Generated trees, binaries and raw output stay in the scratch root. Nothing
# is written inside the repository.

ROOT   := $(abspath ../../..)
WHITEFOOT_SCRATCH_ROOT ?= $(patsubst %/,%,$(if $(TMPDIR),$(TMPDIR),/tmp))/whitefoot
WORK   := $(WHITEFOOT_SCRATCH_ROOT)/whitefoot-compute-bench
BUILD  := $(WORK)/build
DEPS   := $(WORK)/deps
RESULTS ?= $(WORK)/results/$(shell date -u +%Y%m%dT%H%M%SZ)
# Expanded once, here: a recursive RESULTS would run `date` again at every
# later reference and scatter one run across several directories.
RESULTS := $(RESULTS)
WFC    := $(ROOT)/compiler/target/gate/whitefootc
FLOOR  := $(ROOT)/compiler/src/backend/wf_floor.c
SCHED  := $(ROOT)/compiler/src/backend/sched
CC     := /usr/bin/clang
CXX    := /usr/bin/clang++
ARCH   := $(shell uname -m)
# `shasum -a 256` is present on macOS and on Linux; sha256sum is the fallback.
SHA256 := $(shell if command -v shasum > /dev/null 2>&1; then echo "shasum -a 256"; else echo sha256sum; fi)

# Never -march=native: Clang 18 infers an invalid AVX10 combination on one
# hosted runner and that failure is on the record.
BENCH_ARCH := $(if $(filter x86_64,$(ARCH)),-march=x86-64-v3,)
# Not cosmetic. The investigation traces a large core-time swing to whether one
# 32-byte hot loop crosses a 32-byte boundary. It must reach every translation
# unit of a reference implementation or none of it. It deliberately does not
# reach the WF side, which is built below the way whitefootc builds it; the
# comment on WF_FLAGS states that asymmetry and why it is admitted.
LOOP       := $(if $(filter x86_64,$(ARCH)),-falign-loops=32,)
TARGET_CPU := $(if $(filter x86_64,$(ARCH)),-C target-cpu=x86-64-v3,)

WARN   := -Wall -Wextra -Werror -Wpedantic
SCALAR := -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize \
          -fno-lto
BASE   := -O3 -g $(WARN) -pthread $(LOOP) $(SCALAR) $(BENCH_ARCH)
CFLAGS   := -std=c11 $(BASE)
CXXFLAGS := -std=c++17 -DNDEBUG $(BASE)
RFLAGS := -C no-vectorize-loops -C no-vectorize-slp -C lto=off \
          -C symbol-mangling-version=v0 $(TARGET_CPU)

# Exactly what whitefootc itself passes clang. VERIFIED in
# compiler/src/bin/whitefootc.rs and compiler/src/driver.rs: -std=c11,
# TARGET_COMPILE_ARGUMENTS = -pthread, HOST_OPTIMIZATION_ARGUMENTS = -O2,
# HOST_LINK_LIBRARIES = -lm, plus -Wno-override-module on the IR input.
# The Whitefoot runtime and the emitted module are built the way the compiler
# builds them, not the way the kernels are built, because anything else would
# measure a runtime no Whitefoot program ever gets. THIS IS AN ADMITTED
# ASYMMETRY: the WF module and the runtime are built at -O2 with no -march and
# no loop alignment while every reference is built at -O3 with both. It cuts
# against the WF row, and README.md states it in its own paragraph.
WF_FLAGS := -std=c11 -pthread -O2 -Wno-override-module

# An A/B control, and nothing else. Whatever is set here is appended to the
# `--par` emission of the B TWIN below -- never to the plain image, which is
# emitted with this variable empty whatever it is set to -- so a run can
# compare two compilers' worth of behaviour on one tree, in one set of passes,
# by changing one compiler control: for example
# `make compare WF_PAR_CONTROL_FLAGS='--par-recursive-frontier off'`, which
# withholds the recursion budget the default asks the runtime for. It is
# EMPTY BY DEFAULT and must stay empty for any table that is recorded: the
# `wf` row of a recorded table is the program plain `--par` produces, which is
# this bundle's whole question (README, "What WF means here"). A non-empty
# setting is written into manifest.txt and announced in the table header, and
# it puts a second row, `wf-b`, in every block, so a control-flag table can
# never be read as a plain one; the manifest records the variable either way.
# Nothing here selects a chunk count or a grain: the compiler owns those, and a
# flag that made the table nicer would be measuring a program no Whitefoot user
# gets.
WF_PAR_CONTROL_FLAGS ?=

# The value of that variable is part of what emitted the twin's module, exactly
# as the source and the compiler binary are, so it is kept in a stamp the
# twin's module rule depends on and that is rewritten only when the value
# changes: a control-flag run re-emits the twin instead of silently re-timing
# the one the last run left, and a run that changes nothing re-emits nothing.
# The plain module does not read it and is never re-emitted by it.
PAR_CONTROL := $(BUILD)/par-control-flags.txt
$(PAR_CONTROL): FORCE
	@mkdir -p $(BUILD)
	@printf '%s\n' '$(WF_PAR_CONTROL_FLAGS)' > $@.new
	@cmp -s $@.new $@ || mv -f $@.new $@
	@rm -f $@.new
FORCE:

# The same handle on the other side of the link, and the same discipline. The
# emission control above changes the module the compiler writes; this one is
# appended to the compile of the four Whitefoot RUNTIME translation units of
# the B TWIN below -- never to the plain image's four, which are compiled with
# this variable empty whatever it is set to -- so a run can measure what one
# runtime constant is worth on this block of kernels, against the shipped
# runtime in the same passes, without editing the constant per run: for example
# `make compare WF_RUNTIME_CONTROL_FLAGS=-DWF_PAR_SPLIT_WORK_UNIT=300000`,
# which builds the twin's splitter work unit at a quarter of its default. It is
# EMPTY BY DEFAULT and must stay empty for any table that is recorded: the `wf`
# row of a recorded table is the program plain `--par` produces linked with the
# runtime this tree ships, and a runtime built to make a row nicer would be
# measuring a runtime no Whitefoot program gets. It is written into
# manifest.txt either way and announced in the table header when it is not
# empty, exactly as the emission control is. It reaches no reference, no oracle
# and not the harness: those are not the Whitefoot runtime.
WF_RUNTIME_CONTROL_FLAGS ?=

# Kept in a stamp for the same reason the emission control is: the value is
# part of what compiled an object, so the twin's four runtime objects depend on
# it and a changed value recompiles and relinks the twin instead of silently
# re-timing the one the last run left. A run that changes nothing rebuilds
# nothing, and the plain image's four objects never read it at all.
RUNTIME_CONTROL := $(BUILD)/runtime-control-flags.txt
$(RUNTIME_CONTROL): FORCE
	@mkdir -p $(BUILD)
	@printf '%s\n' '$(WF_RUNTIME_CONTROL_FLAGS)' > $@.new
	@cmp -s $@.new $@ || mv -f $@.new $@
	@rm -f $@.new

# The third handle, and the one that moves how the B TWIN's Whitefoot side is
# COMPILED rather than what it says. Whatever is set here is appended to the
# twin's compile of the emitted `--par` module object AND of its four Whitefoot
# runtime units -- never to the plain image's, which are compiled with
# $(WF_FLAGS) alone whatever this is set to -- so a run can measure what a code
# generation flag is worth on exactly the translation units whitefootc itself
# builds, against the shipped flag set in the same passes: for example
# `make compare WF_MODULE_CONTROL_FLAGS='-falign-functions=64 -falign-loops=32'`,
# which gives the twin's WF side the loop alignment the references already get
# and the WF side does not (see the asymmetry paragraph in README.md). It
# reaches no reference, no oracle, not the harness and not the `--no-overlap`
# control object, because none of those is the Whitefoot side of the link, and
# the seq object is shared with the plain image.
#
# It is EMPTY BY DEFAULT and must stay empty for any table that is recorded,
# exactly as the two above: the `wf` row of a recorded table is the program
# plain `--par` produces built the way whitefootc builds it, and a WF side
# compiled at flags no Whitefoot program gets would be measuring a program no
# Whitefoot user gets. It is written into manifest.txt either way and announced
# in the table header when it is not empty.
WF_MODULE_CONTROL_FLAGS ?=

# Kept in a stamp for the same reason the other two are: the value is part of
# what compiled an object, so the twin's module object and its four runtime
# objects depend on it and a changed value recompiles and relinks the twin
# instead of silently re-timing the one the last run left. The plain image's
# objects never read it at all.
MODULE_CONTROL := $(BUILD)/module-control-flags.txt
$(MODULE_CONTROL): FORCE
	@mkdir -p $(BUILD)
	@printf '%s\n' '$(WF_MODULE_CONTROL_FLAGS)' > $@.new
	@cmp -s $@.new $@ || mv -f $@.new $@
	@rm -f $@.new

# ------------------------------------------------- the twin's own sources --
# The fourth handle, and the only one that changes WHERE the twin's Whitefoot
# side comes from rather than what is appended to its build. The three control
# variables above answer "what would this flag be worth here" from one tree;
# these three answer "what would the tree this branch started from read here",
# which is the question a regression check asks. They name the twin's four
# Whitefoot runtime sources and the compiler that emits its `--par` module:
#
#   WF_B_SCHED_DIR  the directory holding core.c, prim_host.c and entry.c
#   WF_B_FLOOR      the wf_floor.c translation unit
#   WF_B_WFC        the whitefootc binary that emits the twin's module
#
# Each DEFAULTS TO THIS TREE'S OWN, so a bundle that sets none of them is the
# bundle that was here before: the twin is built from the same sources as the
# plain image and only a control flag can separate them. Pointing any of the
# three at another checkout -- a worktree of the merge-base, exported outside
# the repository -- builds the twin from that tree's runtime and that tree's
# emission and times it against this tree's `wf` in the same passes, which is
# what `.github/workflows/compute-regression.yml` runs. The plain image reads
# none of them: `wf` is always this tree's program.
#
# This is ONE BUILD PATH, not a second one. The baseline twin is the twin the
# controls already built, handed different inputs; every rule, link line and
# assertion below is the same rule, link line and assertion.
WF_B_SCHED_DIR ?= $(SCHED)
WF_B_FLOOR     ?= $(FLOOR)
WF_B_WFC       ?= $(WFC)

# What the twin was built from, for manifest.txt: the merge-base revision on a
# regression run, and the words below when the twin's sources are this tree's.
# It is a label and reaches no compile, so changing it re-emits nothing.
WF_B_SOURCE ?=

# Set when any of the three points somewhere other than this tree. It joins
# the three control flags in $(AB) below, so naming a baseline is by itself
# enough to ask for the twin -- a regression run sets no control flag and must
# still get a second image.
BASELINE := $(if $(filter-out $(SCHED),$(WF_B_SCHED_DIR)),1,)$(if $(filter-out $(FLOOR),$(WF_B_FLOOR)),1,)$(if $(filter-out $(WFC),$(WF_B_WFC)),1,)

# The stamp that makes a changed baseline rebuild the twin, exactly as the
# three control stamps make a changed flag rebuild it. A path alone would not
# do it: two runs can name the same directory with different bytes in it -- a
# second regression run against a moved merge-base is precisely that -- and
# make would compare timestamps against sources it had never seen. So the
# stamp carries the CONTENT HASH of the twin's inputs, and the twin's module
# and its four runtime objects depend on it: different baseline bytes,
# different stamp, re-emitted module and recompiled runtime. Identical bytes
# under another path rebuild nothing, which is right -- that is the same build.
#
# It hashes every `.c` and `.h` of the scheduler directory rather than the
# three units the link takes, because those three include the directory's
# headers and a header is exactly what a rule listing source files alone would
# miss. The probe and smoke units are hashed too and compile into nothing; the
# cost of that over-approximation is a twin rebuilt when it need not have been,
# which is the side of this trade to be wrong on.
BASELINE_STAMP := $(BUILD)/baseline-source.txt
$(BASELINE_STAMP): FORCE
	@mkdir -p $(BUILD)
	@test -f $(WF_B_SCHED_DIR)/core.c && test -f $(WF_B_SCHED_DIR)/prim_host.c \
	  && test -f $(WF_B_SCHED_DIR)/entry.c && test -f $(WF_B_FLOOR) \
	  && test -x $(WF_B_WFC) || { \
	    echo 'compute-bench: the twin sources named by WF_B_SCHED_DIR, WF_B_FLOOR and' >&2; \
	    echo 'WF_B_WFC must all exist; the defaults are this tree'"'"'s own.' >&2; exit 1; }
	@$(SHA256) $(WF_B_SCHED_DIR)/*.c $(WF_B_SCHED_DIR)/*.h $(WF_B_FLOOR) $(WF_B_WFC) \
	  | awk '{ print $$1 }' > $@.new
	@cmp -s $@.new $@ || mv -f $@.new $@
	@rm -f $@.new

# ---------------------------------------------------------------- the twin --
# The A/B instrument. Either control above answers "what would this be worth
# here", and on a host whose run-to-run spread is wider than the effect being
# looked for -- this bundle's development host reads the quadrature W=4 ratio
# anywhere from 0.916 to 1.035 over six runs of IDENTICAL BYTES -- comparing
# two separate runs cannot answer it. The reducer already solves that for the
# references: within one pass every form runs as its own process in an
# alternating order, and the `wf` row's ratio is the median of WITHIN-PASS
# paired ratios rather than a quotient of two medians. What was missing was a
# second Whitefoot image in those same passes.
#
# So when any of the three controls is set -- or when WF_AB=1 asks for the twin
# with no
# control at all, which is the instrument's own null check -- every kernel gets
# a SECOND image, `$(BUILD)/<kernel>-b`, built from the same sources with the
# controls applied, timed as the form `wf-b` in the same alternating per-pass
# order as every other form, and reported by reduce.awk with its own
# ratio/cpu_r/lower plus one `A/B wf-b/wf` line per block. The plain image is
# always built with all three controls empty, and every `-b` module and object
# carries its own name (`<kernel>-par-b.ll`, `sched_core-b.o`, ...), so nothing
# plain is ever rebuilt with a control flag and the two images differ in
# exactly what the controls say.
#
# A table with a `wf-b` row is an A/B instrument and is NEVER recorded as a
# plain table; the record-only workflow sets none of these variables and
# therefore never builds a twin. Leaving them all empty leaves the bundle as it
# was: one image per kernel, one `wf` row, no extra build and no extra process.
#
# $(BASELINE) is in the list because naming another tree's runtime or compiler
# through WF_B_SCHED_DIR, WF_B_FLOOR or WF_B_WFC above is by itself a request
# for the twin: a regression run sets no control flag and must still get its
# second image.
WF_AB ?=
AB := $(if $(strip $(WF_PAR_CONTROL_FLAGS)$(WF_RUNTIME_CONTROL_FLAGS)$(WF_MODULE_CONTROL_FLAGS)$(WF_AB)$(BASELINE)),1,)

KERNELS ?= mandelbrot quadrature records fir
PASSES ?= 5
CALLS  ?= 5

CHECK   := $(WORK)/check
CHECKED := $(addprefix $(CHECK)/checked_,$(KERNELS))
JOBS    ?= $(shell nproc 2>/dev/null || sysctl -n hw.ncpu)

# The ParlayLib probe in `deps` decides whether this object exists at all.
# It is read here, in a sub-make that `build` starts after `deps` has run.
PARLAY_OBJ := $(if $(wildcard $(DEPS)/parlay-unavailable.txt),,$(BUILD)/backend_parlay.o)

# The harness, the references and the oracles are one build, shared by the
# plain image and the twin: no control reaches them, so a second copy
# would be the same bytes under another name and would move code placement,
# which is this bundle's largest confound. Only the Whitefoot side of the link
# is doubled -- the emitted module object and the four runtime units -- because
# only those read WF_RUNTIME_CONTROL_FLAGS or WF_MODULE_CONTROL_FLAGS.
SHARED_OBJS := $(BUILD)/harness.o $(BUILD)/backend_serial.o $(BUILD)/backend_static.o \
               $(BUILD)/backend_tbb.o $(PARLAY_OBJ) $(BUILD)/backend_rayon.o
RUNTIME_OBJS   := $(BUILD)/floor.o $(BUILD)/sched_core.o $(BUILD)/sched_prim_host.o \
                  $(BUILD)/sched_entry.o
RUNTIME_OBJS_B := $(BUILD)/floor-b.o $(BUILD)/sched_core-b.o $(BUILD)/sched_prim_host-b.o \
                  $(BUILD)/sched_entry-b.o
COMMON_OBJS := $(SHARED_OBJS) $(RUNTIME_OBJS)

LINK_LIBS := -L$(DEPS)/lib -Wl,-rpath,$(DEPS)/lib -ltbb \
             $(DEPS)/rayon/libwf_compute_bench_rayon.a -ldl -lm

.PHONY: all compiler deps build images verify compare programs-check clean clean-deps FORCE
all: build

# ---------------------------------------------------------------- compiler --
# Skipped down to a presence test when WFC was overridden on the command line,
# so the root gate does not rebuild a compiler it already built.
compiler:
	@if [ "$(origin WFC)" = "command line" ]; then \
	    test -x "$(WFC)"; \
	else \
	    cargo build --manifest-path $(ROOT)/compiler/Cargo.toml --profile gate \
	        --bin whitefootc --locked --offline; \
	fi

# -------------------------------------------------------------------- deps --
# `fetch` is the only network step; everything after it is offline. A
# sandboxed first run dies there, and the message below says so rather than
# leaving a bare git error.
deps:
	@mkdir -p $(BUILD) $(DEPS)
	@OUT=$(DEPS) BENCH_ARCH="$(BENCH_ARCH)" DEPS_SCALAR_FLAGS="$(SCALAR)" \
	 DEPS_RUSTFLAGS="$(RFLAGS)" CC="$(CC)" CXX="$(CXX)" sh deps.sh fetch || { \
	    echo 'compute-bench: `deps.sh fetch` is the only network step in this bundle.' >&2; \
	    echo 'It clones oneTBB and ParlayLib at their pins and populates the cargo' >&2; \
	    echo 'cache. A sandboxed host must run it once with the network available.' >&2; \
	    exit 1; }
	OUT=$(DEPS) BENCH_ARCH="$(BENCH_ARCH)" DEPS_SCALAR_FLAGS="$(SCALAR)" \
	 DEPS_RUSTFLAGS="$(RFLAGS)" CC="$(CC)" CXX="$(CXX)" sh deps.sh build
	@if [ -e $(DEPS)/parlay-unavailable.txt ]; then \
	    reason=$$(head -1 $(DEPS)/parlay-unavailable.txt | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g'); \
	    { echo '/* Generated by `make deps` from the ParlayLib probe. */'; \
	      echo '#define WFB_NO_PARLAY 1'; \
	      printf '#define WFB_PARLAY_REASON "%s"\n' "$$reason"; } > $(BUILD)/parlay_status.h; \
	else \
	    echo '/* Generated by `make deps`: the ParlayLib probe compiled. */' > $(BUILD)/parlay_status.h; \
	fi

# ------------------------------------------------------- the two WF modules --
# One source, two compilations, because --par and --no-overlap are opposite
# lowerings and the compiler rejects them together. The @main/@wf__main_body
# rename is necessary because both modules export them and harness.c owns
# both; every user function the compiler emits has internal linkage, which is
# also why the host adapter has to be LLVM IR rather than a C prototype. The
# two greps are structural assertions about the module, not stored counts: the
# --par module really offers work and the control really does not.
#
# These are per-kernel rules rather than pattern rules on purpose: a target
# reached only through a pattern rule is an intermediate, and make 4.3 will
# not recreate a missing intermediate while the image that consumed it is
# still up to date -- which would leave `compare` hashing a module that is no
# longer on disk.
#
# The compiler binary is a normal prerequisite as well: a module emitted by an
# earlier build of the compiler is not this tree's module, and with the
# order-only `| compiler` alone make kept one revision's modules across three
# compiler rebuilds (found landed: the first merged-tree table in
# research/investigations/compute-runtime/RESULTS.md timed quadrature on a
# module emitted at 33ed2c00; the fresh emission is the table that replaced it).
define kernel-modules
$$(BUILD)/$(1)-par.ll: programs/$(1).wf $(1)_host.ll $$(WFC) | compiler
	@mkdir -p $$(BUILD)
	$$(WFC) --par --par-ledger --emit-llvm programs/$(1).wf -o $$@.raw > $$(BUILD)/$(1)-par.ledger
	sed -e 's/@main(/@wfb_$(1)_par_main(/g' \
	    -e 's/@wf__main_body(/@wfb_$(1)_par_body(/g' $$@.raw > $$@
	sed -e 's/@wf_bench_$(1)(/@wf_bench_$(1)_par(/' \
	    -e 's/@wf_bench_$(1)_release(/@wf_bench_$(1)_par_release(/' $(1)_host.ll >> $$@
	grep -q 'call void @wf__par_publish(' $$@

# The twin's module: the same recipe with WF_PAR_CONTROL_FLAGS appended and its
# own name, so the plain module above can never be emitted under a control
# flag. The symbol renames are identical because the two modules never meet in
# one image.
#
# It is emitted by $(WF_B_WFC), which is this tree's compiler unless a baseline
# named another one, so a regression run compares this tree's emission against
# the merge-base's rather than running both arms through one emitter. The
# source is this tree's either way: a language change that moved a kernel's
# .wf source has to be measured on the source the branch ships, and the
# baseline compiler either accepts it or the run says so. $(BASELINE_STAMP)
# carries the content hash of that compiler, so a moved baseline re-emits.
$$(BUILD)/$(1)-par-b.ll: programs/$(1).wf $(1)_host.ll $$(WF_B_WFC) $$(PAR_CONTROL) \
                         $$(BASELINE_STAMP) | compiler
	@mkdir -p $$(BUILD)
	$$(WF_B_WFC) --par --par-ledger $$(WF_PAR_CONTROL_FLAGS) --emit-llvm programs/$(1).wf -o $$@.raw > $$(BUILD)/$(1)-par-b.ledger
	sed -e 's/@main(/@wfb_$(1)_par_main(/g' \
	    -e 's/@wf__main_body(/@wfb_$(1)_par_body(/g' $$@.raw > $$@
	sed -e 's/@wf_bench_$(1)(/@wf_bench_$(1)_par(/' \
	    -e 's/@wf_bench_$(1)_release(/@wf_bench_$(1)_par_release(/' $(1)_host.ll >> $$@
	grep -q 'call void @wf__par_publish(' $$@

$$(BUILD)/$(1)-seq.ll: programs/$(1).wf $(1)_host.ll $$(WFC) | compiler
	@mkdir -p $$(BUILD)
	$$(WFC) --no-overlap --emit-llvm programs/$(1).wf -o $$@.raw
	sed -e 's/@main(/@wfb_$(1)_seq_main(/g' \
	    -e 's/@wf__main_body(/@wfb_$(1)_seq_body(/g' $$@.raw > $$@
	sed -e 's/@wf_bench_$(1)(/@wf_bench_$(1)_seq(/' \
	    -e 's/@wf_bench_$(1)_release(/@wf_bench_$(1)_seq_release(/' $(1)_host.ll >> $$@
	! grep -q 'call void @wf__par_publish(' $$@

# The constant weight operand of the emitted wf__par_split_budget call. The
# kernel reports it so the table's `note` column can print the chunk count the
# compiler chose. It is a report, never an input: no flag, environment
# variable or source edit selects a chunk count anywhere in this bundle.
$$(BUILD)/$(1)_split.h: $$(BUILD)/$(1)-par.ll
	@w=$$$$(grep -o 'call i64 @wf__par_split_budget(i64 [^,]*, i64 [0-9]*)' $$< \
	        | sed -e 's/.*, i64 //' -e 's/)$$$$//' | head -1); \
	 test -n "$$$$w" || w=0; \
	 { echo '/* Generated by the Makefile from the emitted --par module. */'; \
	   printf '#define WFB_SPLIT_WEIGHT ((size_t)%s)\n' "$$$$w"; } > $$@

# The twin's weight, read out of the twin's own module into a header of the
# same name under $$(BUILD)/b, which the twin's kernel object reaches through
# an earlier -I. A `--par` control that moved the emitted weight would
# otherwise have the twin's `note` column quoting the plain image's.
$$(BUILD)/b/$(1)_split.h: $$(BUILD)/$(1)-par-b.ll
	@mkdir -p $$(BUILD)/b
	@w=$$$$(grep -o 'call i64 @wf__par_split_budget(i64 [^,]*, i64 [0-9]*)' $$< \
	        | sed -e 's/.*, i64 //' -e 's/)$$$$//' | head -1); \
	 test -n "$$$$w" || w=0; \
	 { echo '/* Generated by the Makefile from the twin --par module. */'; \
	   printf '#define WFB_SPLIT_WEIGHT ((size_t)%s)\n' "$$$$w"; } > $$@
endef
$(foreach k,$(KERNELS),$(eval $(call kernel-modules,$(k))))

# --------------------------------------------------------------- objects ----
$(BUILD)/%-par.o: $(BUILD)/%-par.ll
	$(CC) $(WF_FLAGS) -c $< -o $@
$(BUILD)/%-par-b.o: $(BUILD)/%-par-b.ll $(MODULE_CONTROL)
	$(CC) $(WF_FLAGS) $(WF_MODULE_CONTROL_FLAGS) -c $< -o $@
$(BUILD)/%-seq.o: $(BUILD)/%-seq.ll
	$(CC) $(WF_FLAGS) -c $< -o $@

# The plain image's four Whitefoot runtime units. NO CONTROL FLAG REACHES
# THESE, whatever WF_RUNTIME_CONTROL_FLAGS is set to: the `wf` row is the
# runtime this tree ships, and an A/B run must not quietly move both arms.
$(BUILD)/floor.o: $(FLOOR)
	@mkdir -p $(BUILD)
	$(CC) $(WF_FLAGS) -c $< -o $@
$(BUILD)/sched_core.o: $(SCHED)/core.c
	$(CC) $(WF_FLAGS) -c $< -o $@
$(BUILD)/sched_prim_host.o: $(SCHED)/prim_host.c
	$(CC) $(WF_FLAGS) -c $< -o $@
$(BUILD)/sched_entry.o: $(SCHED)/entry.c
	$(CC) $(WF_FLAGS) -c $< -o $@

# The twin's four. The same four translation units, read from $(WF_B_SCHED_DIR)
# and $(WF_B_FLOOR) -- this tree's own unless a baseline named another tree --
# with the runtime control appended and the two stamps that recompile them when
# a control value or the baseline's bytes change.
$(BUILD)/floor-b.o: $(WF_B_FLOOR) $(RUNTIME_CONTROL) $(MODULE_CONTROL) $(BASELINE_STAMP)
	@mkdir -p $(BUILD)
	$(CC) $(WF_FLAGS) $(WF_MODULE_CONTROL_FLAGS) $(WF_RUNTIME_CONTROL_FLAGS) -c $< -o $@
$(BUILD)/sched_core-b.o: $(WF_B_SCHED_DIR)/core.c $(RUNTIME_CONTROL) $(MODULE_CONTROL) $(BASELINE_STAMP)
	$(CC) $(WF_FLAGS) $(WF_MODULE_CONTROL_FLAGS) $(WF_RUNTIME_CONTROL_FLAGS) -c $< -o $@
$(BUILD)/sched_prim_host-b.o: $(WF_B_SCHED_DIR)/prim_host.c $(RUNTIME_CONTROL) $(MODULE_CONTROL) $(BASELINE_STAMP)
	$(CC) $(WF_FLAGS) $(WF_MODULE_CONTROL_FLAGS) $(WF_RUNTIME_CONTROL_FLAGS) -c $< -o $@
$(BUILD)/sched_entry-b.o: $(WF_B_SCHED_DIR)/entry.c $(RUNTIME_CONTROL) $(MODULE_CONTROL) $(BASELINE_STAMP)
	$(CC) $(WF_FLAGS) $(WF_MODULE_CONTROL_FLAGS) $(WF_RUNTIME_CONTROL_FLAGS) -c $< -o $@

$(BUILD)/%_bench.o: %_bench.c harness.h backend.h $(BUILD)/%_split.h
	$(CC) $(CFLAGS) -I$(BUILD) -c $< -o $@
$(BUILD)/%_bench-b.o: %_bench.c harness.h backend.h $(BUILD)/b/%_split.h
	$(CC) $(CFLAGS) -I$(BUILD)/b -I$(BUILD) -c $< -o $@
$(BUILD)/harness.o: harness.c harness.h backend.h
	$(CC) $(CFLAGS) -I$(BUILD) -c $< -o $@
$(BUILD)/backend_serial.o: backend_serial.c backend.h harness.h
	$(CC) $(CFLAGS) -c $< -o $@
$(BUILD)/backend_static.o: backend_static.c backend.h harness.h
	$(CC) $(CFLAGS) -c $< -o $@
$(BUILD)/backend_rayon.o: backend_rayon.c backend.h harness.h
	$(CC) $(CFLAGS) -I$(DEPS)/rayon -c $< -o $@
$(BUILD)/backend_tbb.o: backend_tbb.cpp backend.h harness.h
	$(CXX) $(CXXFLAGS) -isystem $(DEPS)/include -c $< -o $@
$(BUILD)/backend_parlay.o: backend_parlay.cpp backend.h harness.h
	$(CXX) $(CXXFLAGS) -isystem $(DEPS)/include -c $< -o $@

# ------------------------------------- the assertions that make a WF row WF --
# The emitted module defines weak no-op stubs for every wf__par_* symbol, so a
# link that loses sched/entry.c still links, still runs, still produces correct
# output and is SILENTLY SEQUENTIAL. Two assertions close that.
#   1. Link-time: harness.c references wf__par_grants(), which has no weak
#      stub in the emitted module, so dropping the scheduler sources fails the
#      link outright.
#      The recursion budget is asserted the same way and for the same reason:
#      a link that reached its weak stub would answer zero at every component
#      entry and run every recursive kernel sequentially from its first node.
#   2. Post-link, in two parts, because the ELF spelling is wrong on Darwin in
#      both halves: Mach-O nm prints the label with a leading underscore, so an
#      anchored wf__par_publish$ never matches, and Mach-O renders a weak
#      definition in the same one-letter class as a strong one, so the T-versus-W
#      test cannot discriminate there at all.
# Every assertion below has to be able to fail the recipe on its own. The
# first two are separate recipe lines, and make stops at the first nonzero
# status. The two greps inside a case branch are one recipe line, so they are
# joined with `&&`: sequenced with `;` the branch would exit with the status
# of the second grep alone, and a strong-binding assertion that failed would
# be masked by the one after it.
define assert-real-runtime
	nm $(1) | grep -q ' [TS] _\{0,1\}wf__par_publish$$'
	nm $(1) | grep -q ' [TS] _\{0,1\}wf__par_split_budget$$'
	nm $(1) | grep -q ' [TS] _\{0,1\}wf__par_recursion_budget$$'
	case "$$(uname -s)" in \
	  Darwin) nm -m $(1) | grep -q '(__TEXT,__text) external _wf__par_publish$$' && \
	          nm -m $(1) | grep -q '(__TEXT,__text) external _wf__par_split_budget$$' && \
	          nm -m $(1) | grep -q '(__TEXT,__text) external _wf__par_recursion_budget$$' ;; \
	  *)      nm    $(1) | grep -q ' T wf__par_publish$$' && \
	          nm    $(1) | grep -q ' T wf__par_split_budget$$' && \
	          nm    $(1) | grep -q ' T wf__par_recursion_budget$$' ;; \
	esac
endef

# Mach-O plain nm prints T for a weak function definition too, so ask for its
# full binding description there; ELF nm distinguishes a strong T directly.
define require-strong
	case "$$(uname -s)" in \
	  Darwin) nm -m $(1) | awk -v name='_$(2)' \
	            '$$NF==name && /\(__TEXT,__text\) external/ && !/weak/ {ok=1} END {exit !ok}' ;; \
	  *)      nm $(1) | grep -q ' T $(2)$$' ;; \
	esac
endef

# -------------------------------------------------------------- the images --
# One image per kernel. Object and link order is fixed here because the
# bundle's single largest confound was code placement: six byte-identical
# kernel bodies at 0/16/48 mod 64 across six images split a width-one median
# 7.3 ms against 10.7 ms with no scheduler involved. Any change to this line
# can move the ranking.
define kernel-image
$(BUILD)/$(1): $(BUILD)/$(1)_bench.o $(BUILD)/$(1)-par.o $(BUILD)/$(1)-seq.o \
                $(BUILD)/$(1)-par.ll $(BUILD)/$(1)-seq.ll $(BUILD)/$(1)_split.h \
                $$(COMMON_OBJS)
	$$(CXX) -pthread -fno-lto \
	    $(BUILD)/$(1)_bench.o $$(BUILD)/harness.o $$(BUILD)/backend_serial.o \
	    $$(BUILD)/backend_static.o $$(BUILD)/backend_tbb.o $$(PARLAY_OBJ) \
	    $$(BUILD)/backend_rayon.o $(BUILD)/$(1)-par.o $(BUILD)/$(1)-seq.o \
	    $$(BUILD)/floor.o $$(BUILD)/sched_core.o $$(BUILD)/sched_prim_host.o \
	    $$(BUILD)/sched_entry.o $$(LINK_LIBS) -o $$@
	@$$(call assert-real-runtime,$$@)
	@$$(call require-strong,$$@,wf__floor_run)
	@$$(call require-strong,$$@,wf__par_acquire_lane)
endef
$(foreach k,$(KERNELS),$(eval $(call kernel-image,$(k))))

# The twin image, link line for link line the same as the plain one above --
# the same object order, the same libraries, the same three assertions -- with
# the four `-b` runtime objects, the `-b` module object and the `-b` kernel
# object in place of the plain ones. It shares the harness, the references and
# the sequential control object with the plain image. The assertions apply to
# it because a twin that silently lost the scheduler would read as a runtime
# constant that was worth a great deal.
define kernel-image-b
$(BUILD)/$(1)-b: $(BUILD)/$(1)_bench-b.o $(BUILD)/$(1)-par-b.o $(BUILD)/$(1)-seq.o \
                $(BUILD)/$(1)-par-b.ll $(BUILD)/$(1)-seq.ll $(BUILD)/b/$(1)_split.h \
                $$(SHARED_OBJS) $$(RUNTIME_OBJS_B)
	$$(CXX) -pthread -fno-lto \
	    $(BUILD)/$(1)_bench-b.o $$(BUILD)/harness.o $$(BUILD)/backend_serial.o \
	    $$(BUILD)/backend_static.o $$(BUILD)/backend_tbb.o $$(PARLAY_OBJ) \
	    $$(BUILD)/backend_rayon.o $(BUILD)/$(1)-par-b.o $(BUILD)/$(1)-seq.o \
	    $$(BUILD)/floor-b.o $$(BUILD)/sched_core-b.o $$(BUILD)/sched_prim_host-b.o \
	    $$(BUILD)/sched_entry-b.o $$(LINK_LIBS) -o $$@
	@$$(call assert-real-runtime,$$@)
	@$$(call require-strong,$$@,wf__floor_run)
	@$$(call require-strong,$$@,wf__par_acquire_lane)
endef
$(foreach k,$(KERNELS),$(eval $(call kernel-image-b,$(k))))

# `build` starts a sub-make so that PARLAY_OBJ above is read after `deps` has
# written the probe's result, not before it.
build: compiler deps
	$(MAKE) images
# The twin images are in this list only when a control is set or WF_AB=1 asks
# for them; with all three controls empty this is the plain list it always was.
# `compare` consults the same $(AB) below rather than looking for the file, so
# a stale `-b` image left by an earlier A/B run is neither rebuilt nor timed.
images: $(addprefix $(BUILD)/,$(KERNELS)) \
        $(if $(AB),$(addsuffix -b,$(addprefix $(BUILD)/,$(KERNELS))))

# ------------------------------------------------------------------ verify --
# Every form the image reports, at every width the harness's width rule emits
# on this host, oversubscribed ones included. No expected count appears here:
# the driver prints what its oracle computed and this only requires that it
# compared something.
#
# When a twin was built, its `wf` form is verified at every width too, in a
# second sweep after the plain one. The twin links a differently built runtime
# and answers the same oracle or it is not a control; its split fixture line
# also prints the chunk count that runtime chose, which is how an A/B run reads
# the twin's grain before it times anything.
verify: build
	@mkdir -p $(WORK)
	@: > $(WORK)/verify.log
	@set -e; for k in $(KERNELS); do \
	  $(BUILD)/$$k list > $(WORK)/$$k.forms; \
	  widths=`sed -n 's/^# cpus=[0-9]* widths=//p' $(WORK)/$$k.forms`; \
	  forms=`awk -F'\t' '!/^#/ && $$3 != "n/a" { print $$1 }' $(WORK)/$$k.forms`; \
	  for w in $$widths; do \
	    for form in $$forms; do \
	      if [ $$w -eq 1 ]; then \
	        case $$form in wf|wf-seq|serial) ;; *) continue;; esac; \
	      else \
	        case $$form in wf-seq|serial) continue;; esac; \
	      fi; \
	      echo "== $$k $$form width=$$w"; \
	      WF_WORKERS=$$w $(BUILD)/$$k verify $$form $$w > $(WORK)/verify.out 2>&1 \
	        || { cat $(WORK)/verify.out; exit 1; }; \
	      cat $(WORK)/verify.out >> $(WORK)/verify.log; \
	      awk '/VERIFY PASS/ { if (match($$0, /compared=[0-9]+/)) { \
	             if (substr($$0, RSTART+9, RLENGTH-9) + 0 > 0) ok = 1 } } \
	           END { if (!ok) { print "no VERIFY PASS with compared>0" > "/dev/stderr"; exit 1 } }' \
	        $(WORK)/verify.out; \
	    done; \
	  done; \
	done
	@set -e; test -z "$(AB)" || for k in $(KERNELS); do \
	  widths=`sed -n 's/^# cpus=[0-9]* widths=//p' $(WORK)/$$k.forms`; \
	  for w in $$widths; do \
	    echo "== $$k-b wf width=$$w"; \
	    WF_WORKERS=$$w $(BUILD)/$$k-b verify wf $$w > $(WORK)/verify.out 2>&1 \
	      || { cat $(WORK)/verify.out; exit 1; }; \
	    cat $(WORK)/verify.out >> $(WORK)/verify.log; \
	    awk '/VERIFY PASS/ { if (match($$0, /compared=[0-9]+/)) { \
	           if (substr($$0, RSTART+9, RLENGTH-9) + 0 > 0) ok = 1 } } \
	         END { if (!ok) { print "no VERIFY PASS with compared>0" > "/dev/stderr"; exit 1 } }' \
	      $(WORK)/verify.out; \
	  done; \
	done
	@echo "verify PASS: $(WORK)/verify.log"

# ----------------------------------------------------------------- compare --
# A run never overwrites an earlier table, so RESULTS must not exist. Anyone
# re-running by hand needs a fresh RESULTS=; `make clean` also clears the way,
# but it removes the whole work directory, every earlier table in it included.
# manifest.txt records the WHOLE `rustc -vV` -- host triple, commit hash and
# date, LLVM version -- one `rustc: ` line each, because the release number
# alone does not identify the compiler that built the Rayon reference. The
# table header has room for one line, so the reducer below is handed the first.
#
# The twin is a CELL LIKE ANY OTHER. `wf-b` goes into the same per-pass cell
# list, is rotated and reversed by the same rule, and runs as its own process
# -- so the twin and the plain image alternate within a pass and the reducer
# can pair them per pass instead of dividing two run medians. The `-b` image
# calls its own form `wf`, because it is the same harness; the one awk below
# rewrites that one field to `wf-b` on its way into raw.tsv, on the header, the
# trailer and the data rows alike, which is the whole of the twin's presence in
# the recorded stream. Its stdout is kept in logs/ first rather than piped, so
# a failing twin process still fails the run: /bin/sh has no pipefail.
compare: build
	@test ! -e "$(RESULTS)" || { \
	    echo 'compute-bench: $(RESULTS) already exists; a run never overwrites an' >&2; \
	    echo 'earlier table. Pass a fresh RESULTS= or run `make clean`.' >&2; exit 1; }
	@mkdir -p $(RESULTS)/logs
	@set -e; for k in $(KERNELS); do $(SHA256) $(BUILD)/$$k $(BUILD)/$$k-par.ll $(BUILD)/$$k-seq.ll; \
	  test -z "$(AB)" || $(SHA256) $(BUILD)/$$k-b $(BUILD)/$$k-par-b.ll; done > $(RESULTS)/hashes-before.txt
	@set -e; { \
	  echo "uname: $$(uname -a)"; \
	  echo "cpus_online: $$(getconf _NPROCESSORS_ONLN)"; \
	  if [ -d /sys/devices/system/cpu/cpu0/topology ]; then \
	     for d in /sys/devices/system/cpu/cpu[0-9]*/topology; do \
	       c=$${d%/topology}; c=$${c##*/}; \
	       echo "topology: $$c core=$$(cat $$d/core_id 2>/dev/null) package=$$(cat $$d/physical_package_id 2>/dev/null) siblings=$$(cat $$d/thread_siblings_list 2>/dev/null)"; \
	     done; \
	  elif [ "$$(uname -s)" = Darwin ] && command -v sysctl > /dev/null 2>&1; then \
	     echo "topology: physicalcpu=$$(sysctl -n hw.physicalcpu) logicalcpu=$$(sysctl -n hw.logicalcpu)"; \
	  else echo "topology: unqualified (sysfs topology absent)"; fi; \
	  if [ -r /sys/devices/system/cpu/smt/active ]; then \
	     echo "smt: $$(cat /sys/devices/system/cpu/smt/active)"; \
	  else echo "smt: unqualified (file absent)"; fi; \
	  if command -v taskset > /dev/null 2>&1; then echo "mask: $$(taskset -pc $$$$ 2>&1)"; \
	  else echo "mask: unqualified (no taskset on this host)"; fi; \
	  if [ -r /sys/fs/cgroup/cpu.max ]; then echo "cgroup_cpu_max: $$(cat /sys/fs/cgroup/cpu.max)"; \
	  else echo "cgroup_cpu_max: unqualified (file absent)"; fi; \
	  if [ -r /sys/fs/cgroup/cpuset.cpus.effective ]; then \
	     echo "cpuset: $$(cat /sys/fs/cgroup/cpuset.cpus.effective)"; \
	  else echo "cpuset: unqualified (file absent)"; fi; \
	  if git -C $(ROOT) rev-parse HEAD > /dev/null 2>&1; then \
	     echo "compiler_revision: $$(git -C $(ROOT) rev-parse HEAD)"; \
	  else echo "compiler_revision: unknown"; fi; \
	  $(CC) --version | head -1 | sed 's/^/clang: /'; \
	  $(CXX) --version | head -1 | sed 's/^/clang++: /'; \
	  rustc -vV | sed 's/^/rustc: /'; \
	  cargo --version | sed 's/^/cargo: /'; \
	  cmake --version | head -1 | sed 's/^/cmake: /'; \
	  echo "CFLAGS: $(CFLAGS)"; \
	  echo "CXXFLAGS: $(CXXFLAGS)"; \
	  echo "WF_FLAGS: $(WF_FLAGS)"; \
	  echo "WF_PAR_CONTROL_FLAGS: $(WF_PAR_CONTROL_FLAGS)"; \
	  echo "WF_RUNTIME_CONTROL_FLAGS: $(WF_RUNTIME_CONTROL_FLAGS)"; \
	  echo "WF_MODULE_CONTROL_FLAGS: $(WF_MODULE_CONTROL_FLAGS)"; \
	  echo "WF_AB_TWIN: $(if $(AB),yes (the wf-b row; the wf row is plain),no)"; \
	  echo "WF_B_SOURCE: $(if $(strip $(WF_B_SOURCE)),$(strip $(WF_B_SOURCE)),$(if $(BASELINE),unlabelled (WF_B_SOURCE was not set),this tree))"; \
	  test -z "$(BASELINE)" || echo "WF_B_SCHED_DIR: $(WF_B_SCHED_DIR)"; \
	  test -z "$(BASELINE)" || echo "WF_B_FLOOR: $(WF_B_FLOOR)"; \
	  test -z "$(BASELINE)" || echo "WF_B_WFC: $(WF_B_WFC)"; \
	  test -z "$(AB)" || sed 's/^/WF_B_SOURCE_SHA256: /' $(BASELINE_STAMP); \
	  echo "RFLAGS: $(RFLAGS)"; \
	  echo "BENCH_ARCH: $(BENCH_ARCH)"; \
	  echo "pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0"; \
	  echo "passes: $(PASSES)"; echo "calls: $(CALLS)"; \
	  cat $(RESULTS)/hashes-before.txt; \
	  cat $(DEPS)/metadata.txt; \
	} > $(RESULTS)/manifest.txt
	@set -e; : > $(RESULTS)/raw.tsv; ki=0; \
	for k in $(KERNELS); do \
	  $(BUILD)/$$k list > $(RESULTS)/logs/$$k.forms; \
	  widths=`sed -n 's/^# cpus=[0-9]* widths=//p' $(RESULTS)/logs/$$k.forms`; \
	  forms=`awk -F'\t' '!/^#/ && $$3 != "n/a" { print $$1 }' $(RESULTS)/logs/$$k.forms`; \
	  : > $(RESULTS)/logs/$$k.cells; \
	  for w in $$widths; do for form in $$forms; do \
	    if [ $$w -eq 1 ]; then \
	      case $$form in wf|wf-seq|serial) ;; *) continue;; esac; \
	    else \
	      case $$form in wf-seq|serial) continue;; esac; \
	    fi; \
	    echo "$$form $$w" >> $(RESULTS)/logs/$$k.cells; \
	    if [ "$$form" = wf ] && [ -n "$(AB)" ]; then echo "wf-b $$w" >> $(RESULTS)/logs/$$k.cells; fi; \
	  done; done; \
	  n=`awk 'END { print NR }' $(RESULTS)/logs/$$k.cells`; \
	  p=0; while [ $$p -lt $(PASSES) ]; do \
	    off=$$(( (p + ki) % n )); rev=$$(( p % 2 )); \
	    awk -v off=$$off -v n=$$n -v rev=$$rev \
	      '{ a[NR] = $$0 } END { for (i = 0; i < n; i++) { \
	          j = rev ? n - 1 - i : i; print a[(off + j) % n + 1] } }' \
	      $(RESULTS)/logs/$$k.cells > $(RESULTS)/logs/$$k-$$p.order; \
	    while read -r form w; do \
	      echo "== pass $$p: $$k $$form width=$$w"; \
	      if [ "$$form" = wf-b ]; then \
	        WF_WORKERS=$$w $(BUILD)/$$k-b time wf $$w $$p $(CALLS) \
	          > $(RESULTS)/logs/$$k-wf-b-w$$w-p$$p.out 2> $(RESULTS)/logs/$$k-$$form-w$$w-p$$p.log \
	          || { cat $(RESULTS)/logs/$$k-$$form-w$$w-p$$p.log >&2; exit 1; }; \
	        awk -F'\t' -v OFS='\t' \
	          '/^# driver=/ || /^# batch / { sub(/ form=wf /, " form=wf-b "); print; next } \
	           /^#/ { print; next } \
	           { $$2 = "wf-b"; print }' \
	          $(RESULTS)/logs/$$k-wf-b-w$$w-p$$p.out >> $(RESULTS)/raw.tsv; \
	      else \
	        WF_WORKERS=$$w $(BUILD)/$$k time $$form $$w $$p $(CALLS) \
	          >> $(RESULTS)/raw.tsv 2> $(RESULTS)/logs/$$k-$$form-w$$w-p$$p.log \
	          || { cat $(RESULTS)/logs/$$k-$$form-w$$w-p$$p.log >&2; exit 1; }; \
	      fi; \
	    done < $(RESULTS)/logs/$$k-$$p.order; \
	    p=$$((p + 1)); \
	  done; \
	  ki=$$((ki + 1)); \
	done
	@awk -f reduce.awk -v passes=$(PASSES) -v calls=$(CALLS) \
	    -v host="$$(uname -sm)" -v cpus="$$(getconf _NPROCESSORS_ONLN)" \
	    -v mask="$$(sed -n 's/^mask: //p' $(RESULTS)/manifest.txt)" \
	    -v date="$$(date -u +%Y-%m-%dT%H:%M:%SZ)" -v run="$${GITHUB_RUN_ID:-local}" \
	    -v compiler="$$(sed -n 's/^compiler_revision: //p' $(RESULTS)/manifest.txt)" \
	    -v clang="$$(sed -n 's/^clang: //p' $(RESULTS)/manifest.txt)" \
	    -v rustc="$$(sed -n 's/^rustc: //p' $(RESULTS)/manifest.txt | head -1)" \
	    -v refflags="$(CFLAGS)" -v wfflags="$(WF_FLAGS)" \
	    -v parcontrol="$(WF_PAR_CONTROL_FLAGS)" \
	    -v runtimecontrol="$(WF_RUNTIME_CONTROL_FLAGS)" \
	    -v modulecontrol="$(WF_MODULE_CONTROL_FLAGS)" \
	    -v bsource="$(if $(BASELINE),$(if $(strip $(WF_B_SOURCE)),$(strip $(WF_B_SOURCE)),unlabelled),)" \
	    -v pins="oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0" \
	    $(RESULTS)/raw.tsv | tee $(RESULTS)/table.txt
	@set -e; for k in $(KERNELS); do $(SHA256) $(BUILD)/$$k $(BUILD)/$$k-par.ll $(BUILD)/$$k-seq.ll; \
	  test -z "$(AB)" || $(SHA256) $(BUILD)/$$k-b $(BUILD)/$$k-par-b.ll; done > $(RESULTS)/hashes-after.txt
	@set -e; { echo "--- after the run ---"; \
	  cat $(RESULTS)/hashes-after.txt; \
	} >> $(RESULTS)/manifest.txt
	@cmp -s $(RESULTS)/hashes-before.txt $(RESULTS)/hashes-after.txt \
	    || { echo "compute-bench: a kernel image or emitted module changed during" >&2; \
	         echo "the run; the table is not a measurement of one build." >&2; \
	         diff $(RESULTS)/hashes-before.txt $(RESULTS)/hashes-after.txt >&2; exit 1; }
	@echo "compare PASS: $(RESULTS)/table.txt"

# ---------------------------------------------------------- programs-check --
# The cheap gate target, expressed as one target per kernel. A `for ... done`
# loop returns the status of the last command of the last iteration, so a
# rejection of any kernel but the last, or a failed publish-site grep on any
# but the last, would leave the target green and the hole it exists to close
# open. Each recipe line is its own shell and make stops at the first failure.
# The stamp depends on the compiler as well as the source: what this target
# asserts is a property of what whitefootc emits, so a rebuilt compiler makes
# every stamp stale even when no program changed. `programs-check` builds
# $(WFC) through `compiler` before the sub-make reads it as a prerequisite.
programs-check: compiler
	@mkdir -p $(CHECK)
	$(MAKE) -j$(JOBS) $(CHECKED)

$(CHECK)/checked_%: programs/%.wf $(WFC)
	$(WFC) --par        --emit-llvm programs/$*.wf -o $(CHECK)/$*-par.ll
	$(WFC) --no-overlap --emit-llvm programs/$*.wf -o $(CHECK)/$*-seq.ll
	grep -q 'call void @wf__par_publish(' $(CHECK)/$*-par.ll
	! grep -q 'call void @wf__par_publish(' $(CHECK)/$*-seq.ll
	touch $@

# ------------------------------------------------------------------ clean ---
clean:
	rm -rf $(WORK)
# Kept separate: the pinned source cache is expensive and should survive an
# ordinary clean. It removes the installed dependency tree with the cache, so
# the next `make deps` refetches and rebuilds every dependency from scratch --
# including the Rayon staticlib, which `deps.sh build` otherwise keeps when it
# and the recovered symbols file are already there.
clean-deps:
	rm -rf $(WHITEFOOT_SCRATCH_ROOT)/whitefoot-compute-deps $(DEPS)

# `.SECONDARY:` with no prerequisites is deliberately NOT used here: the
# manual makes every target intermediate, and make 4.3 then declines to
# recreate a missing one while its consumer is up to date. The emitted
# modules and the split headers are named targets and named prerequisites
# of their image instead, so they are never intermediate and never deleted.
.DELETE_ON_ERROR:

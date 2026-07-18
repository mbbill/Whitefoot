# Headline Artifact Shortlist (brainstorm synthesis, 2026-07-10)

Source: 5-lens opus ideation panel (security-press, dev-virality,
invisible-ubiquity, novel-capability, rust-skeptic) + owner directives D7/D7a.
Constant-time-crypto carded separately at Fable tier (see ct card).

## The convergence signal

FOUR of five independent lenses surfaced the SAME top candidate unprompted:
**zlib `inflate` as an ABI-compatible `libz.so` drop-in.** When lenses with
different objectives converge, that is the pick to trust. Runner-up
convergence: LZ4/Snappy block codecs, and the "tiny ubiquitous parser" family
(base64, CRC32, DER, protobuf-varint, DNS).

## Tier S — lead candidate

### S1. libz drop-in (inflate first, then crc32/adler32, then deflate)
Headline: "An AI wrote a drop-in `libz.so` — bit-identical, C-fast, and the
CVE-2022-37434 heap-overflow class is a compile error."
- Swap-in: GOLD. LD_PRELOAD/symlink over libz.so.1; every dynamically-linked
  program (git, web servers, package managers) picks it up with no relink.
- vs Rust wave: zlib-rs exists and ships in Firefox/Chromium paths — BUT it
  carries `unsafe` in SIMD/pointer hot loops (bug class only mostly gone) and
  is human-written. Whitefoot: zero unsafe incl. fast path + AI-authored.
- Verify: bit-identical over Canterbury/Silesia + real blobs; zlib's own test
  suite; libFuzzer fuzz-diff; replay the CVE-2022-37434 PoC -> clean reject.
- Rung: NEAR. Gap: growable/streaming output buffers (the one real blocker);
  Huffman tables (near const-arrays); window/state over byte buffers (have).
- Risk: exact streaming/flush/error-code parity is fiddly; zlib-rs set a high
  safe-speed bar, so Huffman inner-loop parity must be shown not asserted.
- STAGING: crc32/adler32 is a NOW-rung single-symbol LD_PRELOAD (bit-identical
  checksum inside unmodified gzip) — ship it first as the walking-skeleton of
  the whole ABI-swap + fuzz-diff protocol, then inflate, then deflate.

## Tier A — strong, distinct angles

### A1. LZ4 / Snappy block decompressor (data-infra ubiquity)
Headline: "The codec inside Kafka/RocksDB/ClickHouse, AI-rewritten: same
speed, wildcopy-overflow class unrepresentable."
- Swap-in: drop-in liblz4/snappy; runs live inside databases. Verify:
  bit-identical + replay LZ4 CVE-2019-17543. Rung: near (simpler than zlib —
  no Huffman; LZ4 decode is pure copy loops). Risk: the safe bounds checks on
  the match-copy loop are exactly where LZ4's speed lives — parity is the
  whole game; strong channel-1/OP-4-elision showcase.

### A2. simdjson-class JSON validator/minifier (dev virality)
Headline: "simdjson's GB/s, none of the unsafe — AI-written."
- Swap-in: CLI + lib. Verify: bit-identical minify + accept/reject vs jq/
  simdjson. Rung: near (needs the strings/UTF-8 story). Risk: simdjson is a
  SIMD monument; "GB/s in pure checked scalar+autovec" is unproven and may not
  hold — pick validator/minifier (structural) not full DOM parse first.

### A3. UTF-8 validator (simdutf-class) — NOW rung, tiny, ubiquitous
Headline: "AI-written UTF-8 validation, bit-identical verdicts, overlong-
encoding bug class gone." Swap-in: lib/CLI. Rung: NOW (pure byte scan, in
subset today). Best "prove the pipeline on something trivial this week"
candidate alongside crc32.

### A4. DER/X.509 + DNS message parsers (security infra)
Headline: "Two decades of ASN.1 / DNS-compression memory-corruption CVEs,
unrepresentable in an AI-written parser." Swap-in: weaker (library, not one
ABI). Rung: near. Best PURE-SECURITY story; less "felt speed."

## Tier B — capstones (compose Tier S/A), later
PNG (needs inflate+crc32), safe-webp VP8L (BLASTPASS headline), safe-mp4
Stagefright demuxer, stb_image loader. Each is a Tier-S/A composition once the
kernels exist. safe-webp has the single BIGGEST security headline (BlastPass /
Chrome 0-day) but is a near+ rung.

## Language capability showcases (not swap-in artifacts, but press-worthy)
- Threaded/musttail interpreter dispatch from naive source (Rust CANNOT
  express) — channel-4, anchored to owner's Silverfir engine.
- Checked algebraic laws refusing false optimizations (channel-3, built).
- Effect-typed LTO-class hoisting across .so boundary without LTO (channel-2,
  built).
- Constant-time crypto via `secret` effect (carded separately, Fable tier).

## Recommended path
1. crc32 LD_PRELOAD (NOW) — walking skeleton: proves ABI-swap + fuzz-diff +
   "runs inside unmodified gzip" end to end on a trivial kernel. ~days.
2. Build the two gating features it exposes are minimal; then LZ4 decode
   (near, simplest real codec) as second rung.
3. inflate (the S1 headline) once growable buffers land.
4. Capstone: PNG or safe-webp for the security megaphone.
Everyday-felt + swap-in + AI-authored + provable-property, escalating rung by
rung, each shippable and independently newsworthy.

use std::os::unix::process::ExitStatusExt;

use super::support::{
    CompiledProgram, build_program, compile_and_run, compile_programs, emitted_function,
    fixture_directory,
};

/// The accepted compressed input length in `tests/programs/raw_deflate_boundary.wf`.
///
/// The driver reads one byte past it to tell a file it can hold from a file it
/// cannot; nothing in its contract exposes the length.
const ACCEPTED_INPUT_LENGTH: usize = 4096;

/// The raw DEFLATE stream `research/experiments/raw-deflate-default-shape`
/// records as fixture `stock-zlib-l6-default-strategy-text`: its pinned zlib at
/// level 6 and the default strategy over [`stock_text_payload`], emitted as one
/// final dynamic block, so decoding it runs `decode_dynamic`.
const STOCK_DYNAMIC_TEXT: &[u8] = &[
    0xed, 0xca, 0x47, 0x0a, 0xc2, 0x40, 0x00, 0x05, 0x50, 0x7b, 0x89, 0xbd, 0x77, 0x1d, 0x7b, 0x4d,
    0xec, 0xbd, 0x21, 0x18, 0x57, 0x2e, 0x85, 0xac, 0x07, 0x4d, 0x88, 0x20, 0x4c, 0x48, 0x26, 0x78,
    0x7d, 0x4f, 0xe0, 0x0d, 0xfe, 0x5b, 0x3f, 0x45, 0x7f, 0x73, 0x55, 0x63, 0x8c, 0x13, 0x93, 0x7e,
    0xc9, 0x55, 0xbe, 0xdd, 0x2f, 0x0f, 0x99, 0xbc, 0x54, 0x8d, 0xda, 0x1f, 0x2e, 0x5a, 0x3a, 0x35,
    0x54, 0xf2, 0x64, 0xa6, 0x61, 0x5b, 0x92, 0xa0, 0xe0, 0xe2, 0xe2, 0xe2, 0xe2, 0xe2, 0xe2, 0xe2,
    0xe2, 0xe2, 0xe2, 0xe2, 0xe2, 0xe2, 0xe2, 0xe2, 0xe2, 0xe2, 0xe2, 0xfe, 0xbd, 0x0e, 0xa7, 0xcb,
    0xed, 0xf1, 0xfa, 0xfc, 0x81, 0xa0, 0x10, 0x0a, 0x47, 0xa2, 0xb1, 0x78, 0x22, 0x99, 0x4a, 0x67,
    0xb2, 0xb9, 0x7c, 0xa1, 0x58, 0x2a, 0x57, 0xaa, 0x35, 0x52, 0x6f, 0x34, 0x5b, 0xed, 0x4e, 0xb7,
    0xd7, 0x1f, 0x0c, 0x47, 0xa2, 0x34, 0x9e, 0x4c, 0x67, 0xf3, 0xc5, 0x72, 0xb5, 0xde, 0x6c, 0x77,
    0xfb, 0xc3, 0xf1, 0x74, 0xfe, 0x01,
];

/// The same corpus's `malformed-dynamic-literal-oversubscribed`: a dynamic
/// block whose literal/length code lengths over-subscribe the tree.
const MALFORMED_LITERAL_TREE: &[u8] = &[
    0x05, 0xc0, 0x01, 0x04, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00,
];

/// The payload the corpus compresses into [`STOCK_DYNAMIC_TEXT`], which is
/// therefore the exact byte sequence decoding it must produce.
fn stock_text_payload() -> Vec<u8> {
    let mut payload = b"Whitefoot raw DEFLATE default-shape corpus.\n".repeat(113);
    payload.extend(0u8..64);
    payload
}

fn boundary_driver() -> CompiledProgram {
    build_program(&compile_programs(&[
        "raw_deflate.wf",
        "raw_deflate_dynamic.wf",
        "raw_deflate_dynamic_decode.wf",
        "raw_deflate_boundary.wf",
    ]))
}

#[test]
fn stored_fixed_and_dynamic_blocks_execute_with_data_failures() {
    let llvm = compile_programs(&[
        "raw_deflate.wf",
        "raw_deflate_dynamic.wf",
        "raw_deflate_dynamic_decode.wf",
        "raw_deflate_vectors.wf",
    ]);
    let inflate = emitted_function(&llvm, "inflate");
    assert!(inflate.contains("call void @free"));
    let length = emitted_function(&llvm, "decode_length");
    let distance = emitted_function(&llvm, "copy_distance");
    assert!(length.contains("icmp ult i64"));
    assert!(length.contains("call void @wf_trap"));
    assert!(distance.contains("icmp ult i64"));
    assert!(distance.contains("call void @wf_trap"));
    let table = emitted_function(&llvm, "build_huffman_table");
    assert!(table.contains("call ptr @malloc"));
    assert!(table.contains("call void @wf_trap"));
    let dynamic = emitted_function(&llvm, "decode_dynamic");
    assert!(dynamic.contains("call void @free"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// The decoder reached through a real system boundary rather than a stream the
/// program builds for itself.
///
/// `tests/programs/raw_deflate_boundary.wf` names its input on the command
/// line, resolves it against the command's working-directory capability, and
/// reads it with `read_once`, so every table index the decoder computes derives
/// from bytes that entered through the boundary. The compressed bytes and the
/// expected output both come from the recorded correctness corpus, so this case
/// checks the decoder against that oracle and not against itself.
#[test]
fn the_boundary_driver_decodes_a_file_read_through_the_system_path() {
    let program = boundary_driver();
    let directory = fixture_directory();
    directory.write(b"stream.deflate", STOCK_DYNAMIC_TEXT);

    let output = program.run(directory.path(), &[b"stream.deflate"]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, stock_text_payload());
    assert!(output.stderr.is_empty());
}

/// Every outcome the boundary can produce reaches its own status.
///
/// A malformed or truncated input is an ordinary outcome of reading a real
/// file, so none of these may trap or be absorbed into the success path.
#[test]
fn each_boundary_and_decode_outcome_reaches_its_own_status() {
    let program = boundary_driver();
    let directory = fixture_directory();
    directory.write(b"stream.deflate", STOCK_DYNAMIC_TEXT);
    directory.write(b"empty.deflate", b"");
    // The stream ends inside the dynamic block's code lengths, which is what a
    // read that stops before the stream does looks like to the decoder.
    directory.write(b"truncated.deflate", &STOCK_DYNAMIC_TEXT[..60]);
    directory.write(b"malformed.deflate", MALFORMED_LITERAL_TREE);
    directory.write(b"oversize.deflate", &vec![0_u8; ACCEPTED_INPUT_LENGTH + 1]);

    let cases: &[(&[&[u8]], i32, &[u8])] = &[
        (&[], 1, b"usage: raw_deflate_boundary FILE\n"),
        (
            &[b"absent.deflate"],
            2,
            b"raw_deflate_boundary: cannot read the compressed input\n",
        ),
        (
            &[b"empty.deflate"],
            3,
            b"raw_deflate_boundary: empty compressed input\n",
        ),
        (
            &[b"oversize.deflate"],
            4,
            b"raw_deflate_boundary: compressed input exceeds the input buffer\n",
        ),
        (
            &[b"truncated.deflate"],
            5,
            b"raw_deflate_boundary: compressed stream ends early\n",
        ),
        (
            &[b"malformed.deflate"],
            6,
            b"raw_deflate_boundary: malformed compressed stream\n",
        ),
    ];
    for (arguments, code, diagnostic) in cases {
        let output = program.run(directory.path(), arguments);
        assert_eq!(
            output.status.code(),
            Some(*code),
            "unexpected status for {arguments:?}"
        );
        assert_eq!(output.stderr, *diagnostic, "unexpected diagnostic");
        assert!(output.stdout.is_empty(), "a failed decode published bytes");
    }

    // A destination with no reader is the one outcome no fixture content can
    // produce, so it needs the real mechanism.
    let (status, diagnostics) =
        program.run_with_closed_output(directory.path(), &[b"stream.deflate"]);
    assert_eq!(
        status.signal(),
        None,
        "a write to a closed destination must not kill the process"
    );
    assert_eq!(status.code(), Some(8));
    assert_eq!(
        diagnostics,
        b"raw_deflate_boundary: cannot publish the decoded output\n"
    );
}

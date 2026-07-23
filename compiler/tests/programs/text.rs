use super::support::{compile_and_run, compile_program, emitted_function};

#[test]
fn percent_decoder_executes_through_the_ordinary_pipeline() {
    let llvm = compile_program("percent_decode.wf");
    let decode = emitted_function(&llvm, "decode");
    assert!(decode.contains("icmp ult i64"));
    assert!(decode.contains("call void @wf_trap"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn utf8_parser_executes_through_the_ordinary_pipeline() {
    let llvm = compile_program("utf8parse.wf");
    let parse = emitted_function(&llvm, "parse");
    assert!(parse.contains("icmp ult i64"));
    assert!(parse.contains("call void @wf_trap"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn recursive_prefix_parser_builds_evaluates_and_drops_its_ast() {
    let llvm = compile_program("prefix_expression.wf");
    let parser = emitted_function(&llvm, "parse_expression");
    assert!(parser.contains("call"));
    assert!(parser.contains("@wf_parse_expression"));
    assert!(llvm.contains("call ptr @malloc"));
    assert!(llvm.contains("call void @free"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

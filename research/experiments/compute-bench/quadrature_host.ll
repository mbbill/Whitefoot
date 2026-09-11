; Serves compute-bench: the quadrature host adapter. It is LLVM IR rather than
; C because every user function the compiler emits has internal linkage, so
; nothing outside the module can call @wf_integrate. It forwards one
; integration and computes nothing; the batch loop lives in
; quadrature_bench.c, so the .wf source and this file are unchanged by it.
; The Makefile appends this file to each emitted module and sed-renames the
; entry point to the -par or -seq spelling, so one text serves both. This
; kernel returns a scalar, so it has no buffer and no release entry point.
define double @wf_bench_quadrature(double %a, double %b, double %center, double %width, double %tolerance, i64 %depth) {
  %r = call double @wf_integrate(double %a, double %b, double %center, double %width, double %tolerance, i64 %depth)
  ret double %r
}

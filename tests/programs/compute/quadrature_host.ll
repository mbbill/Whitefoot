; Scalar/pointer host ABI for the formal quadrature fixture.
; The shared adapter binder selects the entry-equivalent execution world.
define double @wf_bench_quadrature(double %a, double %b, double %center, double %width, double %tolerance, i64 %depth) {
  %r = call double @wf_integrate(double %a, double %b, double %center, double %width, double %tolerance, i64 %depth)
  ret double %r
}

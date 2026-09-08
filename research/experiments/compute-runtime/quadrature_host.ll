; Research adapter for this emitted module, not a public WF ABI.
define double @wf_research_quadrature(double %a, double %b, double %center, double %width, double %tolerance, i64 %depth, i1 %parallel) {
  br i1 %parallel, label %par, label %seq
par:
  %p = call double @wf_integrate(double %a, double %b, double %center, double %width, double %tolerance, i64 %depth)
  ret double %p
seq:
  %s = call double @wf__par_seq_integrate(double %a, double %b, double %center, double %width, double %tolerance, i64 %depth)
  ret double %s
}

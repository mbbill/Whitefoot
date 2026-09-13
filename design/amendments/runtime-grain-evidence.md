Node: design/compiler/parallel-lowering/parallel-runtime.md

Decision: The independent-map splitter retains its 150,000 work unit and 16-chunks-per-lane cap as the current static estimate while runtime helper work remains an open cost, because the original Mandelbrot measurements support those constants and the compute-model experiment's 10,000-unit control improves wide blocked work but adds 36 percent wall time to a narrow-row stencil and about 90 percent to a chain's pull traversal at four workers, instead of lowering the global work unit to compensate for missing runtime trip counts or treating the former kernels' grain plateau as universal. This replaces the existing work-unit decision's grounds, preserves its constants, and adds the adverse evidence in research/investigations/compute-model/DESIGN.md#grain-attribution-control; it does not select a dynamic estimator.

Rejected:
- A global 10,000 work unit as the runtime-block fix: rejected because the same outer static weight prices both wide and narrow helper work, and the measured gains on prefix and histogram transfer a material cost to the narrow stencil and high-diameter pull control.

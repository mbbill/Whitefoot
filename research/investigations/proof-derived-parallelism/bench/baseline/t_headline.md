| config | best WF | which | best Rust | which | WF/Rust | best WF vs Rust-seq |
|---|---:|---|---:|---|---:|---:|
| `bal_d8_w16` | 0.550 | wf_par/1 | 0.253 | rs_cut/4 | 2.17x | 0.95x unres. |
| `bal_d8_w64` | 0.486 | wf_par/2 | 0.185 | rs_cut/4 | 2.62x | 0.99x unres. |
| `bal_d8_w192` | 0.405 | wf_par/2 | 0.189 | rs_rayon/4 | 2.14x | 0.67x |
| `bal_d10_w16` | 0.554 | wf_par/1 | 0.191 | rs_cut/4 | 2.91x | 0.97x unres. |
| `bal_d10_w64` | 0.310 | wf_par/2 | 0.199 | rs_rayon/4 | 1.56x | 0.56x |
| `bal_d10_w192` | 0.332 | wf_par/2 | 0.187 | rs_cut/8 | 1.78x | 0.49x |
| `bal_d12_w16` | 0.601 | wf_par/1 | 0.223 | rs_cut/8 | 2.70x | 0.94x unres. |
| `bal_d12_w64` | 0.290 | wf_par/2 | 0.182 | rs_cut/8 | 1.59x | 0.51x |
| `bal_d12_w192` | 0.292 | wf_par/4 | 0.135 | rs_rayon/8 | 2.16x | 0.48x |
| `skew_d16_w16` | 0.575 | wf_par/1 | 0.287 | rs_cut/4 | 2.00x | 0.83x |
| `skew_d16_w64` | 0.454 | wf_par/2 | 0.202 | rs_rayon/4 | 2.24x | 0.81x |
| `skew_d16_w192` | 0.499 | wf_par/8 | 0.152 | rs_rayon/8 | 3.29x | 0.81x |

| config | best WF | which | best Rust | which | WF/Rust | best WF vs Rust-seq |
|---|---:|---|---:|---|---:|---:|
| `bal_d8_w16` | 0.223 | wf_par/4 | 0.260 | rs_cut/4 | 0.85x unres. | 0.39x |
| `bal_d8_w64` | 0.151 | wf_par/4 | 0.182 | rs_cut/4 | 0.83x | 0.31x |
| `bal_d8_w192` | 0.158 | wf_par/default | 0.180 | rs_cut/4 | 0.88x unres. | 0.26x |
| `bal_d10_w16` | 0.177 | wf_par/4 | 0.179 | rs_cut/4 | 0.99x unres. | 0.31x |
| `bal_d10_w64` | 0.141 | wf_par/4 | 0.147 | rs_cut/4 | 0.96x unres. | 0.28x |
| `bal_d10_w192` | 0.124 | wf_par/default | 0.142 | rs_cut/8 | 0.87x unres. | 0.21x |
| `bal_d12_w16` | 0.166 | wf_par/default | 0.158 | rs_cut/8 | 1.05x unres. | 0.29x |
| `bal_d12_w64` | 0.115 | wf_par/default | 0.121 | rs_cut/8 | 0.95x unres. | 0.23x |
| `bal_d12_w192` | 0.114 | wf_par/default | 0.128 | rs_rayon/default | 0.89x unres. | 0.19x |
| `skew_d16_w16` | 0.191 | wf_par/4 | 0.189 | rs_cut/4 | 1.01x unres. | 0.33x |
| `skew_d16_w64` | 0.140 | wf_par/default | 0.162 | rs_rayon/4 | 0.86x unres. | 0.28x |
| `skew_d16_w192` | 0.124 | wf_par/default | 0.143 | rs_rayon/8 | 0.87x unres. | 0.20x |
| `grid_d21_w256` | 0.079 | wf_par/default | 0.077 | rs_rayon/default | 1.02x unres. | 0.16x |

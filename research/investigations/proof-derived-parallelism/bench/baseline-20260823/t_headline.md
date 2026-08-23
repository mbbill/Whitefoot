| config | best WF | which | best Rust | which | WF/Rust | best WF vs Rust-seq |
|---|---:|---|---:|---|---:|---:|
| `bal_d8_w16` | 0.225 | wf_par/4 | 0.255 | rs_cut/4 | 0.88x unres. | 0.40x |
| `bal_d8_w64` | 0.180 | wf_par/4 | 0.179 | rs_cut/4 | 1.01x unres. | 0.37x |
| `bal_d8_w192` | 0.172 | wf_par/default | 0.180 | rs_cut/4 | 0.96x unres. | 0.29x |
| `bal_d10_w16` | 0.179 | wf_par/4 | 0.176 | rs_cut/4 | 1.02x unres. | 0.32x |
| `bal_d10_w64` | 0.151 | wf_par/default | 0.145 | rs_cut/4 | 1.04x unres. | 0.30x |
| `bal_d10_w192` | 0.139 | wf_par/default | 0.139 | rs_cut/8 | 1.00x unres. | 0.23x |
| `bal_d12_w16` | 0.167 | wf_par/default | 0.155 | rs_cut/8 | 1.08x unres. | 0.29x |
| `bal_d12_w64` | 0.125 | wf_par/default | 0.120 | rs_cut/8 | 1.04x unres. | 0.25x |
| `bal_d12_w192` | 0.128 | wf_par/default | 0.126 | rs_rayon/default | 1.01x unres. | 0.22x |
| `skew_d16_w16` | 0.182 | wf_par/4 | 0.185 | rs_cut/4 | 0.98x unres. | 0.31x |
| `skew_d16_w64` | 0.137 | wf_par/default | 0.162 | rs_rayon/4 | 0.84x unres. | 0.27x |
| `skew_d16_w192` | 0.121 | wf_par/default | 0.142 | rs_rayon/8 | 0.85x unres. | 0.20x |
| `grid_d21_w256` | 0.077 | wf_par/default | 0.076 | rs_rayon/default | 1.01x unres. | 0.16x |

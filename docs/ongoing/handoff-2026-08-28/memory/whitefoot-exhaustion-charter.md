---
name: whitefoot-exhaustion-charter
description: "Owner 2026-08-23: any reachable segfault is a WF design failure; resource exhaustion must become a controlled, designed event — chartered research direction"
metadata: 
  node_type: memory
  type: project
  originSessionId: ab659700-c818-4271-84f2-e403a962b810
  modified: 2026-08-23T08:38:51.189Z
---

Owner charter (2026-08-23, verbatim): "只要程序有可能跑到segfault,那就是wf
的设计失败了。wf设计目标就是绝对的可靠和和性能。现在连trap都是在正确程序里
面不可能的事情,但却会因为资源耗尽而segfault,感觉有点不可接受。我们得像个
合适的办法来控制资源。但这个问题可能比较复杂,需要好好考虑"

**Why:** the guard-page SIGSEGV keeps memory safety but is an undiagnosed,
uncontrolled death whose ceiling is a compiler artifact (LLVM IPCP moved a
spine frame 16B→48B = 3x depth); indistinguishable from corruption from
outside. Heap-failure behavior uncounted. [SCOPE-3] currently carves
exhaustion out as host territory — that carve-out is the thing to replace.

**How to apply:** design space graded (2026-08-23 ideation, six dossiers in
do_not_scan/wf-exhaust/): (1) clean-death floor — sigaltstack handler +
async-signal-safe diagnosed abort, zero happy-path cost, NOT a second trap
class (SCOPE-3 rewrite, distinct from SCOPE-4 claims); (2) stack ledger —
compiler emits per-function/per-SCC frame economics, visibility like
--par-ledger; (3) proof-derived depth bounds — far goal, smallest provable
class first; (4) WF-unique: proof-derived stack segmentation — hand-out
sites as subtree-granularity segmentation points (measured: the parallel
default already RAISES tree-shape ceilings by spreading subtrees across
worker stacks; pure spines with no eligible site cannot be saved this way);
(5) recovery-as-typed-error likely rejected (signature pollution, growable
stacks need pointer maps). Constraints: resources never language concepts;
zero happy-path cost; claims are never resource checks. Spec work targets
the post-rebase lineage (main's v0.34+). Sequenced after: claim-redirect
landing, 0077 close, rebase onto main. See [[whitefoot-parallelism-doctrine]].

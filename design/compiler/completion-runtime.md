Decision: A completion record belongs to the ordinary linked call's native activation and is found by address, with no global slot token or source capacity wait, because the call already owns the live request and borrowed backing until return, instead of a fixed global slot pool with generation-checked tokens and a refusal the submitter loops on.

Decision: A ring that is absent or refused at startup falls back silently to the blocking-helper adapter, while a target failure after an operation has been accepted ends the process as a fail-stop with no fallback and no writer-visible `IoError`, because before acceptance both engines answer the same request identically, whereas after acceptance ownership of the target operation has transferred, so falling back would duplicate it and ignoring the error would strand it, instead of degrading a post-acceptance failure to the adapter or to an `IoError`.

Decision: The native completion record remains 160 bytes, keeping larger operation-specific requests on the helper adapter, because enlarging a common record for one operation taxes every pending request, instead of a larger shared record or an additional accept-owned buffer.

Decision: The helper pool starts at zero where a native ring is ready and grows on demand, on a measured wait of about 20 microseconds sampled one execution in sixteen and unconditionally for a peer-bound request, up to the machine's CPU count, because an unset helper count pinned one helper, the worst measured setting for a program with width, instead of a fixed helper count.

Decision: An opened descriptor's kind is read with one `fstat` on the reaping thread rather than a second ring operation linked to the open, because two ring round trips cost more than the open they wrap, measured at 152 against 116 milliseconds on the eight-wide many-file program, while the mode of a descriptor the kernel has produced is an inode read that cannot wait, instead of a linked `IORING_OP_STATX`.

Decision: Ordinary linked implementations call their required native engine through strong link references, because an absent engine cannot complete a submitted request, instead of a weak inline fallback for an unresolved native implementation.

Decision: Every TCP socket the runtime creates or accepts sets `TCP_NODELAY`, because with the option off io_uring ran at about 1,562 requests per second with a 41 millisecond p99 at 64 peers and 64 KiB and gained 11.8 to 15 times paired throughput with it on, instead of the host's default packet policy.

Decision: The runtime's tuning constants are measured and recorded beside their definitions, a reap budget of 64 completions per lock, a join spin of 10 microseconds before parking, an io_uring submission depth of 64 with 2,048 completion entries, because the reap budget alone cut a 64-connection run from 720 thousand futex calls to 19 thousand and doubled its round-trip rate, instead of unmeasured defaults.

Decision: On Windows the completion port's wake records each native wait in an intrusive list under the runtime wait lock, publishes one packet per previously unnotified waiter, re-posts a polled packet only while a notified cohort is outstanding, and parks a newcomer on the runtime condition until that cohort has left the kernel, because a wake sized by the parked-scheduler count let a newer park take an older sleeper's packet, which stalled a scheduler job at its 20-minute limit and replayed as two wakes expected and none received, instead of one unaddressed packet per announced sleeper consumed unconditionally by whichever park receives it.

Rejected:
- A fixed global slot pool with generation-checked tokens and a capacity wait: rejected because the native call activation already owns the request until return and the pool only added refusals nothing could act on.

<!-- Adversarial review of research/investigations/io-model/DESIGN.md revision 1,
     2026-08-25, sol-ultra agent "blindspot-scout". Checked in as evidence behind revision 2;
     paths sanitized to repo-relative and <scratch-root> forms. Findings were
     re-verified by the lead before adoption; §3f's original disposition in the
     runtime and sweep reports was OVERRULED by constitution T3 (see DESIGN.md). -->

# I/O foundation 完整性审计

假设未来有两个单调时钟采样。下面是示意，不是当前可编译接口：

```wf
let t1 = clock_sample(clock: &'c clock);
let t2 = clock_sample(clock: &'c clock);
```

设计把 clock 归入普通 world read，并准备允许 `read/read` 重叠。[DESIGN §3b](</research/investigations/io-model/DESIGN.md:123>)。如果世界先处理第二个请求，程序可能得到 `t2 < t1`，这个结果不对应同一条单调时间线上的源码顺序执行。

这暴露了核心缺口：当前设计已经找到提交和完成的运行时形状，却还没有定义外部历史、动态 capability、进程生命周期和失败边界。结论是 10 项 `BLOCKING`、18 项 `STRUCTURAL`、4 项 `DISTANT`。

我没有把 §9 已登记的 cancellation、partial completion、timeout、backpressure、own-mode region representation 重新计数。时钟部分只讨论时钟域和事件排序；终止部分讨论进程死亡，不讨论 writer cancellation。设计已经提到的 capability alias、cross-region fence 和 10k server 限制也没有重复算作新发现。

## BLOCKING：任何 spec batch 之前必须回答

| ID | 具体问题 | 为什么会咬到 | 受压的现有设计 | 相容方向 |
|---|---|---|---|---|
| B1 | 每个 completion 是否都必须对应一个先前 submit？ | `SIGTERM`、设备热拔插、文件变化通知可能在没有预装 frame 时到达；子进程也可能在 `wait` 前退出。 | §1 把世界压成 submit/complete，§4 又假定 SQE 携带预分配 frame 指针。[DESIGN §1](</research/investigations/io-model/DESIGN.md:28>) [DESIGN §4](</research/investigations/io-model/DESIGN.md:162>) | 增加 compiler-owned event-source contract，规定 arm、pending、coalescing、drop 和 fatal 行为。writer 仍只调用普通操作，不获得异步 handler 或调度构造。 |
| B2 | world `reads` 的等价关系是什么？ | 两次 stdin、clock、entropy 或 listener 读取可能消费顺序、采样顺序或动态集合。`read/read` 不保证可交换。 | “moving world races sequential execution too”不足以证明 overlap；当前 spec 反而把推进文件 cursor 的读取明确视为状态写入。[DESIGN §3c](</research/investigations/io-model/DESIGN.md:136>) [SYS-11](</spec/kernel-spec.md:2602>) | 每个 family 声明外部历史和线性化规则。默认把消费、采样和 mint 操作写入隐藏 sequence region；只有规范证明幂等、可交换的 snapshot read 才使用共享 `reads`。 |
| B3 | capability 输出如何表达 fresh、alias、subregion 和多父来源？ | `accept` 动态产生 connection；`rename` 同时修改两个目录；两次 open 可能指向同一对象；stdout 和 stderr 是两个 owner，却可能重定向到同一 sink。[SYS-12](</spec/kernel-spec.md:2613>) | “每个 capability 一个 world-region identity”只覆盖一对一关系。即使 §9 的 own-mode 表示已经解决，也没有 fresh 或复合来源关系。 | system declaration 记录隐藏 origin graph：`fresh`、`aliases`、resource projections、multi-input footprints。没有证明的 separateness 一律保守冲突，writer 看不到这些身份。 |
| B4 | frame 和 completion 的精确状态机是什么？ | 迟到或重复 CQE 可能命中已复用 frame，产生 ABA；DMA 已写内存但 CPU 在 flag 后仍读到旧缓存；两个完成者可能同时发布结果。 | §4 只写了“frame pointer”和“completion is a flag flip”。[DESIGN §4](</research/investigations/io-model/DESIGN.md:162>) | 固定 `created → published → world-owned → completed → joined → reusable` 状态机，加入 generation-tagged operation ID、exactly-once terminal transition、release/acquire publication。target qualification 必须证明该协议。 |
| B5 | “完成”究竟完成了哪件事？ | 写完成时 buffer 可能已安全复用，但数据尚未对第三方可见，更未持久化；pipe 最后一个 writer 的隐式 release 会产生 EOF；connect 入队不等于连接建立。 | 设计把结果、loan 归还和 flag flip 合并为一个词；当前 spec 已经要求每个资源有 release policy，但 foundation 没有把 release 纳入 completion 协议。[SYS-5](</spec/kernel-spec.md:2434>) | 每个 semantic ID 明确 submission linearization、world visibility、loan-release、terminal outcome、durability 和 compiler-derived release。隐式 release 也走同一套 operation contract。 |
| B6 | world failure、程序缺陷、TCB exhaustion 和 target defect 如何分流？ | `ENOSPC`、`ECONNRESET` 应是 typed world outcomes；本地 allocator refusal 属于 resource death；错误状态下的 `EBADF` 应当不可表示；completion backend 不应意外泄漏 `WouldBlock` 或 `EINTR`。 | active spec 有 `IoError::ResourceExhausted`、`NoSpace` 等 world values，[SYS-7](</spec/kernel-spec.md:2506>)；0079 又准备把 TCB exhaustion 定义为第四类 fail-stop death。[0079 ERR-4 recipe](</docs/ongoing/0079-exhaustion-floor.md:894>) | 每个 operation row 固定 failure disposition：typed outcome、static obligation rejection、claim trap、TCB resource death、target/start failure。分类依据语义来源，不依据 host errno 名字。 |
| B7 | 进程死亡时，已经提交的外部工作和诊断记录怎么办？ | 异步 stderr write 已提交后另一 lane OOM。磁盘或终端可能在 resource record 之后继续写，甚至与记录交错。无需 cancellation API，这个冲突已经存在。 | 0079 recipe 要求 exhaustion 后不再产生程序外部效果，[0079 SCOPE-3 recipe](</docs/ongoing/0079-exhaustion-floor.md:793>)；当前 `TRAP-1` 却允许 already-started external work 保留自身语义。[TRAP-1](</spec/kernel-spec.md:2395>) | 定义 termination cut：cut 后不再 submit；cut 前已线性化的操作要么计为既成效果，要么由 arbiter 隔离并确认静止。trap/resource 记录使用与 writer stderr 串行化的控制通道。 |
| B8 | program kind 从世界获得哪些根 capability，又如何结束？ | 当前 `command` 没有 stdin；裸机没有 args、cwd、OS process status 或 OS teardown；service 还需要 listener、clock、entropy、credentials。 | spec 只有一个 `command main` 和四项输入。[FN-7](</spec/kernel-spec.md:1180>) 设计却同时声称 bare-metal contract 不依赖 OS。[DESIGN §4](</research/investigations/io-model/DESIGN.md:183>) | 先定义 program-kind-specific root capability tables 和 lifecycle contracts。`command.stdin` 应是显式 stateful capability；未来 firmware/service 使用不同闭表，仍保持一个 entry、无 ambient authority。 |
| B9 | Whitefoot OS 出现前，capability authenticity、alias 和 revocation 由谁保证？ | POSIX descriptor 可经 duplication、inheritance 或 redirection alias；revocation 时 kernel 仍可能持有 DMA buffer 和 frame；整数 descriptor 还会被重用。 | §7 只说 affine capability 使 reclaim tractable，[DESIGN §7](</research/investigations/io-model/DESIGN.md:230>)，但 affine 只限制 Whitefoot value，不限制 host 中的别名和 in-flight 引用。 | target qualification 必须声明保护机制。近期开销可选为进程隔离、generation-tagged handle table 或全局保守 alias group；revocation 先阻止新 submit，再 drain 或 quarantine late completions。 |
| B10 | 怎样写 compiler-independent I/O conformance？ | live DNS、网络和时钟会让 verdict 不稳定；permission 又永远不是 obligation，所以测试不能要求“确实 overlap”。 | PAR-1 只能用 annotation 加 compiler/runtime evidence，因为没有 source-to-verdict case。[0074 conformance boundary](</docs/done/0074-proof-derived-parallelism.md:94>) I/O 还增加了外部结果和顺序。 | 建立三层证据：scripted deterministic world 检查语义和多种 completion schedule；target qualification 检查真实 backend；性能测试只测 actualization，不作为语言 conformance。 |

## STRUCTURAL：会塑造设计，但可在 foundation 定界后逐族落地

| ID | 具体问题 | 为什么会咬到 | 受压点 | 相容方向 |
|---|---|---|---|---|
| S1 | 各类 signal 分别是什么？ | `SIGPIPE` 是 write outcome，`SIGCHLD` 是 child lifecycle，`SIGINT` 可能是事件或强制终止，`SIGSEGV/SIGBUS` 是 TCB/floor 问题。把它们统一成 interrupt completion 会混淆语义。 | 设计只说 ISR 不运行 writer code；当前 spec 仅具体处理 SIGPIPE。[SYS-12](</spec/kernel-spec.md:2628>) | 建立 signal classification table。raw handler 永不进入 writer；SIGPIPE 继续归操作结果，SIGCHLD 由 Child capability 吸收，fatal fault 留在 containment。 |
| S2 | `fork`、`exec`、spawn、wait 和 exit code 的所有权是什么？ | `fork` 会复制 affine Output、runtime locks 和 in-flight frames；in-place `exec` 会绕过 compiler-derived release；child 可能正常退出、被 signal 杀死或 resource-death。 | 当前 entry 只产生正常 `ExitStatus`，异常终止没有 status。[PROG-3](</spec/kernel-spec.md:1489>) | 不提供 writer `fork` 或 in-place `exec`。提供原子 `spawn`，显式移动 child inputs，返回 completion-required `Child`；`wait` 是 completion，并产生独立 `ChildOutcome`。 |
| S3 | pipe 两端、capability transfer 和跨程序顺序归谁所有？ | 两个 WF 程序相接时，producer 和 consumer 都不拥有 pipe 内部队列；最后一个 writer release 产生 EOF；多个 writer 的字节可能交错。 | per-program world regions 不能单独定义跨进程顺序。 | arbiter 拥有 pipe history，程序分别拥有 `PipeRx`、`PipeTx`。resource contract 固定 FIFO、EOF、atomic-message 边界；cap transfer 消耗 owner，失败 outcome 必须归还它。 |
| S4 | create、rename、unlink、delete 写的是哪个 region？ | `rename(A/x, B/y)` 同时改变两个目录 namespace，并可能改变目标对象 link state；删除名称后已打开 handle 仍可存活。 | 一 capability 一 region 无法表达多目录 namespace mutation。 | 引入 `DirectoryMut` 或 rights facet；create 写父目录并 mint object capability，rename 写源和目标 namespace，unlink 写父 namespace。每项 operation row 列出完整多 region footprint。 |
| S5 | file object、cursor、byte ranges 和 metadata 是否同一个 world region？ | 四块并行 `read_at` 不应因一个 cursor 被串行化；两个 cursor 又可能指向同一文件；并行写不同 byte ranges 的原子性依 target 而异。 | 当前 `ReadFile` 把 object 与 cursor 合成一个 stateful resource。[SYS-11](</spec/kernel-spec.md:2602>) | 区分 shared file object 与 affine cursor。positioned I/O 只有在 target 能证明 byte-range semantics 时获得细粒度 footprint，否则保守排序。 |
| S6 | mmap、shared memory 和 MMIO 能否成为普通 Whitefoot memory？ | 另一个进程改 shared page，或设备改 status register 后，普通 load 可能被优化器缓存或提升，直接破坏 T1 和 optimizer facts。 | “外部接触只经 system operation”在映射内存上失效；现有 storage 只给 gated FFI 预留 `foreign_shared`。[STOR-1](</spec/kernel-spec.md:638>) | 普通 borrow 永不指向 externally mutable storage。只先允许 snapshot/read-only 或 arbiter-proved exclusive mapping；shared/MMIO 通过 typed volatile/atomic operations 或 D17-checked representation lane。 |
| S7 | monotonic clock、wall clock、timer 和 deadline 属于哪些域？ | NTP 或人工校时会让 wall clock 回退；两个不同 clock 的 instant 不可比较；operation 和 deadline 同时完成时必须有确定 winner。 | §9 只把 timeout 命名为 completion，没有定义 clock identity 或 comparison law。 | 独立 `MonotonicClock` 与 `WallClock` capability。deadline 只接受同一 monotonic origin 的 opaque instant；race winner 由 arbiter linearization 决定，不由 worker schedule 决定。 |
| S8 | entropy 与 deterministic PRNG 如何分开？ | 两个 entropy draws 不可按 shared reads 自由交换；裸机可能没有合格 TRNG；record/replay 会把 seed 变成敏感资料。 | `external` 已把 random sequence 算作外部状态，但 foundation 完全未建模。[EFF-1](</spec/kernel-spec.md:1355>) | `Entropy` 是 explicit stateful capability；一次 seed acquisition 是 world operation。之后使用 owned local PRNG state，普通 ownership 负责顺序。无合格 entropy 的 target 在 qualification 阶段失败。 |
| S9 | bind、listen、connect、accept 的状态转移和 authority 是什么？ | accept 消耗 listener backlog 并动态 mint connection；bind 修改 network namespace；connect 可能需要返回失败后的 builder 状态。 | “network writes across connections”只处理已有连接，没有连接从何而来。[DESIGN §3c](</research/investigations/io-model/DESIGN.md:136>) | 使用 explicit `NetworkAuthority`、`Listener` 和 connection type states。accept 写 listener sequence region并返回 fresh connection origin；bind/listen/connect 的每个状态转移由 outcome type 完整表达。 |
| S10 | stream、full duplex、half-close 和 datagram 是否共享一个操作族？ | 单一 socket region 会不必要地串行化 read/write；允许全部 overlap 又可能打乱同方向 FIFO。datagram 还需要保持 message boundary、source address 和 truncation outcome。 | per-connection FIFO 过于粗略。 | stream 拆成关联的 Rx/Tx facets，每方向单独排序，half-close 是 consuming transition。datagram 使用独立 message outcome，绝不套用 byte-stream cursor contract。 |
| S11 | DNS 与 service discovery 的 authority、cache 和结果顺序是什么？ | resolver 会读取配置、网络、clock 和 cache；结果含多个地址、TTL，并可能在两次调用间变化。 | 设计只写了 network，没有 resolver capability 或 variable-result ownership。 | 显式 `Resolver` capability 固定配置和 authority；结果顺序、TTL 和缓存观察写进 family contract。没有 ambient libc resolver。 |
| S12 | variable-sized external results 的 backing 由谁拥有？ | DNS 地址集、peer identity、environment、certificate chain 都不能放进固定 inline outcome。当前 `HostString` 又只允许 argv-lifetime backing。 | system operation 当前声明“不分配”，新 producer 必须采用新 owned-backing type。[SYS-2](</spec/kernel-spec.md:2266>) [HOST-3](</spec/kernel-spec.md:2338>) | 统一选择 caller-provided buffers/cursors，或显式 owned backing resource。不得让 backend 隐式分配并把 lifetime 藏在 opaque result 中。 |
| S13 | writer 怎样看见 in-flight operation 和 overlap denial？ | 程序挂在 join 时，目前只能看见“没结束”，无法知道尚未 submit、世界持有、completion 丢失还是 alias 保守拒绝。 | compute 已有 `--par-ledger`，I/O 没有 sibling。 | 增加 compile-time `--io-ledger`，记录 site、origin graph、world footprints、grant/denial 和 join。另设 opt-in runtime trace，使用 generation ID，只记 metadata，独立 channel，关闭时零语义影响。 |
| S14 | deterministic record/replay 是否值得现在命名？ | `WF_WORKERS=0` 仍会读取实时文件、网络、clock 和 entropy，因此不是完整的 deterministic reproduction anchor。[DESIGN §5](</research/investigations/io-model/DESIGN.md:193>) | “sequential/overlapped 两个世界”混合了执行顺序与外部世界来源。 | 值得现在命名为 recorded-world backend，但不是第三套语言 lowering。执行轴是 sequential/overlapped，world-provider 轴是 live/recorded；replay 校验 submit trace，并重放结果、mint graph 和 completion order。 |
| S15 | 裸机 ISR priority、嵌套中断和无 allocator mailbox 如何满足 frame protocol？ | 高优先级 IRQ 可打断低优先级 enqueue；ISR 不能 malloc 或拿普通锁；设计同时称 bare metal 需要 executor，又称 `WF_WORKERS=0` 是 zero-runtime embedded build。 | [DESIGN bare metal](</research/investigations/io-model/DESIGN.md:183>) 与 [two worlds](</research/investigations/io-model/DESIGN.md:201>) 的术语冲突。 | target contract 固定 ISR-safe acknowledgement、priority rules、allocation-free intrusive mailbox 和 atomics。`WF_WORKERS=0` 应表示无 overlap/worker pool，不应表示没有最小 I/O driver substrate。 |
| S16 | DMA buffer 的 pinning、IOMMU 和 cache coherency 谁负责？ | device 完成 descriptor 后，如果 cache 尚未 invalidate，join 仍会读到旧 bytes；移动 descriptor 不应改变 DMA address；设备可能要求 alignment 或 contiguous pages。 | “buffer on loan”只解决 source ownership，没有解决物理地址与缓存可见性。 | 使用 `DmaBuffer` family 或 target-qualified hidden pin lease。completion 在 DMA sync 和 cache maintenance 完成后才 release-publish；普通 buffer 只有 target 能映射时才可借给设备。 |
| S17 | operation 是否保证最终 completion？ | peer 永远不发送是合法等待；设备拔除应返回 typed failure；lost interrupt 则是 TCB defect。若不区分，join 永久停住无法归因。 | scheduler 只说明如何等待，没有 progress contract。 | 每个 family 声明可能永久等待还是在环境条件下最终完成。device removal 强制完成所有 outstanding operations；lost completion 属于 target qualification/runtime defect。 |
| S18 | visibility、durability 和 crash consistency 怎样组合？ | file write success 后掉电可能丢数据；`fsync(file)` 后目录项仍可能未持久；atomic replacement 在 crash 中需要比 cross-region fence 更强的协议。 | 设计只把 fsync 当普通 fence；当前 Output 已承认 late writeback error 可丢失。[SYS-12](</spec/kernel-spec.md:2623>) | 分开 buffered、durable 和 atomic-replacement resource families。持久化 transition 使用 completion-required owner，并明确 file 与 directory durability，不让 release 暗示 commit。 |

## DISTANT：现在命名并给出重入条件

| ID | 具体问题 | 为什么会咬到 | 受压点 | 相容方向 |
|---|---|---|---|---|
| D1 | TLS 放在 system family 还是普通 library？ | handshake 同时需要 stream、entropy、wall time、trust store、credentials 和多轮协议状态；把它做成 socket flag 会隐藏全部 authority。 | 现有单 capability、单操作 completion 形状不足。 | 先停在名称层。等 stream facets、clock、entropy 和 owned bytes 落地后，优先测试普通 checked library；只把硬件密钥或 OS credential store 留作 capability-backed primitive。 |
| D2 | terminal、console mode 和 device control 如何避免通用 `ioctl` 逃生口？ | resize、raw mode、color、serial baud 和设备私有命令都不是普通 byte stream；通用 integer opcode 会绕过 semantic-ID 审核。 | spec 已明确 terminal control 是未声明的独立 capability。[SYS-12](</spec/kernel-spec.md:2630>) | 每个控制面建立 typed capability 和 closed semantic IDs。没有 generic ioctl、untyped property bag 或 raw descriptor。 |
| D3 | 跨程序 lock、lease 和 coordination 是 memory ownership 还是 I/O？ | 两个程序需要 leader lease 或 file lock；持有者死亡后必须由 arbiter 回收，普通 `&uniq` 无法表达外部竞争者。 | local ownership 不覆盖另一程序。 | 把 acquire 视为 world completion，成功 mint affine lease capability；release、expiry、revocation 和 owner death 由 arbiter contract 定义，不暴露共享 mutex memory。 |
| D4 | `sendfile`、`splice`、storage-to-NIC 等 zero-copy operation 如何表达？ | 操作不借普通 buffer，却同时读取 file object、推进 offset 并写 socket；这是 P0 很可能需要的真实路径。 | buffer-loan 不是所有数据移动的共同表示。 | 待 B3 多 region footprints 落地后，增加双 capability semantic ID：精确声明源读取、cursor mutation、目标写入和 completion point，不新增 writer scheduling syntax。 |

## 最低闭包

首个 I/O spec batch 至少要为每个 semantic ID 固定七项 compiler-owned 数据：

1. authority 输入和 capability mint/alias 关系；
2. world footprint 与可交换性；
3. submit、线性化、loan release 和 completion 点；
4. typed outcome、resource death 与 TCB defect 的分界；
5. progress 和 compiler-derived release policy；
6. target qualification 所需的保护、内存顺序和生命周期保证；
7. ledger/conformance 使用的稳定 operation identity。

这些记录都不需要 `async`、task、线程、优先级或 queue depth 进入语言。writer 仍写顺序程序；compiler 只在上述事实足以证明时获得 overlap permission。

本次为纯只读审计，分支仍为 `io/model`，没有修改任何文件。

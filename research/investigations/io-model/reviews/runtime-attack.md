<!-- Adversarial review of research/investigations/io-model/DESIGN.md revision 1,
     2026-08-25, sol-ultra agent "runtime-attacker". Checked in as evidence behind revision 2;
     paths sanitized to repo-relative and <scratch-root> forms. Findings were
     re-verified by the lead before adoption; §3f's original disposition in the
     runtime and sweep reports was OVERRULED by constitution T3 (see DESIGN.md). -->

# Whitefoot I/O 并发运行时对抗审计

完成已经就绪，所有 lane 还能睡死吗？按 §4 当前写出的状态机会。lane 在这里指一条执行 Whitefoot 代码并拥有 deque 和 I/O 完成队列的调度线程。

```text
L0、L1 都在 join(Ti)，deque 为空

1. 检查 Ti：PENDING
2. pop / steal：没有计算工作
3. 内核投递 Ti 的 CQE
4. reap CQE：Ti = DONE
5. 直接进入 io_uring_enter(min_complete = 1)
6. 此后没有第二个 CQE

结果：两个 Ti 都已完成，两个 continuation 都可继续，
但 L0、L1 都在等待下一次完成，永久睡眠。
```

[DESIGN.md §4](/research/investigations/io-model/DESIGN.md:148) 只列出“reap CQ → park”，没有规定“处理过任何 CQE 就必须回到调度循环”。这不是 `io_uring` 的缺陷，而是用户态状态机少了一条边。

总判定：§4 的架构方向可以保留，当前协议不能进入实现；§5 的 `W=0/W=1` 分界意图可以保留，但“现有拒绝路径足够”和“单 lane 仍按源码顺序执行”都被现有代码直接否定。

本报告基于 `io/model` 的 `7ef03f8e`，全程只读，未构建、未运行、未修改工作区。

## 1. `io_uring` 唤醒与永久休眠

### 1.1 所有唤醒来源

| 来源 | 到达形式 | 必须建立的保证 |
|---|---|---|
| 计算 frame 发布 | eventfd 对应的 CQE，或定向 ring 消息 | deque 的 release 发布先于通知；通知只表示“重新扫描”，不表示一个任务 |
| I/O 完成、错误、短完成 | 提交该操作的 ring 上的 CQE | CQE 处理完必须重新检查 join target |
| 取消、超时 | 取消请求 CQE 加原操作的终态 CQE | 取消请求本身不能释放 frame 或 buffer |
| trap、资源死亡、进程关闭 | 信号或进程终止 | hosted 目标直接结束进程，不依赖调度器唤醒；bare metal 另需 quiesce |
| `EINTR`、CQ overflow、ring teardown | 控制结果 | 不是可执行工作，必须显式处理，不能当成普通空队列 |

### 1.2 `eventfd registered in each ring` 的方向写反了

`io_uring_register_eventfd` 的语义是：ring 出现 CQE 时，内核增加 eventfd。它不能把用户向 eventfd 的写入变成 ring 的 CQE。[liburing 手册](https://www.man7.org/linux/man-pages/man3/io_uring_register_eventfd.3.html)明确给出了这个方向。

若要让计算发布成为 CQE，每个 ring 必须提交监听 eventfd 的 `IORING_OP_POLL_ADD`。该操作默认是 one-shot，完成一次后必须重提；multishot 也必须检查 `IORING_CQE_F_MORE`，终止后重新布防。[io_uring_enter(2)](https://www.man7.org/linux/man-pages/man2/io_uring_enter.2.html)记录了这些 poll 规则。

按“registered”的字面实现，trace 是：

```text
publisher: deque.push(F); eventfd_write(E, 1)
parked lane: io_uring_enter(..., min_complete=1)

E 增加，但 ring 没有 CQE，lane 不醒。
```

当前结构化计算模型中，发布者通常还能在自己的 join 执行 `F`，所以这条 trace 未必单独造成全池死锁；它确定会丢掉计算唤醒，并会在未来出现外部生产者或 continuation 时成为全池死锁。

### 1.3 eventfd 合并不是计数协议

普通 eventfd 的一次读会取走整个计数并清零；`EFD_SEMAPHORE` 才是每次减一。[eventfd(2)](https://man7.org/linux/man-pages/man2/eventfd.2.html)给出了这一区别。

所以多个 publish 合成一次通知本身没有问题，但只能满足以下协议：

```text
收到一次通知
    -> 扫描所有工作源，直到确认静止
    -> 重新布防
    -> 公告自己将休眠
    -> 最后再扫描一次
    -> park
```

不能把 eventfd 值解释成 frame 数，也不能由一个 lane 清零后只执行一个 frame。若要精确唤醒多个 lane，应使用现有 `wf__par_idle` 的定向选择思路配合每-lane 通知，或者明确接受“一次唤醒一个 drain-to-quiescence worker”。

### 1.4 “CQ 空到 enter”这个窄窗口本身安全

若 poll 已经布防，CQE 在最后一次空检查之后到达，`io_uring_enter(..., min_complete=1)` 会观察到已经存在的 CQE并返回；无需额外 condition variable。[io_uring_enter(2)](https://www.man7.org/linux/man-pages/man2/io_uring_enter.2.html)保证等待的是 CQ 中可用事件。

真正危险的是三处用户态状态：

1. reap 得到 CQE、翻转完成 flag 后仍然 park；
2. one-shot eventfd poll 已消费但未重提；
3. CQE 投递到 A 的 ring，而真正等待它的是 B。

### 1.5 被偷 frame 的 ring 归属必须独立定义

现有 compute slot 用 `slot->home` 指向发布者的 lane。[par_runtime.c](/compiler/src/backend/par_runtime.c:183) 的 slot 可以被其他 lane 执行。

如果 I/O ring 也按 frame 的 `home` 选择，会出现：

```text
A 发布 compute frame C，B 偷走 C
B 在 C 内提交 I/O，但 SQE 进入 A 的 ring
B 在自己的 ring 上 join 并 park
A 收到 CQE、翻转 B 的 target flag，然后重新 park
B 没有收到自己的 CQE，也没有跨-ring通知

target 已 DONE，A、B 都睡眠。
```

提交 ring 必须取自“执行提交的 lane”，join 必须 park 在同一个 ring；或者完成处理器必须向实际 waiter 的 ring 定向投递唤醒。不能复用 compute slot 的 `home` 含义。

### 1.6 “自己的 deque 存着 dependent continuation”与 no-continuations 冲突

在当前设计里，依赖 continuation 应当是 join 下方尚未返回的 C 栈，不应位于 deque。若实现把 completion 转成 deque continuation：

- 它已经不再是 “no continuations”；
- waiter 或内核线程会成为 Chase-Lev deque 的第二个 producer；
- 现有单-owner push 假设失效；
- 需要新的跨线程入队与唤醒协议。

应把“completion 只写结果和终态，绝不向 compute deque 发布 continuation”列为显式不变量。

## 2. join-as-worker 的延迟、栈和饥饿

### 2.1 完成已就绪，join 却执行任意长的无关任务

当前 `wf__par_wait` 在检查 target 后，先 pop 自己的 deque，再任意 steal，然后把整个 frame 执行完。[par_runtime.c](/compiler/src/backend/par_runtime.c:455)

```text
join 检查 T：尚未完成
T 的 CQE 随即到达
join steal 到 U
U 运行十秒，或永不返回
T 一直处于 ready，join 无法继续
```

I/O 会放大这个问题，因为 CQE 必须由用户态 reap 才能翻转 flag。§4 的 idle 顺序把 CQ 放在计算工作之后，持续存在的 compute backlog 可以让 I/O 完成永远不被 reap。

最少需要以下优先级：

1. drain 自己的 CQ；
2. 若当前 join target 已完成，立即返回；
3. 执行同一结构化 window 内也必须完成的 compute frame；
4. 最多执行一个无关 steal，再回到第一步。

即使这样，无抢占的 `U` 仍可任意长。若要保住源程序的 liveness，I/O join 不应执行任意无关 frame，只能帮助同一个 window 内、退出前本来就必须 join 的 frame。

### 2.2 join 内 steal 会递归增长 lane 栈

实际调用链是：

```text
A at join
  wf__par_execute(B)
    B at join
      wf__par_execute(C)
        C at join
          ...
```

前一层 join 和用户 frame 都没有退出。当前运行时注释仍声称 stolen call 从 lane 栈底开始，[par_runtime.c](/compiler/src/backend/par_runtime.c:287)；0079 的后续审计已经明确认定这个前提为假，[0079-exhaustion-floor.md](/docs/ongoing/0079-exhaustion-floor.md:387)。

当前总 slot 数给动态嵌套一个粗糙上限，最多 `64 lanes × 64 slots`，所以不是数学意义上的无限；但它不受源码调用图或字节栈预算约束，任一被偷 frame 又能进行深递归。lane 在接近 1 GiB 栈底时只需再执行一个小 frame，就会产生 `{"resource":"stack"}`。

`--stack-ledger` 也看不到这条调度边：它明确排除 runtime translation unit，并看不到 thunk 间接调用。[stack_ledger.rs](/compiler/src/backend/stack_ledger.rs:22)

最小修补是加入 per-lane `help_depth`：

- 最外层 join 可以执行一个 helper frame；
- helper 内的 join 只 drain completion 或 park，不继续 steal；
- ledger 明示这一个额外 scheduler 层的保留量。

若不限制，设计必须撤掉“固定 lane 栈让 stealing 只增加 headroom”之类的论证，并把这种资源死亡当作明确代价。

### 2.3 两个工作源需要公平规则

只写“one scheduler, two work sources”还没有调度规则。至少要规定：

- join target completion 高于其他工作；
- CQ 每执行有限个 compute frame 后必须 drain；
- CQ flood 也不能永久饿死 compute deque；
- one-shot/multishot completion 每批处理有界；
- 非终止 frame 无法提供延迟保证，这个限制必须公开。

## 3. kqueue、waiter 和 blocking disk pool

### 3.1 buffer loan 必须跨线程延长到终态

危险 trace：

```text
lane 提交 read(buf: &uniq ...)
submit 返回
lane 离开 window，移动、释放或复用 buf
disk worker 稍后向旧地址写入
```

对输入 buffer 需要跨线程 exclusive loan；对输出 buffer 需要 shared/read loan。loan 必须从提交线性化点保持到原操作的终态完成，期间：

- 地址不可移动；
- 内存不可释放或复用；
- lane 不可读写 exclusive buffer；
- frame 不可回到 free list；
- waiter/disk worker 必须具备访问该存储的目标级安全资格。

这正是 io_uring 对 buffer 的实际要求：读写 buffer 必须保持有效直到 completion。[io_uring(7)](https://man7.org/linux/man-pages/man7/io_uring.7.html)

### 3.2 waiter 可以存在，但不能阻塞在普通 I/O 或 mailbox

单 waiter 只有在它始终执行短、非阻塞动作时才安全：

- kqueue readiness；
- submission 分发；
- completion 入 mailbox；
- wake。

普通文件操作必须放入 disk pool。pool 还需要固定深度、排队上限、公平策略和取消保留容量。若 waiter 在满 mailbox 上阻塞，而且 wake 发生在成功入队之后，就会形成：

```text
mailbox 满
waiter 阻塞于 enqueue，尚未 wake
所有 lane 已 park，只有它们能 drain mailbox
```

最稳妥的形状是 frame 自带预分配 completion node，使 mailbox 容量覆盖全部允许的 in-flight frames，waiter 永不等待内存或队列空间。

### 3.3 MPSC mailbox 的最低内存模型

`db543775` 本身不是 lane-count 修复；修复是它祖先中的 `39195bca`。该提交把 `wf__par_lane_count` 的七处并发访问全部改成 relaxed atomic，因为“写入相同或无害数值”仍然是 C data race。

mailbox 需要比 lane count 更强的保证：

1. producer 完整写入 result、status 和 node 后，使用 release 发布；
2. consumer 使用 acquire 取出，之后才读 payload；
3. head、tail、next 的所有并发访问遵守所选 MPSC 算法，不能混入 plain access；
4. “队列空”必须能区分真正为空与 producer 已交换 tail、尚未链接 next；
5. enqueue 的线性化点先于 wake；
6. consumer 先公告休眠，再 acquire 重查 mailbox；
7. 每个操作只有一个终态，frame 在 consumer 确认前不得复用；
8. cancel CQE 和原操作 CQE 必须用 generation 或未复用 frame 消除 ABA；
9. waiter、disk workers 和 ISR 都不能同时写同一 completion 字段。

典型错误 trace 是：

```text
P: atomic_exchange(tail, node)
P: 暂停，尚未写 prev->next

C: 看到 head->next == NULL
C: 判断空并 park

P: 写 prev->next
P: 若 wake 已提前发送或未使用 posted 握手，C 永不醒
```

现有 condition-variable 路径用 `posted` 和 idle bit 专门封住这个窗口，[par_runtime.c](/compiler/src/backend/par_runtime.c:303)。mailbox 不能只写“MPSC”便假定同一性质自动成立。

## 4. `WF_WORKERS=1` 与实际 lowering

### 4.1 当前 runtime 确实会让 claim 返回 `NULL`

当前解析代码把所有 `< 2` 的值映射为 0：

```c
if (end == setting || *end != '\0' || requested < 2) {
    return 0;
}
```

而 bootstrap query 是：

```c
int wf__par_pool_active(void) {
    return wf__par_requested_lanes() >= 2;
}
```

见 [par_runtime.c](/compiler/src/backend/par_runtime.c:611) 和 [pool query](/compiler/src/backend/par_runtime.c:798)。

所以当前 `W=1`：

- `wf__par_pool_active()` 返回 false；
- bootstrap 选择 sequential clone；
-若 overlapped clone 被强行选择，`wf__par_claim()` 会因没有 lane 返回 `NULL`。

仅修改 query 可以得到“overlapped clone + 所有 compute claim 拒绝”，但此时没有准备 lane 0，也没有 per-lane ring。若把 `wf__par_start` 改成真的初始化一个 lane，现有 `wf__par_claim` 又会拿到 free slot，不再自拒绝。

设计必须拆开三个概念：

- 是否选择 overlapped world；
- 有几条 Whitefoot compute execution lanes；
- I/O ring/backend 是否已初始化。

`wf__par_pool_active` 应改名或改合同为 mode selector。`W=1` 应表示一条程序执行 lane、零条 stealing worker、一个 I/O scheduler endpoint；compute claim 需要显式 `compute_lanes < 2` 拒绝。

### 4.2 `NULL` fallback 不在原语句位置执行

现有 emitter 在 hand-out 处只做 claim 和条件分支；`NULL` 时把调用推迟到最后一个成员之后的 join。[parallel.rs](/compiler/src/backend/emitter/parallel.rs:385) 与 [join lowering](/compiler/src/backend/emitter/parallel.rs:534) 形成的控制流是：

```text
s1: claim == NULL，记录“稍后 inline”
s2: 现在执行最后一个成员
join(s1): 现在才 inline 执行 s1
```

因此 [DESIGN §5](/research/investigations/io-model/DESIGN.md:204) 所说“program statements execute in source order”是假的，即使只有一个线程。

### 4.3 三类可观察变化

1. **trap record 确定改变。** 现有测试中 `left` 是首成员，`right` 是最后成员。当前 `W=0/1` 都选 sequential clone，所以 `left` 的 claim record 固定获胜。[trap_latch.rs](/compiler/src/backend/tests/trap_latch.rs:205)  
   若 `W=1` 选择 overlapped clone且 claim 全拒绝，`right` 先运行并 trap，记录变成 `right_index_in_range`。当前 [PAR-1](/spec/kernel-spec.md:2016) 允许错误执行由 schedule 选择 claim，但 `W=1` 不再是源码顺序的复现路径；只剩 `W=0`。

2. **published bytes 可以改变。** 假设首成员含失败 claim，最后成员提交独立 output：

   ```text
   W=0: 首成员 trap，output 从未提交
   W=1 overlapped: output 先提交，随后首成员 inline 并 trap
   ```

   bytes 甚至可能在 trap record 之后由已提交 I/O 发布。v0.36 当前之所以承诺错误执行没有外部 effect，是因为 overlap window 明令禁止 system operation，[kernel-spec.md](/spec/kernel-spec.md:2010)。I/O 设计必须重新裁决：最保守的规则是任何可能提交外部工作的 overlap window 都必须 `traps`-free。

3. **stack resource record 可以改变。** 0079 已测得同一递归的 overlapped clone 为 48 B/level，sequential clone 为 16 B/level。[0079-exhaustion-floor.md](/docs/ongoing/0079-exhaustion-floor.md:475)  
   `W=1` 改选 overlapped clone后，原本完成的程序可能改为 `{"resource":"stack"}`。这在当前规范中属于允许变化的资源条件，但确实是进程结果和 stderr 的变化。

正确的 `W=1` 不应新建 compute pthread。现有 worker 数含调用线程本身；lane 0 应复用 `wf__floor_run` 已提供的 1 GiB entry stack。waiter 和 disk pool 是 TCB 线程，不应执行 writer code。

## 5. in-flight buffer、floor 和 abort

[wf_floor.c](/compiler/src/backend/wf_floor.c:150) 的 SIGSEGV/SIGBUS handler 选中唯一记录后直接 `abort()`；其他 lane 可以仍在 `io_uring_enter`，kernel 或设备也可以继续持有 buffer。

### Hosted Linux 必须由 target qualification 保证

- ring 关闭或任务退出会取消 pending requests；
- 已交给硬件、不能取消的请求继续持有必要引用；
- CQ、callback 和 pinned page 的生命周期不得落到已销毁地址空间之后；
- 用户态不需要在 signal handler 中 drain CQ；
- 外部写入可能完成，且不承诺回滚。

Linux 的 ring shutdown 会自动取消 pending requests，但已经交给硬件的操作通常不可取消。[io_uring cancellation](https://man7.org/linux/man-pages/man7/io_uring_cancelation.7.html) 当前内核退出路径也在拆除地址空间前调用 io_uring cancellation，[Linux `exit.c`](https://github.com/torvalds/linux/blob/master/kernel/exit.c)。这是 Linux 后端资格，不是语言本身的自然定律。

### Bare metal 不能借用“进程退出会回收”

DMA 设备可能在 CPU 宣布 abort 后继续写物理内存。若这些页立即交给另一个实例，就重新引入内存破坏。arbiter 必须做到以下之一：

- 停止新 doorbell，取消或 reset 设备，等待 ownership handback；
- 用 IOMMU 撤销访问，但要遵守设备 drain/fence；
- 把 descriptor 和 buffer 隔离，直到不可取消的 DMA 完成；
- 若 whole-machine abort 等于停机或硬 reset，明确声明内存不会在 reset 前复用。

这些都是 TCB teardown，不是语言 cleanup，所以不与 [EFF-4](/spec/kernel-spec.md:1429) 冲突。

### v0.36 没有覆盖完整问题

[TRAP-1](/spec/kernel-spec.md:2395) 只说：

- hosted OS teardown 回收 process-local objects；
- 已开始的外部工作保留其 family 自己规定的语义；
- 当前同步路径不需要 pending-operation transfer。

它没有定义异步 family、buffer pin/quarantine、bare-metal DMA 或“提交后、record 后完成”的顺序。[PAR-1 的 what-survives 句](/spec/kernel-spec.md:2026) 更明确依赖窗口没有任何 external effect，不能直接泛化到 I/O。

## 6. window exit 的最小 sound cancellation 规则

最小规则应当是：

> 每条正常或 recoverable 的 window 退出边，必须先观察该 window 所有已提交操作的终态。取消请求不是终态。frame、capability 与所有 borrowed buffer 在原操作到达终态之前保持存活且不可复用。trap 不执行语言取消；目标 TCB 负责 quiesce 或 quarantine。

对应状态机：

```text
CLAIMED
  -> SUBMITTED                 // loan 生效
  -> CANCEL_REQUESTED          // loan 仍生效
  -> COMPLETED(result)         // 原操作终态
     或 CANCELLED_BEFORE_EFFECT
  -> CONSUMED
  -> FREE
```

io_uring 的 cancel request 和原操作各有 CQE，次序没有保证；`-ENOENT` 或 `-EALREADY` 都不能证明原操作已安全消失，而且部分硬件操作无法取消。[cancellation manual](https://man7.org/linux/man-pages/man7/io_uring_cancelation.7.html)

因此 §9 的“stop waiting”不够。若 window 真要先退出，runtime 只能接管 buffer 所有权并保留隐藏 reaper；borrowed stack buffer 做不到这一点。与 no-continuations 最相容的首版选择是：不提供早退取消，维持现有 lowering 的“最后成员之后 join 全部，再允许任何 exit edge”。[builder.rs](/compiler/src/lowering/builder.rs:582) 已经具备这个结构。

## 7. 哪些部分可以保留

| 设计部分 | 判定 |
|---|---|
| submit/complete 是语言以下的共同模型 | 保留 |
| 一套 scheduler 观察 compute deque 和 completion source | 保留，但必须写成完整状态机 |
| per-lane ring | 保留，但 ring affinity 必须取执行 lane，且 completion 必须唤醒实际 waiter |
| join 时帮助执行工作 | 限制保留，只帮助同 window 或设置有界 `help_depth` |
| no writer-visible futures / async / continuation | 首版可保留，条件是所有正常退出先终态 join |
| kqueue waiter + disk pool | 条件保留，需要跨线程 loan、MPSC 内存模型和有界 backpressure |
| `WF_WORKERS=0` sequential anchor | 完整保留 |
| `WF_WORKERS=1` 让 I/O overlap | 方向保留 |
| `W=1` 通过现有 claim refusal 自动保持源码顺序 | 不成立 |
| “no write to a world region after trap record” | 对 in-flight I/O 不成立，必须改成 family-defined already-started semantics或禁止 trap/I/O overlap |

## 8. 按优先级排列的必要修订

1. **P0：规定 in-flight loan、终态和 abort teardown。**  
   失败 trace：window 退出或 bare-metal instance 回收 buffer，DMA 随后写入已复用内存。此项直接关系到 Whitefoot 的内存安全承诺。

2. **P0：写出完整 park/wake 状态机，修正 eventfd 方向。**  
   失败 trace：reap target CQE、置 `DONE`、继续 `io_uring_enter`，所有 lane 等待不存在的第二个 CQE。任何 CQE 或 wake hint 被处理后都必须回到调度循环。

3. **P0：裁决 trap 与外部提交能否重叠。**  
   失败 trace：后一个 I/O 先提交，前一个 delayed compute fallback 随后 trap；`W=0` 发布零字节，`W=1` 可发布字节，甚至在 record 后完成。建议首版让所有 I/O overlap window 的完整 call closure 排除 `traps`。

4. **P0：拆分 W=1 的 mode、compute lane 和 I/O backend 状态。**  
   失败 trace：只改 bootstrap query 时 claim 会拒绝但 ring 不存在；初始化 lane 0 后 claim 又会成功。现有 emitter 还会把 refused 首成员移到最后成员之后。必须重命名 query 合同并显式拒绝 compute hand-out。

5. **P1：限制 join helper 的范围和栈嵌套。**  
   失败 trace：深栈上的 join 执行 B，B 的 join 再执行 C，floor 在本来能够等待完成的程序中触发。加入 `help_depth` 或同-window 标签，并把调度层纳入 stack ledger。

6. **P1：给 CQ 和 compute frame 明确公平性。**  
   失败 trace：持续 compute backlog 使 lane 永远不 reap 已到达的 target CQE；反方向的 CQ flood 也可饿死 compute。规定 target-first、有限 batch 和每 frame 后重新检查。

7. **P1：固定 waiter/mailbox/disk-pool 合同。**  
   失败 trace：producer 更新 tail 后尚未链接 node，consumer 判断为空并 park；或 waiter 阻塞于满 mailbox，lane 必须醒来 drain 却尚未收到 wake。需要 release/acquire、exactly-once terminality、预分配节点、取消保留容量和 per-lane 公平分发。

最终 verdict：§4 的“一个 scheduler、两个工作源”和 §5 的“`W=1` 启用世界时钟”值得保留；字面 wake 机制、任意 steal 的 join、现有拒绝路径、单-lane 源码顺序论证，以及 v0.36 已足够覆盖 abort 的判断都不能保留。当前设计应视为架构草图，不是可安全实现的运行时协议。

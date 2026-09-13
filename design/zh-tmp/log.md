# 设计树变更日志

按最新优先排列。每一次被批准的树变更对应一条条目：一个带日期的标题、一行 `Nodes:`
列出每一个发生变更的节点，以及一段 `Summary:`；格式由 `skill/SKILL.md` 规定。

## 2026-09-13 记录各项审计的裁定并将规范修订到 v0.54

Nodes: compiler, compiler/completion-runtime, compiler/target-qualification, compiler/parallel-lowering, compiler/parallel-lowering/two-worlds, compiler/parallel-lowering/parallel-runtime, compiler/resource-exhaustion-floor, language/data-model, language/parallelism/permission-judgment, language/system-interface

Summary: `design/recall-tmp/audit/` 下的十一份模块审计，列出了代码里已经体现、却还没有对应节点的每一处选择，所有者对每一处都做了裁定；`design/recall-tmp/triage.md` 保存着这些裁定。新增两个编译器节点：completion-runtime，包含记录存于帧内的契约、启动期回退与接受之后 fail-stop 之间的不对称处理、160 字节的记录、按需增长的辅助线程、`fstat` 方式的 open 类型判定、硬性的链接要求、每个套接字都设置的 `TCP_NODELAY`、经测量的调优常数，以及暂定的两槽位分阶段循环流水线；以及 target-qualification，包含那道人工核对的版本检查。编译器根节点记录了所链接的运行时是人工审查过的 C 代码、位于 safe-Rust 保证之外。parallel-lowering 新增了许可账本的提示策略；two-worlds 新增了先认领 lane 再建帧的顺序；parallel-runtime 新增了最新优先的 join 顺序以及一吉字节的 lane 栈；resource-exhaustion-floor 新增了栈账本在代码生成之后才测量这一点，以及描述符地板（descriptor floor）。data-model 去掉了 array-of-structs 那条理由曾经引用过的 copy 结构体层级——规范和编译器都没有这个东西——并记录了名义类型身份里的区域轴。permission-judgment 承认一次判别对象调用也是窗口成员，规范也修订到 v0.54，在 [PAR-1] 里写明这一点，并在 [ENT-3.S10] 里为编译器已经推导出的整个 [SYS-8] 系列命名；退出使用的 v0.53 字节被归档。system-interface 记录了所有者的裁定：一个资源的状态由它的类型在 API 边界上承载，不会从被调用者的函数体推导出结果状态的来源，而现有的 `result_state_origin.rs` 与此相矛盾，`docs/todo.md` 现在跟踪这一点。这次修订的选择依据：[PAR-1] 遵循所有者"性能优先"的规则，因为收窄编译器会丢掉一次真实存在的重叠；[ENT-3.S10] 遵循 [SYS-8] 自身的文字，它本来就已经把这八个操作当作一个整体系列对待。
> 通俗解释：这一批改动，是把十一份代码审计里发现的、代码已经在做但设计树里没写明的做法，一条条过了一遍，由所有者拍板该怎么处理，再把结果补进编译器树和语言树：新增了两个编译器节点，分别讲清楚异步 I/O 完成机制和目标平台版本核对机制背后的取舍；在几个已有的并行相关节点里，补上了运行时到底是怎么实现调度顺序、栈大小、join 顺序这些细节的决定；去掉了数据模型里一条说得比实际存在的机制还多的话；并且把语言规范升级到了 v0.54 版本，明确了两处编译器一直以来就是这样做、但规范原来没写清楚的规则。

## 2026-09-12 合并 main 的计算运行时工作并删除清单开关

Nodes: compiler, compiler/parallel-lowering, compiler/parallel-lowering/two-worlds, compiler/parallel-lowering/parallel-runtime

Summary: 本分支开始之后，`main` 合入了计算记分板、递归预算克隆族、等待路径和切分粒度的工作、空闲窗口以及计算回归检查；这次合并保持编译器 README 已退役的状态，所以它新增的段落被搬到这里。two-worlds 用携带预算的克隆族及其默认的运行时推导预算取代 `N` 层递归前沿，`--par-recursive-frontier auto|N|off` 是对这个起始值的唯一控制，顺序拒绝的决策单独保留，`N` 层前沿记为被否决。parallel-lowering 从默认关闭的决策里去掉递归前沿，并记录被否决的对齐标志。parallel-runtime 新增以时间计的空闲窗口（1,024 轮上限作为它的分辨率）、150,000 的切分粒度和 16 的上限，以及为托管运行器节奏损失改唤醒路径的否决。编译器根记录所有者当天裁定的两个否决项：在这个研究编译器内部做加固和重复验证，以及放在编译期开关后面的已取代系统清单状态，后者在同一改动里从解析器中删除。main 一侧的条目从 `mcts_mem/whitefoot/parallelism.md` 恢复。
> 通俗解释：main 上最近一批并行运行时的工作（递归预算、空闲窗口、切分粒度等）合进了本分支，相应的决策补进了编译器树；同时按所有者的意见，加上了"不在这个实验编译器上做纵深防御"和"删掉旧接口清单开关"两条否决。

## 2026-09-12 退休 derivation ledger

Nodes: language

Summary: 规范修订为 v0.53：删除要求在 `spec/derivation/derivation-ledger.md` 维护规则到依据索引的 [META-6]，并归档 v0.52 的原始字节。账本及其 v0.2 前身删除，与它们同目录的 Featherweight-Rust 对照备忘录移到 `research/notes`。`whitefoot-spec` 门禁保留身份、规则编号唯一、交叉引用可解析的检查，去掉索引覆盖检查；META-6 的一致性清单行随规则一起删除。原先指向索引的指南改为指向设计树，language 树根记录这一裁定。选择依据：所有者裁定规范和设计树就是全部记录、二者必须自洽，第三份没人消费的记录只会漂移。
> 通俗解释：spec 以前强制要求一本"每条规则的来历账本"，现在理由都在树里，账本删了，spec 版本号升到 0.53，检查 spec 的小程序只保留真正有用的那几项检查。

## 2026-09-12 与所有者一起审阅 system-interface 子树

Nodes: language/system-interface, language/system-interface/declaration-home, language/system-interface/directory-enumeration, language/system-interface/program-entry-form, language/checks-and-proofs/requires-entry-contract

Summary: system-interface 树根的所有权决策应所有者要求加强：对语言来说每个系统对象都是普通的被拥有对象，不为表达系统状态发明任何语言机制，一个缺少某种能力的系统对象要重新设计成更好的所有权关系（比如一个发放被拥有资源的工厂），绝不用内部可变性或新特性打补丁，因为 agent 屡次试图用语言能力去修补接口的形状，而为一个接口添加的机制会成为所有权模型上的永久漏洞；它的四条否决项保留。效果行那条删去（effects 已经拥有它），io_uring 那条否决项删去（那是运行时测量，不是语言决策）。program-entry-form 的决策与 requires-entry-contract 携带的入口决策合并为这里的一条：唯一的、不带契约、不可调用的 `command fn main`，program-entry-form 连同其复述的否决项删除，requires-entry-contract 删去那条决策及其包装器否决项。declaration-home 的决策行缩短，三条否决项各自保留理由。directory-enumeration 删去复述第一条的否决项。至此 language 树审阅完毕。
> 通俗解释：这一组最重要的改动是把"系统对象就是普通对象、走所有权、不许为它发明语言机制"写死，专门堵 agent 以前反复犯的那种错。其余是去重、合并入口相关的两条、缩短一句过长的话。语言树到此全部过完。

## 2026-09-12 与所有者一起审阅 surface-form 子树

Nodes: language/surface-form, language/surface-form/binding-annotation, language/surface-form/borrow-lexicon, language/surface-form/construction-form, language/surface-form/iteration-forms, language/surface-form/match-form, language/surface-form/operation-spelling, language/surface-form/result-propagation

Summary: surface-form 树根删去两条证据政策决策（模型的习惯和内部语料计数不是依据），language 树根的"只看道理不看代价"已经陈述了它们；并吸收 binding-annotation 的唯一一条（函数体绑定的模式和类型由右侧推出、签名处仍要写）和 iteration-forms 的唯一一条（两种循环形式），这两个节点删除。子树里其余决策全部确认。只是复述决策的否决项从 borrow-lexicon、match-form、result-propagation 中删去，operation-spelling 只保留独立成立的四 token 前瞻那条；construction-form 的否决项去掉末尾一句关于该形式局限的观察，那不是理由。
> 通俗解释：拼写这一组的决策都是真的，主要是清掉大量"决策里已经说过一遍"的否决项，把两个单条文件并进父节点，把两条已经升到树根的话删掉。

## 2026-09-12 与所有者一起审阅 parallelism 子树与 pattern-doctrine

Nodes: language, language/parallelism, language/parallelism/loop-permission, language/parallelism/permission-judgment, language/pattern-doctrine

Summary: parallelism 保留从证明推导许可那条并换上不依赖优化器事实的理由，保留许可与实化分离，删去无调度边那条（树根擦除规则的重复），并按所有者的裁定删去"自动发现并行不是方向"那条已过时的决策及其重复的否决项。loop-permission 唯一属于语言层面的一句（计数循环本身就是许可判定点）并入 parallelism，其余理由讲的是规范条文怎么排版，节点删除。permission-judgment 不变。pattern-doctrine 保留封闭目录、可达替代方案规则和性能优先的修订规则；模型试验那条并入 language 树根的"只看道理不看代价"决策，与第一条 instead of 重复的否决项删去。
> 通俗解释：并行那组砍掉了两条重复或过时的话，把一个只有一句有用内容的小文件并进父节点；pattern-doctrine 留下真正的设计哲学，把"模型写得费劲不算语言上限"并进了树根那条"只看道理"。

## 2026-09-12 与所有者一起审阅定律决策与 ownership 子树

Nodes: language/contracts, language/ownership, language/ownership/affine-replacement, language/ownership/copy-classification, language/ownership/no-reborrow, language/ownership/no-reborrow/control-header-temporary-loans, language/ownership/slice-result-provenance

Summary: contracts 的定律决策现在说的是它本来的意思：带定律的 conformance 必须证明每一条定律源码才被接受，而定律只在规范规则指名的地方使用，例如并行归约的许可，因为在别处证明的定律是检查器之外的第二条证明路径。ownership 子树全部确认为真实的、有测量的决策；affine-replacement 的 slice 与 arena 那条去掉了关于未来 slice 重绑定设计的半句，control-header-temporary-loans 只有一条决策和一条否决项，并入 no-reborrow 后删除。no-reborrow 的调用结果来源与 slice-result-provenance 共用的理由两处都保留，因为它们管的是不同的构造。
> 通俗解释：定律那条改成了大白话能看懂的版本；ownership 这一组质量最高，只删了一句"将来再说"的话、把一个单条文件并入父节点。

## 2026-09-12 与所有者一起审阅 contracts、data-model、effects 与 name-resolution

Nodes: language/contracts, language/data-model, language/data-model/container-representation, language/data-model/tag-only-equality, language/effects, language/name-resolution

Summary: contracts 删去效果子类型那条否决项，它的理由只是复述决策；它的定律那条仍在讨论中。data-model 保留 struct-of-arrays 默认，把稳定身份改写为只声称只追加的契约存在，把搬迁规则一般化为任何在槽之间移动 owner 的操作而不再列举规范里没有的五种操作，删去树根已经陈述的语料证据那条，并吸收 container-representation 里两条属于数据模型的决策：数组状态，以及签名模式独立于表示；container-representation 删除，它的聚合表示那条是实现细节，库自选表示那条是对不存在之物的规划。tag-only-equality 去掉优化器事实的半句。effects 给精确性那条换上不依赖优化器事实的理由，并新增规范里有而树里缺的一条：`pure` 不承诺终止。name-resolution 不变。
> 通俗解释：这一组主要是把"写得像已经存在其实没有"和"给未来做规划"的内容清掉，把一个只剩实现细节的文件并入父节点，另外补上了一条重要的语言事实：`pure` 不等于一定会返回。

## 2026-09-12 与所有者一起审阅 checks-and-proofs 的子节点

Nodes: language/checks-and-proofs/certificate-fold, language/checks-and-proofs/obligation-discharge, language/checks-and-proofs/obligation-discharge/goal-decomposition, language/checks-and-proofs/obligation-discharge/loop-fact-retention, language/checks-and-proofs/obligation-discharge/writer-trap-surface, language/checks-and-proofs/requires-entry-contract, language/checks-and-proofs/requires-entry-contract/requirement-enforcement

Summary: certificate-fold 保留四条决策，删去两条否决项：一条是编译器内部的死胡同，一条与无符号倍数那条决策重复。obligation-discharge 删去编译期拒绝那条决策和 assume-without-check 那条否决项（都是两棵树根的重复），并吸收 writer-trap-surface 的两条决策：反对不可能到达分支的类型化结果规则，以及宿主资源边界；writer-trap-surface 连同它四条 claim 时代的否决项一起删除，language 树根的无陷阱规则已经覆盖它们。goal-decomposition 与 loop-fact-retention 不变。requires-entry-contract 里矛盾要求那条得到了所有者的理由：这样的实例是无害的死代码，而要拒绝它就得要求检查器识别出所有矛盾，这是无法承诺的，允许它反而让检查器保持简单。它吸收了 requirement-enforcement：调用点证明、无被调用者序言并入调用点子句那条，无契约的入口成为一条独立决策，未来外部边界那条作为对不存在之物的设计删除，两条否决项一并移入；requirement-enforcement 删除。base64 识别器那条否决项删去，编译器树根的按规则不按形状已覆盖。
> 通俗解释：这一组的内容基本都是真规则，改动是去重和合并：删掉和树根重复的、删掉当年 claim 语句的历史、把两个只有几条内容的小文件并进父节点。唯一的实质补充是"前置条件自相矛盾的函数算死代码不报错"这条终于有了真理由：检查器不可能保证认出所有矛盾。

## 2026-09-12 与所有者一起审阅 language 树根与 checks-and-proofs

Nodes: language, language/checks-and-proofs

Summary: language 树根保留四条决策。"一种行为"那条应所有者要求改为一般化表述：已接受的程序只有一种由语言固定的可观察行为，debug 与 release 只是例子；迁移成本那条也应所有者要求扩展为：语言选择只看本身的道理，不看迁移成本、不看现有语料里某件事出现的频率、也不看改动要花的工夫，因为语料是为锻炼编译器写的，不代表真实程序。checks-and-proofs 里，无 SMT 那条作为最重要的一条移到第一行；事实来源那条保留清单和否决项、去掉树根已经给出的理由；运行时回退那条作为树根的重复删除；第二条否决项改成直白的措辞。
> 通俗解释：树根四条都留，其中两条按所有者的意思写得更一般：一是程序行为只能有一种，不限于 debug/release；二是设计语言不看改起来费不费事、也不拿测试用例当统计数据。checks-and-proofs 把最重要的"不用 SMT"挪到第一条，删了一条和树根重复的，其余精简措辞。

## 2026-09-12 与所有者一起审阅 compiler 树剩余的叶子

Nodes: compiler/resource-exhaustion-floor, compiler/tag-only-lowering, compiler/wide-probe-lowering

Summary: 所有者确认这三个叶子都是有依据的真实决策：资源耗尽的中止记录与栈探测的选择、1 位与 32 位标签下沉及其测得的 34% 损失、以及宽探测快路径及其预注册的向量化器对照实验。资源耗尽那条因过长而改写措辞，并不再使用"可信计算基"一词，内容不变。至此 compiler 树审阅完毕：迁移产生的 10 个节点、35 条决策，现余 8 个节点、23 条决策。
> 通俗解释：编译器树的最后三条都留下了，只把一句写得太长的话理顺。编译器这棵树整个过完了，砍掉的基本都是空话、历史包袱和重复。

## 2026-09-12 与所有者一起审阅 parallel-lowering 子树

Nodes: compiler/parallel-lowering, compiler/parallel-lowering/two-worlds, compiler/parallel-lowering/parallel-runtime, compiler/parallel-lowering/lane-stack

Summary: 所有者裁定"计算并行默认关闭"那条决策不属于这棵树：`--par` 是实现选择，不影响程序的正确性，并行下沉将来也可能成为默认，所以删除。Windows 与启动那条缩短为其实际内容：运行时或配置坏了要大声失败，而工作线程比请求的少不算坏掉的配置。two-worlds 里克隆集合那条不再重复树根的"按规则不按形状"，改为它自己的理由：只有能到达交付点的函数在两个世界里才不同。parallel-runtime 里 I/O 等待那条并入它本就所属的当前栈运行时决策，lane-stack 的唯一一条决策及其被否决方案也并入，因此 lane-stack 删除。子树里其余内容均确认为真实的、有测量的决策。
> 通俗解释：这一组基本都是真决策，主要是精简：删掉了"并行默认关"这条（那只是个编译选项，不是设计决策），把 Windows 那条冗长的话缩成一句，把两条本来就是一回事的合并，把只有一条内容的 lane-stack 文件并进 parallel-runtime。

## 2026-09-12 与所有者一起审阅 cleanup-traversal 与 derived-totality

Nodes: compiler/cleanup-traversal, compiler/derived-totality

Summary: cleanup-traversal 保留所有者关于释放图成环的裁决及其两条被否决方案；它的第二条决策（每个 buffer 或 run 一个生成的释放循环）只是在描述代码，没有做任何决定，已删除。derived-totality 整个删除：第一条是所有者已经撤回的优化器事实规则，第二条在设计一个并不存在的终止性事实，而它背后唯一站得住的事实（`pure` 不等于终止，所以编译器绝不能承诺 `willreturn`）已由发出属性的测试钉死，且属于语言层面，将在 language 审阅时在 effects 下核对。roadmap 里指向被删节点的链接已去掉。
> 通俗解释：cleanup-traversal 留下了真正的裁决（类型释放成环时怎么处理），删掉了一条只是复述代码的话。derived-totality 整个节点都是围绕"事实开关"和一个没做出来的功能在空谈，所以删掉；真正要紧的那一点（不能向 LLVM 谎称函数一定会返回）已经有测试保证，以后放到语言树里去说。

## 2026-09-12 与所有者一起审阅 compiler 树根

Nodes: compiler, language

Summary: 所有者和 agent 逐条过了 compiler 树根。删除：两条优化器事实的决策，因为所有者的本意从来不是一对 facts-on 与 facts-off 的构建，而是不存在任何改变行为的构建模式，并且编译器交给 LLVM 的东西（比如别名属性或内联）是语言永远看不到的实现细节，不需要规则；"检查器属于可信计算基"那条，是同义反复，它唯一的内容（改编译器而不是改一致性判决）已经是 agent 指令里的规则；序列化回放和"以后再加固"两条，它们只是在否定一个没人再提的架构，现在合并为研究工具决策下的一条被否决方案；Python 参考模型的否决项，属于历史；以及 ripgrep 伞形目标，那是项目方向，现在放在 research 实验索引里。合并：单一 crate 那条变成了研究工具决策，并吸收了"实现要简单"和 safe Rust，后者的理由由 agent 补写；"一致性用例只是证据"与"分期不构成配置档"两条合成一条"按规则不按形状"的决策；确定性拒绝那条吸收了不许超时、预算、哈希顺序的规则。language 树根的优化器事实决策改成了所有者原话里的"一份源码一个程序"，待 language 审阅时再读一遍。agent 指令里的 compiler rules 一节现在指向这棵树，其中两条仓库规则移到了仓库整洁一节，因为所有者裁定实现准则属于被检查的树，而不属于指令。
> 通俗解释：这次是所有者亲自把 compiler 树根过了一遍，11 条砍到 4 条。砍掉的都是些空话或历史包袱：关于"优化器事实开关"的两条整个概念就不对，所有者要的只是"没有调试版发布版之分"，编译器内部给 LLVM 什么提示根本不用写规则；"检查器要可信"是废话；"不做序列化回放"是在反对一个早就没人提的老方案；Python 参考模型是陈年旧事；ripgrep 目标是项目方向，不是编译器设计，挪去了 research 目录。剩下的合并成四条：编译器是研究工具、只按规则不认程序、没实现的功能要明说、报错要确定且指明规则。顺带把 CLAUDE.md 里那一段编译器规则删了，改成指向这棵树。

## 2026-09-11 把拉取请求检查的发现应用到 compiler 树

Nodes: compiler, compiler/cleanup-traversal, compiler/tag-only-lowering, compiler/resource-exhaustion-floor, compiler/parallel-lowering, compiler/parallel-lowering/lane-stack, compiler/parallel-lowering/parallel-runtime

Summary: 本流程的拉取请求检查，针对代码对整棵 compiler 树运行了一遍。有一处真实的偏离（drift），需要项目负责人确认：cleanup-traversal 此前是从一份记忆记录迁移而来的，该记录描述的是一个由强连通分量决定的显式工作列表，但在本分支开始之前，生成器（emitter）依据项目负责人 2026-09-04 的裁定，已经去掉了那个工作列表以及它曾用来绕开的循环拒绝，现在会为每个节点类型生成一个释放动作，它会在释放图闭合的地方调用自身，并由栈账本把这次自调用报告为一条循环记录；该节点已依据生成器记录在案的裁定重写，工作列表与循环拒绝连同该裁定的理由一起移入了它的被否决列表，而 buffer 决策现在陈述的是代码实际拥有的那一个生成出来的释放循环。智能体所做的措辞修复，含义未变：根节点关于划分的那句话，不再使用"语义切分"的说法；tag-only-lowering 说明了字宽惩罚对向量化器具体做了什么；parallel-runtime 在第一次用到"粗粒度上限"和"偏斜状况"这两个说法时就给出了定义；parallel-lowering 中关于启动的决策，点名的是并行下沉版本及其顺序克隆，而不是"世界"与"克隆"；lane-stack 陈述了它所依赖的那条入口栈的事实；而 resource-exhaustion-floor 现在则说明，它的中止覆盖的是栈和堆，而一个不可用的计算 worker 是一次由普通调用回退路径处理的被拒绝 offer。其余每一项编译器决策都被发现已经实现，审查记录中引用的文件与函数保存在本仓库之外。
> 通俗解释：这次是把"拉取请求检查"这道关卡，对着实际代码把整个 compiler 决策树核对了一遍，看记录的决策和代码是否一致。核对中发现了一处真实的偏差：cleanup-traversal 节点原本记的是一种用强连通分量算出来的显式工作列表方案，但代码早就按项目负责人的裁定改成了"每种节点类型生成一个会自我调用的释放动作，靠栈账本识别自调用来代表循环"，所以这个节点被改写，旧方案连同理由一起放进了被否决列表，需要项目负责人确认。除此之外还做了一批不改变含义的措辞修正，比如把术语讲得更具体、给关键说法补上定义。其余的编译器决策都核实为代码里已经实现。

## 2026-09-11 把拉取请求检查的发现应用到 language 树

Nodes: language, language/checks-and-proofs/certificate-fold, language/checks-and-proofs/obligation-discharge, language/checks-and-proofs/obligation-discharge/goal-decomposition, language/checks-and-proofs/obligation-discharge/loop-fact-retention, language/checks-and-proofs/obligation-discharge/writer-trap-surface, language/checks-and-proofs/requires-entry-contract, language/contracts, language/ownership/no-reborrow, language/system-interface, language/system-interface/directory-enumeration, language/data-model/container-representation, language/pattern-doctrine

Summary: 本流程的拉取请求检查，针对规范对整棵 language 树运行了一遍：决策测试、一致性扫描，以及缺乏支撑节点的检查。由智能体应用的以下各项，都是含义源自原始记录或规范本身的措辞或精简修复：system-interface 根节点中的缺陷列表，此前与其替代方案的配对顺序有误，现已重新排序；那些需要依赖旧记录才能理解的术语被展开说明，具体是：certificate-fold 中操作数的已知值、loop-fact-retention 中 DEFLATE 解码器"29 处证出 5 处"的说法、goal-decomposition 中的两种守卫形态、no-reborrow 中的被拥有值穿引（threading）、obligation-discharge 中"把一切都变成陷阱"的编译模式、pattern-doctrine 中的编写者试验，以及 container-representation 中被推迟的库表示权威；在都用到了 contract 这个词的两个节点中，requires-and-ensures 块与类似 trait 的 contract 被区分开来了；directory-enumeration 中不加过滤的自身与父目录条目，现在附上了规范自身的理由；并且移除了两处对根节点已经陈述过的规则的重述——writer-trap-surface 中重复的陷阱原则，以及 obligation-discharge 中重复的单一权威规则——同时把被否决的 claim 形式连同它们各自的理由一起保留在 writer-trap-surface 之下。留给项目负责人决定、未作改动的内容：data-model 中 struct-of-arrays 这一默认设置，其依据是模式目录（patterns catalog）而不是规范；data-model 中的可回收稳定身份，读起来像是已经实现，而规范却仍然把可回收槽位的容器判定为受阻的；data-model 中的重新安置操作——重新哈希、压缩、drain 修复、移位，以及编码字符串的删除——在规范中根本找不到；以及 pattern-doctrine 整体，还有 surface-form 中两条关于证据政策的决策，其依据是章程（constitution）而不是规范正文，因此是否应当把它们留在这棵树中，由项目负责人来决定。
> 通俗解释：这次是同一个"拉取请求检查"关卡，改为针对规范文档核对整个 language 树，包括决策能不能被测试、彼此是否一致、有没有缺少支撑材料。这次没有发现真正的错误，主要是一批含义不变的措辞和精简修复，比如调整了缺陷列表的顺序、把依赖旧笔记才看得懂的术语展开讲清楚、把两种不同含义的"contract"区分开、给一条规则补上规范自身的理由、删掉了两处与根节点重复的说法。还有几处内容特意留白没有改动，因为它们的依据不是规范正文而是模式目录或章程，比如某个默认的数据布局设定和几个规范里根本找不到的操作，这些交给项目负责人自己决定是否保留。

## 2026-09-11 把 compiler 树更新到当前栈运行时与粒度控制

Nodes: compiler/parallel-lowering, compiler/parallel-lowering/two-worlds, compiler/parallel-lowering/parallel-runtime

Summary: `main` 在本分支创建之后合并了四项运行时变更：计算任务被迁移到持久化的原生线程与普通栈之上，"未命中即挂起"（park-on-miss）调度器及其调度枚举器被淘汰，并且落地了三项可选启用的 `--par` 粒度控制，其中标量叶子的默认值暂定为 16。这些变更的记录，此前是 `mcts_mem/whitefoot/parallelism.md`、`parallelism/two-worlds.md` 与 `system-interface.md` 中的新条目，以及本分支所淘汰的那份编译器 README 中的新段落；在这里，它们变成了六项决策与一个被否决的替代方案。有一条理由是由智能体拼合而成，而不是直接转录得来的，需要项目负责人加以确认：当前栈运行时这条理由的依据，是把 2026-09-06 的挂起代价测量结果，与项目负责人给出的"仅运行时拆分"方向结合了起来。`compiler/parallel-lowering` 中关于启动回退的理由，是从生成器（emitter）记录在案的说明中转录而来的。连接并发方面的缺口，以及已失效的 `WF_STACKS` 设置，归入 `docs/todo.md`；新增的各个标志（flag）归入根 README；标量叶子的重新测量方案，归入 proof-derived-parallelism 这项调查（investigation）。与 Windows worker 数量限定相关的条目，是归 `research/investigations/io-model/RESULTS.md` 所有的测量数据，不构成新的决策。
> 通俗解释：在这个分支开工之后，主分支又合并进了四项运行时改动：计算任务改成跑在常驻的原生线程和普通的栈上、原来"抢不到任务就挂起等待"的调度方式被淘汰、新增了三个可以用 `--par` 打开的粒度控制选项。这次改动把原本散落在旧记忆笔记和已经淘汰的编译器 README 里的相关说明，整理成了六条正式决策和一条被否决的方案。其中有一条理由是智能体自己综合出来的，不是照抄旧记录，需要项目负责人确认：它把之前测出来的"线程挂起代价"数据，和项目负责人定下的"只改运行时、不改别处"的方向结合到了一起。其余相关的遗留问题、新增选项说明、后续要做的重新测量，也分别归档到了待办文档、根 README 和一项后续调查里。

## 2026-09-11 迁移剩余的记忆子树并淘汰编译器 README

Nodes: language, language/checks-and-proofs, language/checks-and-proofs/certificate-fold, language/checks-and-proofs/obligation-discharge, language/checks-and-proofs/obligation-discharge/goal-decomposition, language/checks-and-proofs/obligation-discharge/loop-fact-retention, language/checks-and-proofs/obligation-discharge/writer-trap-surface, language/checks-and-proofs/requires-entry-contract, language/checks-and-proofs/requires-entry-contract/requirement-enforcement, language/contracts, language/data-model, language/data-model/container-representation, language/data-model/tag-only-equality, language/effects, language/name-resolution, language/ownership, language/ownership/affine-replacement, language/ownership/copy-classification, language/ownership/no-reborrow, language/ownership/no-reborrow/control-header-temporary-loans, language/ownership/slice-result-provenance, language/parallelism, language/parallelism/loop-permission, language/parallelism/permission-judgment, language/pattern-doctrine, language/surface-form, language/surface-form/binding-annotation, language/surface-form/borrow-lexicon, language/surface-form/construction-form, language/surface-form/iteration-forms, language/surface-form/match-form, language/surface-form/operation-spelling, language/surface-form/result-propagation, language/system-interface, language/system-interface/declaration-home, language/system-interface/directory-enumeration, language/system-interface/program-entry-form, compiler, compiler/cleanup-traversal, compiler/derived-totality, compiler/parallel-lowering, compiler/parallel-lowering/lane-stack, compiler/parallel-lowering/parallel-runtime, compiler/parallel-lowering/two-worlds, compiler/resource-exhaustion-floor, compiler/tag-only-lowering, compiler/wide-probe-lowering
Summary: `mcts_mem/` 中剩余的部分，依据"平实语言"规则被迁入这两棵树，理由取自各记录自身的替换条目与记录在案的依据，且不重述规范的语义。语言（Language）方面：surface-form 下有七个子节点，ownership 下有四个子节点和一个孙节点，data-model 下有两个子节点，system-interface 下有三个子节点，parallelism 下有两个子节点，此外还有 effects、name-resolution、contracts 与 pattern-doctrine。编译器（Compiler）方面：cleanup-traversal、resource-exhaustion-floor、wide-probe-lowering、tag-only-lowering、derived-totality，以及 parallel-lowering（下设 two-worlds、parallel-runtime 与 lane-stack），此外还有两条取自已淘汰的编译器 README 及项目负责人项目裁定的根节点决策：未支持的源码会被报告为未支持而绝不会报告为无效，以及 ripgrep（配合公平的两倍目标）是总体目标项目，且性能优先。经由"向上归一化"折叠：拼写规则被折叠进 surface-form 的根节点，操作数消耗被折叠进 result-propagation。未被保留的内容：development-workflow 子树，因为它的内容属于流程，其归属是 CLAUDE.md；permission-judgment 下关于已淘汰的 claim 与陷阱面的被否决替代方案，因为它们所争论的那个表面已经不复存在；以及带日期的测量数据，其归属是各自的结果记录。container-representation 的理由，是从项目负责人密集的修正意见中转录而来的，应当仔细阅读。`compiler/README.md` 已被淘汰：它的运行说明移入了根 README，已知缺陷移入了 `docs/todo.md`，架构决策移入了 `design/compiler`，而它的"已实现表面清单"则被舍弃，改用一致性测试报告；每一处现行引用都已被重新指向。`mcts_mem/` 会原地保留、冻结不变，直到项目负责人将其删除为止。
> 通俗解释：这是一次大规模的整理，把旧的 mcts_mem 笔记目录里剩下的所有内容，按"要写得让人看懂"的规则，统一搬进了 language 树和 compiler 树，搬运时只保留各自记录的替换理由，不重复规范已经讲过的话。language 树新增了一大批子节点，compiler 树也新增了好几个，同时把重复出现的相同规则合并到了根节点。有些内容被认为不属于这里而没有保留，比如一份流程说明，因为流程内容应当由 CLAUDE.md 来管，还有一份针对某个已经不存在的问题的被否决方案。同时，旧的编译器 README 被正式废弃：运行说明搬进了根 README，已知缺陷搬进了待办文档，架构方面的决定搬进了设计树，而它原来那份功能清单则不要了，改成看一致性测试报告。mcts_mem 目录本身先原样冻结保留，以后再由项目负责人决定删除。

## 2026-09-11 拆分为一棵语言树和一棵编译器树

Nodes: language, compiler, language/checks-and-proofs, language/checks-and-proofs/obligation-discharge, language/checks-and-proofs/obligation-discharge/goal-decomposition, language/checks-and-proofs/obligation-discharge/loop-fact-retention, language/checks-and-proofs/obligation-discharge/writer-trap-surface, language/checks-and-proofs/certificate-fold, language/checks-and-proofs/requires-entry-contract, language/checks-and-proofs/requires-entry-contract/requirement-enforcement
Summary: 项目负责人把语言决策和编译器决策分开了：语言决策依据规范来检查，编译器决策依据代码来检查，因此它们是两棵树。试点子树被迁移到了 `language` 之下，并且每一条重述了某条规则语义的语言决策，都被削减到只剩决策本身，因为语义归规范所有，而一份副本没有任何规则能让它保持最新。"检查器是可信计算基一部分"这条规则，从 obligation-discharge 移到了 compiler 的根节点，而优化器事实规则中关于行为的那一半放在 compiler 根节点，关于接受的那一半放在 language 根节点，二者互相点名对方。compiler 的根节点，取材于 toolchain 与 fact-channels 这两个记忆节点，理由取自它们各自的替换记录：单一可变的 safe Rust crate、内存中经检查的状态作为唯一的下沉权威来源、加固工作被推迟、一致性测试是证据而非权威、不采用按配置（profile）把关的接受判定、确定性的、引用规则的拒绝且不设可移植的首错误顺序，以及事实族只有在完成事实关闭归因并经受对抗性测试之后才被获准；Python 参照模型关卡是它的第一条被否决的替代方案。各份指导文件不再把决策导向 `mcts_mem/`，该目录会保持冻结，直到其内容被迁移完毕。推导索引（derivation ledger）尚未被删除：现行规范中的 META-6 要求它存在，`whitefoot-spec` 关卡也会读取它，因此将其淘汰，是一项项目负责人尚未批准的规范修订。
> 通俗解释：项目负责人决定把原来混在一起的决策拆成两棵独立的树，一棵是 language，专门放需要对照规范来检查的语言设计决策；一棵是 compiler，专门放需要对照代码来检查的实现决策。之前试点整理出来的内容被搬到了 language 下面，并且凡是重复陈述规范内容的决策都被削减到只剩决策本身，因为语义应该以规范为准，重复的副本没人负责保持同步。有两条原来混在一起的规则也被分开处理：一条移到了 compiler 根节点，另一条则一半留在 compiler、一半放到 language，彼此互相引用。compiler 树的根节点决策和它第一个被否决的方案，是从旧记忆笔记里整理出来的；推导索引暂时没有删除，因为删除它需要改规范，而这个分支还没得到批准去改规范。

## 2026-09-11 把"无维护者"规则应用到该 skill 的其余部分

Nodes: tree, tree/checks-and-proofs, tree/checks-and-proofs/obligation-discharge, tree/checks-and-proofs/obligation-discharge/goal-decomposition, tree/checks-and-proofs/obligation-discharge/loop-fact-retention, tree/checks-and-proofs/obligation-discharge/writer-trap-surface, tree/checks-and-proofs/certificate-fold, tree/checks-and-proofs/requires-entry-contract, tree/checks-and-proofs/requires-entry-contract/requirement-enforcement
Summary: 项目负责人定下的规则：任何没有规则去维护它的东西都会腐烂，因而不应当存在。这条规则被套用到了智能体此前添加的一切内容上。节点的标题行与文件名重复，已被去掉；一个节点现在就是它的决策行与被否决行本身。被否决行上的 `; lapses when` 子句，没有任何东西在检查它，已被去掉。`Nodes:` 行上的新增/修改/删除前缀，可以从 git 中推导出来，已被去掉。`design/README.md` 重述了这棵树的状态并携带了一份计划；它已被去掉，根 README 会点明这个目录。各个模板是 lint 所强制执行格式的第二份副本；已被去掉。各项检查提示被迁入了本流程之中，因此只剩一份文件。三份关于 lint 检查内容的描述，被合并为 lint 自身的提示消息。lint 现在是 `make check` 与 `make static` 的一个阶段，因为一个位于关卡之外的关卡目标是没有维护者的。启动引导（bootstrapping）一节是一次性的流程，已被去掉。根节点不再重述章程的各项原则；章程原本承载的四项具体决策——不把运行时陷阱当作语言特性、不绕过必需的证明、不做指数级的检查工作、在没有真实用户之前迁移成本不能作为依据——如今连同各自的理由一起存放在这里，而章程则只保留宗旨、目标与优先级。没有其他决策内容发生变化。
> 通俗解释：项目负责人定了一条通用规则——任何没有专门机制去维护它的东西，迟早会过时失真，所以干脆不应该存在。这次改动把这条规则套用到之前加过的各种零碎内容上，删掉或合并了一批没有规则去维护的东西，比如重复的标题、没人核实的失效说明、能从 git 自己推导出来的增删改标记、和别处重复的说明文件、和 lint 重复的格式模板。lint 检查也被并入了标准的检查流程，不再是单独维护的东西。章程原来重复讲过的几条具体规则，现在带着理由留在这棵树里，章程本身只保留最上层的宗旨和优先级。整个改动没有改变任何一条决策的实质内容，纯粹是删冗余、合并重复。

## 2026-09-11 移除 Scope 字段

Nodes: tree, tree/checks-and-proofs, tree/checks-and-proofs/obligation-discharge, tree/checks-and-proofs/obligation-discharge/goal-decomposition, tree/checks-and-proofs/obligation-discharge/loop-fact-retention, tree/checks-and-proofs/obligation-discharge/writer-trap-surface, tree/checks-and-proofs/certificate-fold, tree/checks-and-proofs/requires-entry-contract, tree/checks-and-proofs/requires-entry-contract/requirement-enforcement
Summary: 项目负责人依据一条通用规则移除了 `Scope:`：一个没有规则去维护它的字段会腐烂，因而不应当存在。Scope 重述的是节点标题与其位置本已暗示的内容，而且没有任何东西会去更新它。这一行已从每一个节点中删除，lint 不再要求它，也不再规定它的顺序，本流程与模板也都不再提及它。没有决策内容发生变化。
> 通俗解释：项目负责人依据上一条"没人维护就不该存在"的规则，把每个节点上的 Scope 字段整体删掉了。原因是这个字段说的内容，节点的标题和它在树里的位置本来就已经说明白了，而且没有任何东西会去更新它。这次改动把 Scope 这一行从所有节点里删除，并同步更新了检查规则、写作流程说明和模板，让它们都不再要求这个字段。没有任何一条决策的实质内容因此改变。

## 2026-09-11 为从未见过记录的读者书写决策

Nodes: tree/checks-and-proofs/requires-entry-contract
Summary: 项目负责人读不懂关于契约块（contract block）的那条决策：它的理由是从记忆记录中的一堆专业术语（伪运行时、已擦除状态、未命名结果约定、alpha 展开、符号化的整体结果数据、窄整数关系载体）压缩而来的。该节点的全部五条决策，以及两条被否决行，都被改写为平实的措辞，内容未作任何改变。项目负责人把这一点定为一条常设规则：一条决策是写给一个从未见过其来源记录的读者看的，而一个不得不打开源材料才能理解其理由的读者，其实是发现了这个节点本身的一个缺陷。这条规则被添加进了本流程的节点格式一节、启动引导步骤，以及设计关卡检查 G1 之中。剩余的子树将依据这条规则迁移，其他试点节点也将被排查是否存在同样的缺陷。
> 通俗解释：项目负责人反映看不懂某个节点里关于"契约块"的那条决策，因为它的理由是直接从旧笔记里的一堆专业术语压缩而来，不去翻旧材料根本理解不了。这次改动把该节点全部的决策和被否决方案都用大白话重新讲了一遍，含义完全没变，只是让人能看懂。项目负责人借这件事定下一条新的常设规则：一条决策必须写给完全没见过来源笔记的读者看，如果读者非得翻旧材料才能理解某条决策的道理，那就说明这条决策本身写得有问题。这条新规则被写进了写作流程和检查清单里，以后其余节点都要照这条规则检查一遍。

## 2026-09-11 节点布局：scope 在先、字段之间空行分隔、拒绝须附理由

Nodes: tree, tree/checks-and-proofs, tree/checks-and-proofs/obligation-discharge, tree/checks-and-proofs/obligation-discharge/goal-decomposition, tree/checks-and-proofs/obligation-discharge/loop-fact-retention, tree/checks-and-proofs/obligation-discharge/writer-trap-surface, tree/checks-and-proofs/certificate-fold, tree/checks-and-proofs/requires-entry-contract, tree/checks-and-proofs/requires-entry-contract/requirement-enforcement
Summary: 仅涉及格式；没有决策内容发生变化。项目负责人在阅读了渲染出来的树之后提出了三点要求：字段之间用空行隔开，以便各自渲染成独立的段落；`Scope:` 直接放在标题下方，因为它说明的是该节点所统辖的范围，审阅者需要在看到各项决策之前先看到它；以及每一行 `Rejected:` 都要用决策所使用的同一个 `because` 标记来附上理由，因为一次没有理由的拒绝算不上一条记录。模板、本流程、检查提示 G1，以及 lint 被一并更新，每一个节点都以机械化的方式做了转换。
> 通俗解释：这次改动只是调整排版格式，没有改变任何决策的实质内容。项目负责人看过实际渲染出来的树之后提了三点要求：各个字段之间要空一行，读起来更清楚；Scope 字段要放在标题正下方，因为读者应该先看范围再看具体决策；每一条被否决的方案都要像正式决策一样写明理由，因为没有理由的拒绝算不上完整的记录。相应地，模板、写作流程和检查规则都做了同步更新，所有节点也统一按新格式转换了一遍。

## 2026-09-11 明确优化器事实规则中接受那一半所约束的对象

Nodes: tree
Summary: 项目负责人发现，"可选优化器事实"这条决策中关于接受的那一半，并没有说清楚它约束的对象是谁，而关于行为的那一半则是清楚且可测试的。这一行现在点明了它的约束对象及其可测试的形式：无论事实是开启还是关闭，编译器都会以相同的判定结果接受相同的一批程序，因此检查器的事实来源对优化器的输出是封闭的，没有任何语言规则会让接受与否取决于某个可选事实是否可推导。它的理由是：否则接受与否将取决于优化器的版本、目标平台或遍（pass）的执行顺序，并且一个只有在事实开启时才被接受的程序，将没有一个事实关闭的参照可供行为规则去比较。现有的两个具体实例，依据"向上归一化"规则被留在根节点之外，待日后迁移时归入各自的子树：一个未被证明独立性的 par 会下沉为顺序执行而不是被拒绝（属于 parallelism）；以及一个已声明的定律，无论是否有任何优化器使用它，都要接受检查（属于 fact-channels）。
> 通俗解释：项目负责人发现，"可选优化器事实"这条规则里，关于运行行为的那一半说得很清楚也容易检验，但关于"程序是否被接受"的那一半，却没说清楚它到底约束的是谁。这次改动把这句话改得更具体：不管某项优化器事实是打开还是关闭，编译器都必须对同一批程序给出相同的接受或拒绝结果，检查器不能依赖优化器额外算出来的信息。补充的理由是：如果不这样规定，接受与否就会跟着优化器版本、目标平台或执行顺序变来变去，而且一个只有打开某个事实才被接受的程序，根本没有一个"事实关闭"的版本可供对照检查。原本放在根节点的两个具体例子被移出，留到以后各自的子树里再处理。

## 2026-09-11 陈述可选优化器事实背后的真实理由

Nodes: tree
Summary: 项目负责人对"可选优化器事实绝不会改变接受结果或语义"这条根节点决策提出了疑问：它迁移过来的理由只是在重述这条规则本身，而没有为它提供依据。记录在案的依据，是从 2026-07 的启动引导计划（bootstrap plan）中找回的——该计划把一个事实关闭的不动点（fixpoint）冻结下来，作为事实开启编译器的判定基准（oracle）——以及从 fact-channels 和 parallelism 这两个节点中找回的。这条决策现在是两行，附有四条真实的理由：接受与否绝不能取决于优化器的版本、目标平台或遍的执行顺序；后端在不重新检查的情况下信任所发出的属性，因此只有一个正确的事实关闭参照才能揭露一个错误的属性；每个事实族的收益都是针对这同一个参照来归因的；以及事实开启与事实关闭之间的行为差异，将会重新引入调试版与发布版的分裂。项目负责人确认这条决策予以保留。parallelism 这个记忆节点中近乎重复的那句话，将依据"向上归一化"规则，在该子树迁移时被去掉。
> 通俗解释：项目负责人质疑一条根节点规则——"可选的优化器事实绝不能改变接受结果或语义"——说它当初迁移过来时只是把规则本身又说了一遍，并没有真正解释为什么要这样规定。这次改动去翻了旧的启动计划和另外两份笔记，找回了四条站得住脚的理由：接受与否不能随优化器版本或平台变化；编译后端直接信任优化器给出的结论而不重新检查，所以必须有一个"全部关闭"的正确版本作参照，才能发现优化器给出的错误结论；每项优化带来的收益都是拿这同一个参照来对比算出来的；如果开事实和关事实的程序行为不一样，就等于又制造出了调试版和发布版不一致的老问题。项目负责人确认这条规则应当保留，另一处几乎重复的旧说法留待以后删除。

## 2026-09-11 迁移根节点与 checks-and-proofs 子树

Nodes: tree, tree/checks-and-proofs, tree/checks-and-proofs/obligation-discharge, tree/checks-and-proofs/obligation-discharge/goal-decomposition, tree/checks-and-proofs/obligation-discharge/loop-fact-retention, tree/checks-and-proofs/obligation-discharge/writer-trap-surface, tree/checks-and-proofs/certificate-fold, tree/checks-and-proofs/requires-entry-contract, tree/checks-and-proofs/requires-entry-contract/requirement-enforcement
Summary: 在提交（commit）3016842 处，从 `mcts_mem/whitefoot.md` 与 `mcts_mem/whitefoot/checks-and-proofs/` 进行的试点迁移。现行的摘要要点变成了各项决策；`.alt` 节点与 `replaced` 移动变成了带有记录在案理由的 `Rejected:` 行；带日期的事实、测量数据与注意事项（pitfall）被舍弃，因为它们归属于各自的结果记录与 `compiler/README.md`。有一条带日期的陈述被保留为一项决策，因为没有任何现行的要点陈述过它：一个整数类型的具名 const 是一个仿射原子。有一条带日期的陈述没有被保留，因为现行的 PRF-1 冗余规则与它相矛盾：2026-07-11 提出的原则，即一个未被使用的显式 check 绝不构成硬性失败；它转而以被否决的替代方案的形式出现。凡是现行要点没有附带记录在案理由的地方，理由都取自最相近的记录在案的依据，应当加以确认。设立这次试点，是为了在剩余的十二个子树被迁移之前，先校准各项精简过滤规则。这条条目只是转录；它不做出任何新的决策。项目负责人将以树差异的形式来审阅它。
> 通俗解释：这是最早的一次试点迁移，把旧笔记里根节点和 checks-and-proofs 相关的内容，按新的格式搬进了设计树，对应到某个具体的代码提交时间点。旧笔记里的要点变成了正式决策条目，标记为"替代方案"或"已废弃"的内容变成了带理由的被否决行，而带具体日期的事实、测量数据和注意事项则被舍弃，因为它们更适合放在各自的实验结果记录里。有一条旧说法因为没有别处提到过而被保留成正式决策，另一条因为和现行规则矛盾而没有被当作决策保留，改放进了被否决方案里。这次操作本质上只是转录整理，不做任何新的实质决策，目的是在搬剩下的子树之前先把整理规则调试好。

# 设计树变更日志

按最新优先排列。每一次被批准的树变更对应一条条目：一个带日期的标题、一行 `Nodes:`
列出每一个发生变更的节点，以及一段 `Summary:`；格式由 `skill/SKILL.md` 规定。

## 2026-09-11 把拉取请求检查的发现应用到 compiler 树

Nodes: compiler, compiler/cleanup-traversal, compiler/tag-only-lowering, compiler/resource-exhaustion-floor, compiler/parallel-lowering, compiler/parallel-lowering/lane-stack, compiler/parallel-lowering/parallel-runtime

Summary: 本流程的拉取请求检查，针对代码对整棵 compiler 树运行了一遍。有一处真实的偏离（drift），需要项目负责人确认：cleanup-traversal 此前是从一份记忆记录迁移而来的，该记录描述的是一个由强连通分量决定的显式工作列表，但在本分支开始之前，生成器（emitter）依据项目负责人 2026-09-04 的裁定，已经去掉了那个工作列表以及它曾用来绕开的循环拒绝，现在会为每个节点类型生成一个释放动作，它会在释放图闭合的地方调用自身，并由栈账本把这次自调用报告为一条循环记录；该节点已依据生成器记录在案的裁定重写，工作列表与循环拒绝连同该裁定的理由一起移入了它的被否决列表，而 buffer 决策现在陈述的是代码实际拥有的那一个生成出来的释放循环。智能体所做的措辞修复，含义未变：根节点关于划分的那句话，不再使用"语义切分"的说法；tag-only-lowering 说明了字宽惩罚对向量化器具体做了什么；parallel-runtime 在第一次用到"粗粒度上限"和"偏斜状况"这两个说法时就给出了定义；parallel-lowering 中关于启动的决策，点名的是并行下沉版本及其顺序克隆，而不是"世界"与"克隆"；lane-stack 陈述了它所依赖的那条入口栈的事实；而 resource-exhaustion-floor 现在则说明，它的中止覆盖的是栈和堆，而一个不可用的计算 worker 是一次由普通调用回退路径处理的被拒绝 offer。其余每一项编译器决策都被发现已经实现，审查记录中引用的文件与函数保存在本仓库之外。

## 2026-09-11 把拉取请求检查的发现应用到 language 树

Nodes: language, language/checks-and-proofs/certificate-fold, language/checks-and-proofs/obligation-discharge, language/checks-and-proofs/obligation-discharge/goal-decomposition, language/checks-and-proofs/obligation-discharge/loop-fact-retention, language/checks-and-proofs/obligation-discharge/writer-trap-surface, language/checks-and-proofs/requires-entry-contract, language/contracts, language/ownership/no-reborrow, language/system-interface, language/system-interface/directory-enumeration, language/data-model/container-representation, language/pattern-doctrine

Summary: 本流程的拉取请求检查，针对规范对整棵 language 树运行了一遍：决策测试、一致性扫描，以及缺乏支撑节点的检查。由智能体应用的以下各项，都是含义源自原始记录或规范本身的措辞或精简修复：system-interface 根节点中的缺陷列表，此前与其替代方案的配对顺序有误，现已重新排序；那些需要依赖旧记录才能理解的术语被展开说明，具体是：certificate-fold 中操作数的已知值、loop-fact-retention 中 DEFLATE 解码器"29 处证出 5 处"的说法、goal-decomposition 中的两种守卫形态、no-reborrow 中的被拥有值穿引（threading）、obligation-discharge 中"把一切都变成陷阱"的编译模式、pattern-doctrine 中的编写者试验，以及 container-representation 中被推迟的库表示权威；在都用到了 contract 这个词的两个节点中，requires-and-ensures 块与类似 trait 的 contract 被区分开来了；directory-enumeration 中不加过滤的自身与父目录条目，现在附上了规范自身的理由；并且移除了两处对根节点已经陈述过的规则的重述——writer-trap-surface 中重复的陷阱原则，以及 obligation-discharge 中重复的单一权威规则——同时把被否决的 claim 形式连同它们各自的理由一起保留在 writer-trap-surface 之下。留给项目负责人决定、未作改动的内容：data-model 中 struct-of-arrays 这一默认设置，其依据是模式目录（patterns catalog）而不是规范；data-model 中的可回收稳定身份，读起来像是已经实现，而规范却仍然把可回收槽位的容器判定为受阻的；data-model 中的重新安置操作——重新哈希、压缩、drain 修复、移位，以及编码字符串的删除——在规范中根本找不到；以及 pattern-doctrine 整体，还有 surface-form 中两条关于证据政策的决策，其依据是章程（constitution）而不是规范正文，因此是否应当把它们留在这棵树中，由项目负责人来决定。

## 2026-09-11 把 compiler 树更新到当前栈运行时与粒度控制

Nodes: compiler/parallel-lowering, compiler/parallel-lowering/two-worlds, compiler/parallel-lowering/parallel-runtime

Summary: `main` 在本分支创建之后合并了四项运行时变更：计算任务被迁移到持久化的原生线程与普通栈之上，"未命中即挂起"（park-on-miss）调度器及其调度枚举器被淘汰，并且落地了三项可选启用的 `--par` 粒度控制，其中标量叶子的默认值暂定为 16。这些变更的记录，此前是 `mcts_mem/whitefoot/parallelism.md`、`parallelism/two-worlds.md` 与 `system-interface.md` 中的新条目，以及本分支所淘汰的那份编译器 README 中的新段落；在这里，它们变成了六项决策与一个被否决的替代方案。有一条理由是由智能体拼合而成，而不是直接转录得来的，需要项目负责人加以确认：当前栈运行时这条理由的依据，是把 2026-09-06 的挂起代价测量结果，与项目负责人给出的"仅运行时拆分"方向结合了起来。`compiler/parallel-lowering` 中关于启动回退的理由，是从生成器（emitter）记录在案的说明中转录而来的。连接并发方面的缺口，以及已失效的 `WF_STACKS` 设置，归入 `docs/todo.md`；新增的各个标志（flag）归入根 README；标量叶子的重新测量方案，归入 proof-derived-parallelism 这项调查（investigation）。与 Windows worker 数量限定相关的条目，是归 `research/investigations/io-model/RESULTS.md` 所有的测量数据，不构成新的决策。

## 2026-09-11 迁移剩余的记忆子树并淘汰编译器 README

Nodes: language, language/checks-and-proofs, language/checks-and-proofs/certificate-fold, language/checks-and-proofs/obligation-discharge, language/checks-and-proofs/obligation-discharge/goal-decomposition, language/checks-and-proofs/obligation-discharge/loop-fact-retention, language/checks-and-proofs/obligation-discharge/writer-trap-surface, language/checks-and-proofs/requires-entry-contract, language/checks-and-proofs/requires-entry-contract/requirement-enforcement, language/contracts, language/data-model, language/data-model/container-representation, language/data-model/tag-only-equality, language/effects, language/name-resolution, language/ownership, language/ownership/affine-replacement, language/ownership/copy-classification, language/ownership/no-reborrow, language/ownership/no-reborrow/control-header-temporary-loans, language/ownership/slice-result-provenance, language/parallelism, language/parallelism/loop-permission, language/parallelism/permission-judgment, language/pattern-doctrine, language/surface-form, language/surface-form/binding-annotation, language/surface-form/borrow-lexicon, language/surface-form/construction-form, language/surface-form/iteration-forms, language/surface-form/match-form, language/surface-form/operation-spelling, language/surface-form/result-propagation, language/system-interface, language/system-interface/declaration-home, language/system-interface/directory-enumeration, language/system-interface/program-entry-form, compiler, compiler/cleanup-traversal, compiler/derived-totality, compiler/parallel-lowering, compiler/parallel-lowering/lane-stack, compiler/parallel-lowering/parallel-runtime, compiler/parallel-lowering/two-worlds, compiler/resource-exhaustion-floor, compiler/tag-only-lowering, compiler/wide-probe-lowering
Summary: `mcts_mem/` 中剩余的部分，依据"平实语言"规则被迁入这两棵树，理由取自各记录自身的替换条目与记录在案的依据，且不重述规范的语义。语言（Language）方面：surface-form 下有七个子节点，ownership 下有四个子节点和一个孙节点，data-model 下有两个子节点，system-interface 下有三个子节点，parallelism 下有两个子节点，此外还有 effects、name-resolution、contracts 与 pattern-doctrine。编译器（Compiler）方面：cleanup-traversal、resource-exhaustion-floor、wide-probe-lowering、tag-only-lowering、derived-totality，以及 parallel-lowering（下设 two-worlds、parallel-runtime 与 lane-stack），此外还有两条取自已淘汰的编译器 README 及项目负责人项目裁定的根节点决策：未支持的源码会被报告为未支持而绝不会报告为无效，以及 ripgrep（配合公平的两倍目标）是总体目标项目，且性能优先。经由"向上归一化"折叠：拼写规则被折叠进 surface-form 的根节点，操作数消耗被折叠进 result-propagation。未被保留的内容：development-workflow 子树，因为它的内容属于流程，其归属是 CLAUDE.md；permission-judgment 下关于已淘汰的 claim 与陷阱面的被否决替代方案，因为它们所争论的那个表面已经不复存在；以及带日期的测量数据，其归属是各自的结果记录。container-representation 的理由，是从项目负责人密集的修正意见中转录而来的，应当仔细阅读。`compiler/README.md` 已被淘汰：它的运行说明移入了根 README，已知缺陷移入了 `docs/todo.md`，架构决策移入了 `design/compiler`，而它的"已实现表面清单"则被舍弃，改用一致性测试报告；每一处现行引用都已被重新指向。`mcts_mem/` 会原地保留、冻结不变，直到项目负责人将其删除为止。

## 2026-09-11 拆分为一棵语言树和一棵编译器树

Nodes: language, compiler, language/checks-and-proofs, language/checks-and-proofs/obligation-discharge, language/checks-and-proofs/obligation-discharge/goal-decomposition, language/checks-and-proofs/obligation-discharge/loop-fact-retention, language/checks-and-proofs/obligation-discharge/writer-trap-surface, language/checks-and-proofs/certificate-fold, language/checks-and-proofs/requires-entry-contract, language/checks-and-proofs/requires-entry-contract/requirement-enforcement
Summary: 项目负责人把语言决策和编译器决策分开了：语言决策依据规范来检查，编译器决策依据代码来检查，因此它们是两棵树。试点子树被迁移到了 `language` 之下，并且每一条重述了某条规则语义的语言决策，都被削减到只剩决策本身，因为语义归规范所有，而一份副本没有任何规则能让它保持最新。"检查器是可信计算基一部分"这条规则，从 obligation-discharge 移到了 compiler 的根节点，而优化器事实规则中关于行为的那一半放在 compiler 根节点，关于接受的那一半放在 language 根节点，二者互相点名对方。compiler 的根节点，取材于 toolchain 与 fact-channels 这两个记忆节点，理由取自它们各自的替换记录：单一可变的 safe Rust crate、内存中经检查的状态作为唯一的下沉权威来源、加固工作被推迟、一致性测试是证据而非权威、不采用按配置（profile）把关的接受判定、确定性的、引用规则的拒绝且不设可移植的首错误顺序，以及事实族只有在完成事实关闭归因并经受对抗性测试之后才被获准；Python 参照模型关卡是它的第一条被否决的替代方案。各份指导文件不再把决策导向 `mcts_mem/`，该目录会保持冻结，直到其内容被迁移完毕。推导索引（derivation ledger）尚未被删除：现行规范中的 META-6 要求它存在，`whitefoot-spec` 关卡也会读取它，因此将其淘汰，是一项项目负责人尚未批准的规范修订。

## 2026-09-11 把"无维护者"规则应用到该 skill 的其余部分

Nodes: tree, tree/checks-and-proofs, tree/checks-and-proofs/obligation-discharge, tree/checks-and-proofs/obligation-discharge/goal-decomposition, tree/checks-and-proofs/obligation-discharge/loop-fact-retention, tree/checks-and-proofs/obligation-discharge/writer-trap-surface, tree/checks-and-proofs/certificate-fold, tree/checks-and-proofs/requires-entry-contract, tree/checks-and-proofs/requires-entry-contract/requirement-enforcement
Summary: 项目负责人定下的规则：任何没有规则去维护它的东西都会腐烂，因而不应当存在。这条规则被套用到了智能体此前添加的一切内容上。节点的标题行与文件名重复，已被去掉；一个节点现在就是它的决策行与被否决行本身。被否决行上的 `; lapses when` 子句，没有任何东西在检查它，已被去掉。`Nodes:` 行上的新增/修改/删除前缀，可以从 git 中推导出来，已被去掉。`design/README.md` 重述了这棵树的状态并携带了一份计划；它已被去掉，根 README 会点明这个目录。各个模板是 lint 所强制执行格式的第二份副本；已被去掉。各项检查提示被迁入了本流程之中，因此只剩一份文件。三份关于 lint 检查内容的描述，被合并为 lint 自身的提示消息。lint 现在是 `make check` 与 `make static` 的一个阶段，因为一个位于关卡之外的关卡目标是没有维护者的。启动引导（bootstrapping）一节是一次性的流程，已被去掉。根节点不再重述章程的各项原则；章程原本承载的四项具体决策——不把运行时陷阱当作语言特性、不绕过必需的证明、不做指数级的检查工作、在没有真实用户之前迁移成本不能作为依据——如今连同各自的理由一起存放在这里，而章程则只保留宗旨、目标与优先级。没有其他决策内容发生变化。

## 2026-09-11 移除 Scope 字段

Nodes: tree, tree/checks-and-proofs, tree/checks-and-proofs/obligation-discharge, tree/checks-and-proofs/obligation-discharge/goal-decomposition, tree/checks-and-proofs/obligation-discharge/loop-fact-retention, tree/checks-and-proofs/obligation-discharge/writer-trap-surface, tree/checks-and-proofs/certificate-fold, tree/checks-and-proofs/requires-entry-contract, tree/checks-and-proofs/requires-entry-contract/requirement-enforcement
Summary: 项目负责人依据一条通用规则移除了 `Scope:`：一个没有规则去维护它的字段会腐烂，因而不应当存在。Scope 重述的是节点标题与其位置本已暗示的内容，而且没有任何东西会去更新它。这一行已从每一个节点中删除，lint 不再要求它，也不再规定它的顺序，本流程与模板也都不再提及它。没有决策内容发生变化。

## 2026-09-11 为从未见过记录的读者书写决策

Nodes: tree/checks-and-proofs/requires-entry-contract
Summary: 项目负责人读不懂关于契约块（contract block）的那条决策：它的理由是从记忆记录中的一堆专业术语（伪运行时、已擦除状态、未命名结果约定、alpha 展开、符号化的整体结果数据、窄整数关系载体）压缩而来的。该节点的全部五条决策，以及两条被否决行，都被改写为平实的措辞，内容未作任何改变。项目负责人把这一点定为一条常设规则：一条决策是写给一个从未见过其来源记录的读者看的，而一个不得不打开源材料才能理解其理由的读者，其实是发现了这个节点本身的一个缺陷。这条规则被添加进了本流程的节点格式一节、启动引导步骤，以及设计关卡检查 G1 之中。剩余的子树将依据这条规则迁移，其他试点节点也将被排查是否存在同样的缺陷。

## 2026-09-11 节点布局：scope 在先、字段之间空行分隔、拒绝须附理由

Nodes: tree, tree/checks-and-proofs, tree/checks-and-proofs/obligation-discharge, tree/checks-and-proofs/obligation-discharge/goal-decomposition, tree/checks-and-proofs/obligation-discharge/loop-fact-retention, tree/checks-and-proofs/obligation-discharge/writer-trap-surface, tree/checks-and-proofs/certificate-fold, tree/checks-and-proofs/requires-entry-contract, tree/checks-and-proofs/requires-entry-contract/requirement-enforcement
Summary: 仅涉及格式；没有决策内容发生变化。项目负责人在阅读了渲染出来的树之后提出了三点要求：字段之间用空行隔开，以便各自渲染成独立的段落；`Scope:` 直接放在标题下方，因为它说明的是该节点所统辖的范围，审阅者需要在看到各项决策之前先看到它；以及每一行 `Rejected:` 都要用决策所使用的同一个 `because` 标记来附上理由，因为一次没有理由的拒绝算不上一条记录。模板、本流程、检查提示 G1，以及 lint 被一并更新，每一个节点都以机械化的方式做了转换。

## 2026-09-11 明确优化器事实规则中接受那一半所约束的对象

Nodes: tree
Summary: 项目负责人发现，"可选优化器事实"这条决策中关于接受的那一半，并没有说清楚它约束的对象是谁，而关于行为的那一半则是清楚且可测试的。这一行现在点明了它的约束对象及其可测试的形式：无论事实是开启还是关闭，编译器都会以相同的判定结果接受相同的一批程序，因此检查器的事实来源对优化器的输出是封闭的，没有任何语言规则会让接受与否取决于某个可选事实是否可推导。它的理由是：否则接受与否将取决于优化器的版本、目标平台或遍（pass）的执行顺序，并且一个只有在事实开启时才被接受的程序，将没有一个事实关闭的参照可供行为规则去比较。现有的两个具体实例，依据"向上归一化"规则被留在根节点之外，待日后迁移时归入各自的子树：一个未被证明独立性的 par 会下沉为顺序执行而不是被拒绝（属于 parallelism）；以及一个已声明的定律，无论是否有任何优化器使用它，都要接受检查（属于 fact-channels）。

## 2026-09-11 陈述可选优化器事实背后的真实理由

Nodes: tree
Summary: 项目负责人对"可选优化器事实绝不会改变接受结果或语义"这条根节点决策提出了疑问：它迁移过来的理由只是在重述这条规则本身，而没有为它提供依据。记录在案的依据，是从 2026-07 的启动引导计划（bootstrap plan）中找回的——该计划把一个事实关闭的不动点（fixpoint）冻结下来，作为事实开启编译器的判定基准（oracle）——以及从 fact-channels 和 parallelism 这两个节点中找回的。这条决策现在是两行，附有四条真实的理由：接受与否绝不能取决于优化器的版本、目标平台或遍的执行顺序；后端在不重新检查的情况下信任所发出的属性，因此只有一个正确的事实关闭参照才能揭露一个错误的属性；每个事实族的收益都是针对这同一个参照来归因的；以及事实开启与事实关闭之间的行为差异，将会重新引入调试版与发布版的分裂。项目负责人确认这条决策予以保留。parallelism 这个记忆节点中近乎重复的那句话，将依据"向上归一化"规则，在该子树迁移时被去掉。

## 2026-09-11 迁移根节点与 checks-and-proofs 子树

Nodes: tree, tree/checks-and-proofs, tree/checks-and-proofs/obligation-discharge, tree/checks-and-proofs/obligation-discharge/goal-decomposition, tree/checks-and-proofs/obligation-discharge/loop-fact-retention, tree/checks-and-proofs/obligation-discharge/writer-trap-surface, tree/checks-and-proofs/certificate-fold, tree/checks-and-proofs/requires-entry-contract, tree/checks-and-proofs/requires-entry-contract/requirement-enforcement
Summary: 在提交（commit）3016842 处，从 `mcts_mem/whitefoot.md` 与 `mcts_mem/whitefoot/checks-and-proofs/` 进行的试点迁移。现行的摘要要点变成了各项决策；`.alt` 节点与 `replaced` 移动变成了带有记录在案理由的 `Rejected:` 行；带日期的事实、测量数据与注意事项（pitfall）被舍弃，因为它们归属于各自的结果记录与 `compiler/README.md`。有一条带日期的陈述被保留为一项决策，因为没有任何现行的要点陈述过它：一个整数类型的具名 const 是一个仿射原子。有一条带日期的陈述没有被保留，因为现行的 PRF-1 冗余规则与它相矛盾：2026-07-11 提出的原则，即一个未被使用的显式 check 绝不构成硬性失败；它转而以被否决的替代方案的形式出现。凡是现行要点没有附带记录在案理由的地方，理由都取自最相近的记录在案的依据，应当加以确认。设立这次试点，是为了在剩余的十二个子树被迁移之前，先校准各项精简过滤规则。这条条目只是转录；它不做出任何新的决策。项目负责人将以树差异的形式来审阅它。

Decision: 表层名称（surface name）标记的是规范中定义的、经过检查的 invariant，绝不会借用后端的词汇，因为像 noalias 这样的名字，命名的是一个下沉之后的结果，而不是检查器真正强制执行的那个 invariant，而不是在语言表层使用后端术语。

Decision: 独占借用模式的拼写是 `&uniq`，因为这个模式的 invariant 是唯一性，而不是可变性，而 `mut` 把独占性和写权限混为一谈，这种混淆在未来的内部可变性（interior-mutability）能力下会出问题，而不是采用 Rust 的 `&mut`。

Rejected:
- 遵循 Rust 惯例使用 `&mut`：被否决，因为独占模式的 invariant 是唯一性而不是可变性，而 `mut` 把独占性和写权限混为一谈，在未来的内部可变性能力下会出问题。
- 把 `noalias` 用作表层名称：被否决，因为它是后端词汇，命名的是一个下沉之后的结果，而不是那个经过检查的 invariant。

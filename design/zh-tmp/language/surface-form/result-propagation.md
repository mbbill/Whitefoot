Decision: 可恢复的失败是一个普通的 `Result` 值，通过 `let value = propagate expression;` 来转发，不存在异常、throw、catch，或栈展开（unwinding），因为 `try` 这个词通常暗示进入异常处理控制流，而这门语言实际做的只是转发一个普通的值，而 `propagate` 恰如其分地命名了这个动作，而不是采用 `try` 这种拼写。

Decision: 一个被 `propagate` 使用的、裸的仿射 `Result` 位置，恰好消耗一次其存储根，显式的 `move` 依然合法，因为转发操作必须消耗一个仿射操作数，而普通的消耗规则能让规范的裸写法做到这一点，且不削弱所有权，而要求写成 `propagate move p`，则与已获认可的编写者写法相矛盾，而不是要求一个显式的 move 操作数。

Rejected:
- `let value = try expression;`：被否决，因为 `try` 暗示着这门语言并不具备的异常处理控制流。
- 对一个具名的仿射 Result 要求写成 `propagate move result`：被否决，因为转发操作必须消耗一个仿射操作数，而裸写法在普通消耗规则下就能做到这一点，且不削弱所有权。

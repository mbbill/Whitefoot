#[derive(Clone, Copy)]
struct Handle(usize);

enum Node {
    Lit(i32),
    Add(Handle, Handle),
}

struct Arena {
    nodes: Vec<Node>,
}

impl Arena {
    fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    fn lit(&mut self, value: i32) -> Handle {
        self.push(Node::Lit(value))
    }

    fn add(&mut self, lhs: Handle, rhs: Handle) -> Handle {
        self.push(Node::Add(lhs, rhs))
    }

    fn push(&mut self, node: Node) -> Handle {
        let handle = Handle(self.nodes.len());
        self.nodes.push(node);
        handle
    }

    fn eval(&self, handle: Handle) -> i32 {
        match self.nodes[handle.0] {
            Node::Lit(value) => value,
            Node::Add(lhs, rhs) => self.eval(lhs) + self.eval(rhs),
        }
    }
}

fn main() {
    let mut arena = Arena::new();
    let one = arena.lit(1);
    let two = arena.lit(2);
    let three = arena.lit(3);
    let four = arena.lit(4);
    let left = arena.add(one, two);
    let right = arena.add(three, four);
    let root = arena.add(left, right);
    assert_eq!(arena.nodes.len(), 7);
    assert_eq!(arena.eval(root), 10);
    drop(arena);
    println!("ok");
}

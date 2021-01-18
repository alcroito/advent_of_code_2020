use std::cell::RefCell;

type Id = usize;
#[derive(Debug)]
#[allow(unused)]
enum Val<'a> {
    Leaf(i32),
    Add(Vec<&'a Val<'a>>),
}

type RuleNodeMap<'a> = std::collections::HashMap<Id, &'a Val<'a>>;
#[allow(unused)]
struct RuleTree<'a> {
    arena: &'a typed_arena::Arena<Val<'a>>,
    rule_node_map: RefCell<RuleNodeMap<'a>>,
}

impl<'a> RuleTree<'a> {
    fn new(arena: &'a typed_arena::Arena<Val<'a>>) -> RuleTree<'a> {
        RuleTree {
            arena,
            rule_node_map: RefCell::new(RuleNodeMap::new()),
        }
    }

    fn build_rule_tree_recursive(&self, _rule_id: usize) {}
}

fn take_tree(t: RuleTree) {
    dbg!(t.rule_node_map);
}

fn main() {
    let arena = typed_arena::Arena::new();
    let rule_tree = RuleTree::new(&arena);
    rule_tree.build_rule_tree_recursive(0);
    take_tree(rule_tree);
}

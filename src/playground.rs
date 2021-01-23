use std::cell::UnsafeCell;
use std::cell::RefCell;

use itertools::Itertools;

fn play() {
    let a = (1..=4)
        .into_iter()
        .map(|v| (1..=v).into_iter().map(|v| (1..=v)))
        .flatten()
        .flatten()
        .collect::<Vec<_>>();
    println!("{:?}", a);
}

struct State {
    number: u8,
}

enum CallKind {
    CallF1,
    CallF2,
}

fn get_call_fn(kind: &CallKind) -> Box<dyn Fn(&u8, &mut State)> {
    let f1 = |i: &u8, state: &mut State| state.number = *i + 1;
    let f2 = |i: &u8, state: &mut State| state.number = *i + 2;

    match kind {
        CallKind::CallF1 => Box::new(f1),
        CallKind::CallF2 => Box::new(f2),
    }
}

fn call_closure(kind: &CallKind) {
    let mut s = State { number: 0 };
    let call_fn = get_call_fn(&kind);

    [1, 2, 3].iter().for_each(|i| call_fn(i, &mut s));
    println!("s is {}", s.number);
}

fn nested_iterator_borrowing() {
    let _a = (0..3)
        .map(|y| (0..3).map(move |x| (x, y)))
        .collect::<Vec<_>>();

    // less efficient?
    let xs = 0..2;
    let ys = 0..2;
    let zs = 0..2;
    let _vertices = xs.flat_map(|x| {
        let z_it = zs.clone();
        ys.clone()
            .flat_map(move |y| z_it.clone().map(move |z| (x, y, z)))
    });

    // more efficient?
    let xs = 0..2;
    let ys = 0..2;
    let zs = 0..2;
    let _vertices = xs.flat_map(|x| {
        ys.clone().flat_map({
            let z_it = &zs;
            move |y| z_it.clone().map(move |z| (x, y, z))
        })
    });

    // Efficient and no nested let rebinding?
    let xs = 0..2;
    let ys = 0..2;
    let zs = 0..2;
    let xs = &xs;
    let ys = &ys;
    let _xyzs = zs.flat_map(|z| {
        ys.clone()
            .flat_map(move |y| xs.clone().map(move |x| (x, y, z)))
    });
    dbg!(_xyzs.collect::<Vec<_>>());

    let ys = 0..2;
    let ys_ref = &ys;
    let _ys_clone = ys.clone(); //         Same type \
    let _ys_ref_clone = ys_ref.clone(); // Same type /
}

fn pass_mutable_dyn_closure() {
    let mut state = 0;
    {   
        let closure = || {
            state += 1;
            state
        };
        let mut boxed_closure: Box<dyn FnMut() -> usize> = Box::new(closure);
        let unboxed_closure = boxed_closure.as_mut();
        call_my_fn(unboxed_closure);
    }
    println!("final value is: {}", state);

    println!("val is: {}", state);
    let closure = || {
        state += 1;
        state
    };
    
    {
        let mut boxed_closure: Box<dyn FnMut() -> usize> = Box::new(closure);
        println!("reference address is: {:p}", boxed_closure.as_ref());
        let pointer = boxed_closure.as_mut();
        let pointer = pointer as *mut dyn FnMut() -> usize;
        println!("pointer   address is: {:p}", pointer);
        unsafe {
            let pointer = &mut *pointer;
            pointer();
            pointer();
            pointer();
        }

    }
    println!("val is: {}", state);
}

fn call_my_fn<F>(mut f: F) where F: FnMut() -> usize {
    f();
}

type Id = usize;
enum RuleKind {
    Leaf(i32),
    Composite(Vec<Id>)
}
#[derive(Debug)]
enum RuleTreeNode<'a> {
    Leaf(i32),
    Composite(Vec<&'a RuleTreeNode<'a>>)
}

type Rules = std::collections::HashMap<Id, RuleKind>;
type RuleNodeMap<'a> = std::collections::HashMap<Id, &'a RuleTreeNode<'a>>;
struct RuleTree<'a> {
    rules: Rules,
    arena: &'a typed_arena::Arena<RuleTreeNode<'a>>,
    rule_node_map: RefCell<RuleNodeMap<'a>>
}

impl<'a> RuleTree<'a> {
    fn new(r: Rules, arena: &'a typed_arena::Arena<RuleTreeNode<'a>>) -> RuleTree<'a> {
        RuleTree {
            rules: r, 
            arena,
            rule_node_map: RefCell::new(RuleNodeMap::new()),
        }
    }

    fn build_rule_tree_recursive(&self, rule_id: usize) {
        let t = self;
        if t.rule_node_map.borrow().contains_key(&rule_id) {
            return;
        }
        let rule = &t.rules[&rule_id];
        match rule {
            RuleKind::Leaf(leaf_value) => {
                let new_arena_node = t.arena.alloc(RuleTreeNode::Leaf(*leaf_value));
                t.rule_node_map.borrow_mut().insert(rule_id, new_arena_node);
            },
            RuleKind::Composite(child_ids) => {
                let mut child_vec = vec![];
                {
                    for child_id in child_ids {
                        let child_exists = t.rule_node_map.borrow().contains_key(child_id);
                        if !child_exists {
                            t.build_rule_tree_recursive(*child_id);
                        } 
                        let child_arena_node = *t.rule_node_map.borrow().get(&child_id).unwrap();
                        child_vec.push(child_arena_node);
                    }
                }
                let val = RuleTreeNode::Composite(child_vec);
                let new_arena_node = t.arena.alloc(val);
                t.rule_node_map.borrow_mut().insert(rule_id, new_arena_node);
            },
        }
    }
}

fn consume_tree(t: RuleTree) {
    dbg!(t.rule_node_map);
}

fn example_of_clunky_arena_based_graph() {
    let mut r = Rules::new();
    r.insert(0, RuleKind::Composite(vec![1]));
    r.insert(1, RuleKind::Composite(vec![2, 2]));
    r.insert(2, RuleKind::Leaf(5));

    // Unfortunately it's not possible to encapsulate both the arena and the RuleTree into a single
    // struct, because that would be a self-referential struct, and it can't be moved by consume_tree.
    // Loooooots of searching around, and the best advice people give is either to keep the structs
    // separate, or revert to using index based graphs rather than refs.
    // owning_ref also seems to not help. Haven't tried rental because that's unmaintaned.
    let arena = typed_arena::Arena::new();
    let rule_tree = RuleTree::new(r, &arena);
    rule_tree.build_rule_tree_recursive(0);
    consume_tree(rule_tree);
}

fn example_of_valid_temporary_mutable_borrows() -> i32 {
    /*
    C++ code.

    int *val = new int();
    int& ref1 = *val;
    val += 5;
    int *p1 = &ref1;
    *p1 += 6;
    *val += 1;
    return *val;
    */

    // Presumably equivalent valid rust code, as per Kimundi.
    let val: Box<UnsafeCell<i32>> = Box::new(UnsafeCell::new(0));
    let ref1: &UnsafeCell<i32> = &*val;
    unsafe { *val.get() += 5 };
    let p1: &UnsafeCell<i32> = &ref1;
    unsafe { *p1.get() += 6 };
    unsafe { *val.get() += 5 };
    unsafe { *val.get() }
}

fn example_of_undefined_behavior_multiple_aliasing_mutable_refs() {
    let val: Box<UnsafeCell<i32>> = Box::new(UnsafeCell::new(0));
    let ref1: &UnsafeCell<i32> = &*val;
    unsafe { 
        let cell_p = ref1.get();
        let ref1_1 = &mut *ref1.get(); // first long-living &mut
        bar(cell_p);
        println!("{}", ref1_1);
    };

}
fn bar(val: *mut i32) {
    unsafe {
        let ref1_2 = &mut *val; // second long-living &mut, UB
        *ref1_2 = 5;
    }
}

fn main() {
    example_of_undefined_behavior_multiple_aliasing_mutable_refs();
    example_of_valid_temporary_mutable_borrows();
    example_of_clunky_arena_based_graph();

    // (0..3)
    // .map(|i| (i * 2)..(i * 2 + 2))
    // .multi_cartesian_product().for_each(|o| println!("{:?}", o));
    vec![(1..=2), (1..=1), (1..=1)]
    .into_iter()
    .multi_cartesian_product().for_each(|o| println!("{:?}", o));

    pass_mutable_dyn_closure();
    nested_iterator_borrowing();
    play();
    call_closure(&CallKind::CallF1);
    call_closure(&CallKind::CallF2);
}

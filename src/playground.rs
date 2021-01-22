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

fn call_my_fn<F>(mut f: F) 
where F: FnMut() -> usize
{
    f();
}

// enum RuleKind {
//     Leaf(i32),
//     Wrapper(usize)
// }

// type Id = usize;
// // type MyFunc = FnMut(usize) -> usize;
// type RefFunc<'f> = dyn FnMut(usize) -> usize + 'f;
// type BoxedFunc<'f> = Box<dyn FnMut(usize) -> usize + 'f>;
// type Rules = std::collections::HashMap<Id, Vec<RuleKind>>;
// type FuncMap<'f> = std::collections::HashMap<Id, BoxedFunc<'f>>;

// fn build_func<'r, 'm>(_r: &'r Rules, rule_id: usize, memo: &'m mut FuncMap) -> &'m RefFunc<'m> {
//     let a = match memo.entry(rule_id) {
//         std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
//         std::collections::hash_map::Entry::Vacant(entry) => {
//             let new_b = Box::new(|_| 1);
//             entry.insert(new_b)     
//          },
//     };
//     a
//     // if let Some(b_boxed) = memo.get(&rule_id) {
//     //     let b_ref = b_boxed.as_ref();
//     //     return b_ref;
//     // }
//     // let new_b = Box::new(|_| 1);
//     // let new_b_ref = &*new_b;
//     // memo.insert(rule_id, new_b);
//     // new_b_ref
// }

type Id = usize;
enum RuleKind {
    Leaf(i32),
    ChildIds(Vec<Id>)
}
#[derive(Debug)]
enum Val<'a> {
    Leaf(i32),
    Add(RefCell<Option<Vec<&'a Val<'a>>>>)
}
type BoxedVal<'a> = Box<Val<'a>>;
type Rules = std::collections::HashMap<Id, RuleKind>;
type FuncMap<'a> = std::collections::HashMap<Id, BoxedVal<'a>>;

fn alloc_nodes<'a>(r: &Rules, rule_id: usize, 
                   memo: &'a RefCell<std::collections::HashMap<Id, Box<Val<'a>>>>) {
    if memo.borrow().contains_key(&rule_id) {
        return;
    }
    let rule = &r[&rule_id];
    match rule {
        RuleKind::Leaf(leaf_value) => {
            // let new_b = arena.alloc(Val::Leaf(*leaf_value));
            let new_b = Box::new(Val::Leaf(*leaf_value));
            memo.borrow_mut().insert(rule_id, new_b);
        },
        RuleKind::ChildIds(..) => {
            // let mut child_vec = vec![];
            // let memo_b = memo.borrow();
            // for child_id in child_ids {
            //     let child_b = *memo_b.get(child_id).unwrap();
            //     child_vec.push(child_b);
            // }
            let val = Val::Add(RefCell::new(None));
            // let new_b = arena.alloc(val);
            let new_b = Box::new(val);
            memo.borrow_mut().insert(rule_id, new_b);
        },
    }

    // match memo.borrow_mut().entry(rule_id) {
    //     std::collections::hash_map::Entry::Occupied(..) => (),
    //     std::collections::hash_map::Entry::Vacant(entry) => {
    //         let rule = &r[&rule_id];
    //         match rule {
    //             RuleKind::Leaf(leaf_value) => {
    //                 let new_b = Box::new(Val::Leaf(*leaf_value));
    //                 entry.insert(new_b);
    //             },
    //             RuleKind::ChildIds(child_ids) => {
    //                 let mut child_vec = vec![];
    //                 let memo_b = memo.borrow();
    //                 for child_id in child_ids {
    //                     let child_b = memo_b.get(child_id).unwrap().as_ref();
    //                     child_vec.push(child_b);
    //                 }
    //                 let val = Val::Add(child_vec);
    //                 let new_b = Box::new(val);
    //                 entry.insert(new_b);
    //             },
    //         }
    //      },
    // };
}

fn init_nodes<'a>(r: &Rules, rule_id: usize, memo: &'a RefCell<std::collections::HashMap<Id, Box<Val<'a>>>>) {
    let rule = &r[&rule_id];
    match rule {
        RuleKind::Leaf(..) => (),
        RuleKind::ChildIds(child_ids) => {
            let memo_b = memo.borrow();
            let branch = memo_b.get(&rule_id).unwrap().as_ref();
            match branch {
                Val::Add(cell) => {
                    let mut borrowed_cell = cell.borrow_mut();
                    if borrowed_cell.is_none() {
                        let mut child_vec = vec![];
                        for child_id in child_ids {
                            let child_b1 = memo_b.get(child_id).unwrap();
                            let child_b = child_b1.as_ref();
                            child_vec.push(child_b);
                        }
                        *borrowed_cell = Some(child_vec);
                    }
                },
                _ => unreachable!(),
            }
        },
    }
}



fn init_post_order_vec(r: &Rules, rule_id: usize, post_order_vec: &mut Vec<usize>) {
    let rule = &r[&rule_id];
    match rule {
        RuleKind::Leaf(..) => (),
        RuleKind::ChildIds(child_ids) => {
            for child_id in child_ids {
                init_post_order_vec(r, *child_id, post_order_vec)
            }
        },
    }
    post_order_vec.push(rule_id);
}

fn init_boxed_vals() {
    let mut r = Rules::new();
    r.insert(0, RuleKind::ChildIds(vec![1]));
    r.insert(1, RuleKind::ChildIds(vec![2, 2]));
    r.insert(2, RuleKind::Leaf(5));

    // let arena = typed_arena::Arena::new();
    let memo = RefCell::new(FuncMap::new());
    let mut post_order_vec:Vec<usize> = vec![];
    init_post_order_vec(&r, 0, &mut post_order_vec);
    for rule_id in post_order_vec {
        alloc_nodes(&r, rule_id, &memo);
        init_nodes(&r, rule_id, &memo);
    }
    dbg!(&memo);
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

    init_boxed_vals();

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

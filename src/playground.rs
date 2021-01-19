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

fn main() {

    // (0..3)
    // .map(|i| (i * 2)..(i * 2 + 2))
    // .multi_cartesian_product().for_each(|o| println!("{:?}", o));
    vec![(1..=2), (1..=1), (1..=1)]
    .into_iter()
    .multi_cartesian_product().for_each(|o| println!("{:?}", o));

    nested_iterator_borrowing();
    play();
    call_closure(&CallKind::CallF1);
    call_closure(&CallKind::CallF2);
}

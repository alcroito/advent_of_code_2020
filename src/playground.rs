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

fn main() {
    play();
    call_closure(&CallKind::CallF1);
    call_closure(&CallKind::CallF2);
}

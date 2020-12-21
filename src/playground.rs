fn play() {
    let a = (1..=4)
        .into_iter()
        .map(|v| (1..=v).into_iter().map(|v| (1..=v)))
        .flatten()
        .flatten()
        .collect::<Vec<_>>();
    println!("{:?}", a);
}

fn main() {
    play();
}

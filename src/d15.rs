use advent::helpers;
use anyhow::{Context, Result};
use itertools::Itertools;

type NumType = u32;
type Numbers = Vec<NumType>;
type NumberHistoryMap = std::collections::HashMap<NumType, NumType>;
fn parse_numbers(s: &str) -> Result<Numbers, std::num::ParseIntError> {
    s.trim()
        .split(',')
        .map(|n| n.parse::<NumType>())
        .try_collect()
}

fn compute_spoken_number(s: &str, target_turn: usize) -> NumType {
    const BOUNDARY: NumType = 30_000_000 / 10;
    let nums = parse_numbers(s).expect("Invalid numbers");
    let mut history_high_numbers = NumberHistoryMap::with_capacity(262144);
    let mut history_low_numbers: Vec<_> = vec![0; BOUNDARY as usize];
    nums.iter().enumerate().for_each(|(turn, &number)| {
        // turn is a 1-based index.
        let turn = turn + 1;
        history_low_numbers[number as usize] = turn as NumType;
    });
    let turn_begin = nums.len() + 1;
    let mut prev = *nums.iter().rev().next().expect("no previous number");

    (turn_begin..=target_turn).for_each(|turn| {
        // For faster performance, lookup small number values in a vector, and big numbers
        // in the hashmap.
        let prev_turn = turn as NumType - 1;
        if prev < BOUNDARY {
            let prev_num_turn = &mut history_low_numbers[prev as usize];
            prev = if *prev_num_turn == 0 {
                0
            } else {
                prev_turn - *prev_num_turn
            };
            *prev_num_turn = prev_turn;
        } else {
            history_high_numbers
                .entry(prev)
                .and_modify(|prev_num_turn| {
                    prev = prev_turn - *prev_num_turn;
                    *prev_num_turn = prev_turn;
                })
                .or_insert_with(|| {
                    prev = 0;
                    prev_turn
                });
        }
    });
    prev
}

fn compute_spoken_number_p1(s: &str) -> NumType {
    compute_spoken_number(s, 2020)
}

fn compute_spoken_number_p2(s: &str) -> NumType {
    compute_spoken_number(s, 30000000)
}

fn solve_p1() -> Result<()> {
    let input = helpers::get_data_from_file_res("d15").context("Coudn't read file contents.")?;
    let result = compute_spoken_number_p1(&input);
    println!("The 2020th spoken number is: {}", result);
    Ok(())
}

fn solve_p2() -> Result<()> {
    let input = helpers::get_data_from_file_res("d15").context("Coudn't read file contents.")?;
    let result = compute_spoken_number_p2(&input);
    println!("The 30000000th spoken number is: {}", result);
    Ok(())
}

fn main() -> Result<()> {
    solve_p1().ok();
    solve_p2()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p1() {
        let input = "0,3,6";
        let result = compute_spoken_number_p1(input);
        assert_eq!(result, 436);

        // assert_eq!(compute_spoken_number_p1("1,3,2"), 1);
        // assert_eq!(compute_spoken_number_p1("2,1,3"), 10);
        // assert_eq!(compute_spoken_number_p1("1,2,3"), 27);
        // assert_eq!(compute_spoken_number_p1("2,3,1"), 78);
        // assert_eq!(compute_spoken_number_p1("3,2,1"), 438);
        // assert_eq!(compute_spoken_number_p1("3,1,2"), 1836);
    }

    // #[test]
    // fn test_p2() {
    //     let input = "0,3,6";
    //     let result = compute_spoken_number_p2(input);
    //     assert_eq!(result, 175594);

    //     assert_eq!(compute_spoken_number_p2("1,3,2"), 2578);
    //     assert_eq!(compute_spoken_number_p2("2,1,3"), 3544142);
    //     assert_eq!(compute_spoken_number_p2("1,2,3"), 261214);
    //     assert_eq!(compute_spoken_number_p2("2,3,1"), 6895259);
    //     assert_eq!(compute_spoken_number_p2("3,2,1"), 18);
    //     assert_eq!(compute_spoken_number_p2("3,1,2"), 362);
    // }
}

/*
turn prev spoken
1  -  0
2  0  3
3  3  6
4  6  0
5  0  3
6  3  3
7  3  1
8  1  0
9  0  4
10 4  0
11 0  2
12 2  0
13 0  2
14 2  2
15 2  1
16 1  8
17 8  0
18 0  5
19 5  0
20 0  2
21 2  6
22 6  18
23 18 0
24 0  4
25 4  15
26 15 0
27 0  3
28 3  21
29 21 0
30 0  3
31 3  3
32 3  1
33 1  17
34 17 0
35 0  5
36 5  17
37 17 3
38 3  6
39 6  17
40 17 3

*/

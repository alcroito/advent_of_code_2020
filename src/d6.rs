use advent::helpers;
use anyhow::{Context, Result};
use std::collections::HashMap;

type MyChar = u8;
type CharCounter = HashMap<MyChar, u32>;

pub enum Op {
    Any,
    All,
}

pub fn get_sum_of_yes_answers(input: &str, op: Op) -> u32 {
    input
        .trim()
        .split("\n\n")
        .map(|group| {
            let (group_answers, person_count) = group.split('\n').fold(
                (CharCounter::new(), 0),
                |(mut acc, person_count), person_answers| {
                    person_answers.as_bytes().iter().cloned().for_each(|c| {
                        let entry = acc.entry(c).or_insert(0);
                        *entry += 1;
                    });
                    (acc, person_count + 1)
                },
            );
            match op {
                Op::Any => group_answers.len() as u32,
                Op::All => group_answers
                    .iter()
                    .filter(|(_, &count)| count == person_count)
                    .count() as u32,
            }
        })
        .sum::<u32>()
}

fn solve_p1() -> Result<()> {
    let data = helpers::get_data_from_file_res("d6").context("Coudn't read file contents.")?;
    let answer = get_sum_of_yes_answers(&data, Op::Any);
    println!("Part 1 answer is: {}", answer);
    Ok(())
}

fn solve_p2() -> Result<()> {
    let data = helpers::get_data_from_file_res("d6").context("Coudn't read file contents.")?;
    let answer = get_sum_of_yes_answers(&data, Op::All);
    println!("Part 2 answer is: {}", answer);
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
        let input = "
abc

a
b
c

ab
ac

a
a
a
a

b";
        let answer = get_sum_of_yes_answers(input, Op::Any);
        assert_eq!(answer, 11);
    }

    #[test]
    fn test_p2() {
        let input = "
abc

a
b
c

ab
ac

a
a
a
a

b";
        let answer = get_sum_of_yes_answers(input, Op::All);
        assert_eq!(answer, 6);
    }
}

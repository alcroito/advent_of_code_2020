use advent::helpers;
use anyhow::{Context, Result};
use itertools::Itertools;

fn detect_fake_number(numbers: &[i64], capacity: usize) -> Option<i64> {
    let mut q = circular_queue::CircularQueue::<i64>::with_capacity(capacity);
    numbers.iter().take(capacity).for_each(|v| {
        q.push(*v);
    });
    numbers.iter().skip(capacity).find_map(|needle| {
        let is_valid_number = q.iter().combinations(2).find(|pair| {
            let pair_sum = pair[0] + pair[1];
            pair_sum == *needle
        });
        match is_valid_number {
            Some(_) => {
                // Push the valid number onto queue. Return None,
                // To continue search for invalid number.
                q.push(*needle);
                None
            }
            // Found fake number.
            None => Some(*needle),
        }
    })
}

fn find_weakness(numbers: &[i64], target: i64) -> i64 {
    let n_len = numbers.len();
    numbers
        .iter()
        .rev()
        .skip(1)
        .rev()
        .enumerate()
        .find_map(|(i, _)| {
            (i + 1..n_len).into_iter().find_map(|j| {
                let contiguous_sum: i64 = numbers[i..j].iter().sum();
                if contiguous_sum == target {
                    Some((i, j))
                } else {
                    None
                }
            })
        })
        .map(|(i, j)| {
            let (min, max) = numbers[i..j]
                .iter()
                .minmax()
                .into_option()
                .expect("No min and max found");
            min + max
        })
        .expect("No weakness found")
}

fn solve_p1() -> Result<()> {
    let data = helpers::get_data_from_file_res("d9").context("Coudn't read file contents.")?;
    let numbers = helpers::lines_to_longs(&data);
    let result = detect_fake_number(&numbers, 25).expect("fake number not found");
    println!("Found part 1 fake number: {}", result);
    Ok(())
}

fn solve_p2() -> Result<()> {
    let data = helpers::get_data_from_file_res("d9").context("Coudn't read file contents.")?;
    let numbers = helpers::lines_to_longs(&data);
    let fake_number = detect_fake_number(&numbers, 25).expect("fake number not found");
    let result = find_weakness(&numbers, fake_number);
    println!("Found part 2 weakness: {}", result);
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
35
20
15
25
47
40
62
55
65
95
102
117
150
182
127
219
299
277
309
576";
        let numbers = helpers::lines_to_longs(input);
        let result = detect_fake_number(&numbers, 5).expect("fake number not found");
        assert_eq!(result, 127);
    }

    #[test]
    fn test_p2() {
        let input = "
35
20
15
25
47
40
62
55
65
95
102
117
150
182
127
219
299
277
309
576";
        let numbers = helpers::lines_to_longs(input);
        let fake_number = detect_fake_number(&numbers, 5).expect("fake number not found");
        let result = find_weakness(&numbers, fake_number);
        assert_eq!(result, 62);
    }
}

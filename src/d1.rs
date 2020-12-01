use advent::helpers;
use std::collections::HashSet;

#[derive(Debug)]
struct TwoNums(i64, i64);
impl PartialEq for TwoNums {
    fn eq(&self, other: &Self) -> bool {
        let TwoNums(x1, x2) = self;
        let TwoNums(y1, y2) = other;
        if (x1 == y1 && x2 == y2) || (x1 == y2 && x2 == y1) {
            return true;
        }
        false
    }
}

fn find_two_numbers_sum(target_sum: i64, numbers: &[i64]) -> Option<TwoNums> {
    let mut complements = HashSet::new();
    for number in numbers.iter() {
        if complements.len() == 0 {
            complements.insert(number);
        } else {
            let complement: i64 = target_sum - number;
            if complements.contains(&complement) {
                return Some(TwoNums(*number, complement));
            }
            complements.insert(number);
        }
    }
    None
}

fn get_two_numbers_product(nums: &Option<TwoNums>) -> Option<i64> {
    match nums {
        Some(TwoNums(n1, n2)) => Some(n1 * n2),
        None => None,
    }
}

fn get_two_numbers_sum_and_product(target_sum: i64, numbers: &[i64]) -> (Option<TwoNums>, Option<i64>) {
    let nums = find_two_numbers_sum(target_sum, numbers);
    let result = get_two_numbers_product(&nums);
    (nums, result)
}

fn solve_p1() {
    const TARGET_SUM:i64 = 2020;
    let data = helpers::get_data_from_file("d1").unwrap();
    let numbers = helpers::lines_to_longs(&data);

    if let (Some(TwoNums(n1, n2)), Some(result)) = get_two_numbers_sum_and_product(TARGET_SUM, &numbers) {
        println!("The 2 numbers summed to {} are: {}, {}", TARGET_SUM, n1, n2);
        println!("The 2 numbers multipled are: {} ", result);
    } else {
        println!("No numbers summed to {}.", TARGET_SUM);
    }
}

#[derive(Debug)]
struct ThreeNums(i64, i64, i64);
impl PartialEq for ThreeNums {
    fn eq(&self, other: &Self) -> bool {
        let mut v1 = vec![self.0, self.1, self.2];
        let mut v2 = vec![other.0, other.1, other.2];
        v1.sort_unstable();
        v2.sort_unstable();
        return v1 == v2;
    }
}

fn find_three_numbers_sum(target_sum: i64, numbers: &[i64]) -> Option<ThreeNums> {
    let mut number_set = HashSet::new();
    for number in numbers.iter() {
        number_set.insert(number);
    }
    for n1 in numbers.iter() {
        for n2 in numbers.iter() {
            let complement: i64 = target_sum - n1 - n2;
            if number_set.contains(&complement) {
                return Some(ThreeNums(*n1, *n2, complement));
            }
        }
    }
    None
}

fn get_three_numbers_product(nums: &Option<ThreeNums>) -> Option<i64> {
    match nums {
        Some(ThreeNums(n1, n2, n3)) => Some(n1 * n2 * n3),
        None => None,
    }
}

fn solve_p2() {
    const TARGET_SUM:i64 = 2020;
    let data = helpers::get_data_from_file("d1").unwrap();
    let numbers = helpers::lines_to_longs(&data);

    if let Some(ThreeNums(n1, n2, n3)) = find_three_numbers_sum(TARGET_SUM, &numbers) {
        println!("The 3 numbers summed to {} are: {}, {}, {}", TARGET_SUM, n1, n2, n3);

        let result = get_three_numbers_product(&Some(ThreeNums(n1, n2, n3))).unwrap();
        println!("The 3 numbers multipled are: {} ", result);
    } else {
        println!("No numbers summed to {}.", TARGET_SUM);
    }
}

#[test]
fn test_p1() {
    const TARGET_SUM:i64 = 2020;
    assert_eq!(find_two_numbers_sum(TARGET_SUM, &[1721, 979, 366, 299, 675, 1456]), Some(TwoNums(1721, 299)));
    assert_eq!(get_two_numbers_sum_and_product(TARGET_SUM, &[1721, 979, 366, 299, 675, 1456]).1, Some(514579));
    assert_eq!(get_two_numbers_sum_and_product(TARGET_SUM, &[500, 1520]).1, Some(760000));
}

#[test]
fn test_p2() {
    const TARGET_SUM:i64 = 2020;
    assert_eq!(find_three_numbers_sum(TARGET_SUM, &[1721, 979, 366, 299, 675, 1456]), Some(ThreeNums(979, 366, 675)));
}

fn main() {
    solve_p1();
    solve_p2();
}

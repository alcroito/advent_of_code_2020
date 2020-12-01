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

// fn solve_p2() {
    
// }

#[test]
fn test_p1() {
    const TARGET_SUM:i64 = 2020;
    assert_eq!(find_two_numbers_sum(TARGET_SUM, &[1721, 979, 366, 299, 675, 1456]), Some(TwoNums(1721, 299)));
    assert_eq!(get_two_numbers_sum_and_product(TARGET_SUM, &[1721, 979, 366, 299, 675, 1456]).1, Some(514579));
    assert_eq!(get_two_numbers_sum_and_product(TARGET_SUM, &[500, 1520]).1, Some(760000));
}

fn main() {
    solve_p1();
    // solve_p2();
}

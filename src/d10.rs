use advent::helpers;
use anyhow::{Context, Result};
use itertools::Itertools;

type Adapters = Vec<i64>;

fn parse_jolt_adapters(i: &str) -> Adapters {
    helpers::lines_to_longs(i.trim())
}

fn prepare_jolt_adapters(adapters: Adapters) -> Adapters {
    // Prepend 0 and append max_jolt + 3.
    let final_device = *adapters.iter().max().expect("No max number") + 3;
    let mut adapters = std::iter::once(0)
        .chain(adapters.into_iter())
        .chain(std::iter::once(final_device))
        .collect::<Adapters>();
    // Sort for pair-wise iteration.
    adapters.sort_unstable();
    adapters
}

fn compute_jolt_differences(adapters: Adapters) -> (i64, i64) {
    let adapters = prepare_jolt_adapters(adapters);
    let diff = adapters
        .iter()
        .tuple_windows()
        .fold((0, 0), |mut diff, (a, b)| {
            let jolt_diff = b - a;
            match jolt_diff {
                1 => diff.0 += 1,
                3 => diff.1 += 1,
                _ => unreachable!(),
            };
            diff
        });
    diff
}

fn compute_adapter_arrangement_count(adapters: Adapters) -> i64 {
    let adapters = prepare_jolt_adapters(adapters);
    let final_device = adapters.iter().max().expect("No max number");
    let mut adapter_path_counter = adapters
        .iter()
        .cloned()
        .map(|a| (a, 0))
        .collect::<std::collections::HashMap<i64, i64>>();
    if let Some(v) = adapter_path_counter.get_mut(&0) {
        *v = 1;
    }

    let counter = adapters
        .iter()
        .skip(1)
        .fold(adapter_path_counter, |mut counter, adapter| {
            counter.insert(
                *adapter,
                (1..=3)
                    .into_iter()
                    .map(|delta| {
                        let input_adapter = adapter - delta;
                        counter.get(&input_adapter).unwrap_or(&0)
                    })
                    .sum::<i64>(),
            );
            counter
        });
    counter[final_device]
}

fn solve_p1() -> Result<()> {
    let input = helpers::get_data_from_file_res("d10").context("Coudn't read file contents.")?;
    let result = compute_jolt_differences(parse_jolt_adapters(&input));
    let result = result.0 * result.1;
    println!(
        "The number of 1-jolt differences multplied by 3-jolt differences is: {}",
        result
    );
    Ok(())
}

fn solve_p2() -> Result<()> {
    let input = helpers::get_data_from_file_res("d10").context("Coudn't read file contents.")?;
    let result = compute_adapter_arrangement_count(parse_jolt_adapters(&input));
    println!(
        "The total number of distinct ways the adapters can be arranged in is: {}",
        result
    );
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
16
10
15
5
1
11
7
19
6
12
4        
";
        let result = compute_jolt_differences(parse_jolt_adapters(input));
        assert_eq!(result.0, 7);
        assert_eq!(result.1, 5);

        let input = "
28
33
18
42
31
14
46
20
48
47
24
23
49
45
19
38
39
11
1
32
25
35
8
17
7
9
4
2
34
10
3        
";
        let result = compute_jolt_differences(parse_jolt_adapters(input));
        assert_eq!(result.0, 22);
        assert_eq!(result.1, 10);
    }

    #[test]
    fn test_p2() {
        let input = "
16
10
15
5
1
11
7
19
6
12
4
    ";
        let result = compute_adapter_arrangement_count(parse_jolt_adapters(input));
        assert_eq!(result, 8);

        let input = "
28
33
18
42
31
14
46
20
48
47
24
23
49
45
19
38
39
11
1
32
25
35
8
17
7
9
4
2
34
10
3
    ";
        let result = compute_adapter_arrangement_count(parse_jolt_adapters(input));
        assert_eq!(result, 19208);
    }
}

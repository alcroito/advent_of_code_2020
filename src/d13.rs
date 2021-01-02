use advent::helpers;
use anyhow::{Context, Result};
use itertools::Itertools;

fn parse_bus_id_and_minutes(s: &str) -> (u64, Vec<Option<u64>>) {
    let mut lines = s.trim().lines();
    let target_timestamp = lines
        .next()
        .expect("No first line")
        .parse::<u64>()
        .expect("Invalid initial timestamp");
    let ids = lines
        .next()
        .expect("No bus ids")
        .split(',')
        .map(|id| {
            if id == "x" {
                None
            } else {
                Some(id.parse::<u64>().expect("Invalid bus id"))
            }
        })
        .collect_vec();
    (target_timestamp, ids)
}

fn find_bus_id_and_minutes(s: &str) -> u64 {
    let (target_timestamp, bus_ids) = parse_bus_id_and_minutes(s);
    let bus_ids = bus_ids.into_iter().filter_map(|i| i).collect_vec();
    let bus_period_ids_and_departure_times = bus_ids
        .iter()
        .map(|bus_id| {
            let (period_id, rem) = num_integer::div_rem(target_timestamp, *bus_id);
            let period_id = if rem > 0 { period_id + 1 } else { period_id };
            let departure_time = period_id * bus_id;

            (*bus_id, period_id, departure_time)
        })
        .collect_vec();

    let (min_bus_id, _bus_period_id, departure_time) = bus_period_ids_and_departure_times
        .iter()
        .min_by(|x, y| x.2.cmp(&y.2))
        .expect("No minimum bus id");

    (departure_time - target_timestamp) * min_bus_id
}

fn find_earliest_magic_timestamp(s: &str, start_min_timestamp: u64) -> u64 {
    let (_, buses) = parse_bus_id_and_minutes(s);
    let buses = buses
        .into_iter()
        .enumerate()
        .filter_map(|(delta, maybe_id)| maybe_id.map(|frequency| (delta, frequency)))
        .collect_vec();
    println!("buses {:?}", buses);
    let mut timestamp: u64 = start_min_timestamp;
    let mut repeating_bus_period_so_far = buses[0].1;
    for (t_delta, bus_frequency) in buses.iter().skip(1) {
        loop {
            let possible_bus_departure_ts = timestamp + *t_delta as u64;
            if possible_bus_departure_ts % bus_frequency == 0 {
                break;
            }
            timestamp += repeating_bus_period_so_far;
        }
        repeating_bus_period_so_far *= bus_frequency;
    }
    timestamp
}

fn solve_p1() -> Result<()> {
    let input = helpers::get_data_from_file_res("d13").context("Coudn't read file contents.")?;
    let result = find_bus_id_and_minutes(&input);
    println!(
        "The bus id multiplied by the number of minutes is: {}",
        result
    );
    Ok(())
}

fn solve_p2() -> Result<()> {
    let input = helpers::get_data_from_file_res("d13").context("Coudn't read file contents.")?;
    let result = find_earliest_magic_timestamp(&input, 100000000000000);
    println!(
        "The earliest timestamp with the magic property is: {}",
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
        let input = "939
7,13,x,x,59,x,31,19";
        let result = find_bus_id_and_minutes(input);
        assert_eq!(result, 295);
    }

    #[test]
    fn test_p2() {
        let input = "939\n7,13,x,x,59,x,31,19";
        let result = find_earliest_magic_timestamp(input, 0);
        assert_eq!(result, 1068781);

        let input = "939\n67,7,59,61";
        let result = find_earliest_magic_timestamp(input, 0);
        assert_eq!(result, 754018);

        let input = "939\n67,x,7,59,61";
        let result = find_earliest_magic_timestamp(input, 0);
        assert_eq!(result, 779210);

        let input = "939\n67,7,x,59,61";
        let result = find_earliest_magic_timestamp(input, 0);
        assert_eq!(result, 1261476);

        let input = "939\n1789,37,47,1889";
        let result = find_earliest_magic_timestamp(input, 0);
        assert_eq!(result, 1202161486);
    }
}

use advent::helpers;
use anyhow::{Context, Result};
use itertools::Itertools;

fn find_bus_id_and_minutes(s: &str) -> u64 {
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
        .filter_map(|id| {
            if id == "x" {
                None
            } else {
                Some(id.parse::<u64>().expect("Invalid bus id"))
            }
        })
        .collect_vec();

    let bus_journey_counts_and_departure_times = ids
        .iter()
        .map(|bus_id| {
            let (journey_count, rem) = num_integer::div_rem(target_timestamp, *bus_id);
            let journey_count = if rem > 0 {
                journey_count + 1
            } else {
                journey_count
            };
            let departure_time = journey_count * bus_id;

            (*bus_id, journey_count, departure_time)
        })
        .collect_vec();
    let (min_bus_id, bus_journey_count, departure_timestamp) =
        bus_journey_counts_and_departure_times
            .iter()
            .min_by(|x, y| x.2.cmp(&y.2))
            .expect("No minimum bus id");
    println!("Bus ids: {:?}", ids);
    println!(
        "Bus jouney counts:   {:?}",
        bus_journey_counts_and_departure_times
    );
    println!("Min bus id:             {}", min_bus_id);
    println!("Bus journey count:      {}", bus_journey_count);
    println!("Min bus departure time: {}", departure_timestamp);
    println!("Target  departure time: {}", target_timestamp);
    println!(
        "Departure time diff   : {}",
        departure_timestamp - target_timestamp
    );
    (departure_timestamp - target_timestamp) * min_bus_id
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
    let _input = helpers::get_data_from_file_res("d13").context("Coudn't read file contents.")?;
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
        let _input = "939
7,13,x,x,59,x,31,19";
    }
}

use advent::helpers;
use std::error::Error;

type BoxedError = Box<dyn Error + Send + Sync>;
type Res = Result<u8, BoxedError>;

fn row_op_to_binary(c: &u8) -> Res {
    match *c as char {
        'F' => Ok(b'0'),
        'B' => Ok(b'1'),
        _ => Err(From::from(format!("Invalid row op: '{}'", *c as char))),
    }
}

fn col_op_to_binary(c: &u8) -> Res {
    match *c as char {
        'L' => Ok(b'0'),
        'R' => Ok(b'1'),
        _ => Err(From::from(format!("Invalid col op: '{}'", *c as char))),
    }
}

fn decode_string<F>(s: &str, r: std::ops::Range<usize>, op_mapper: F) -> Res
where
    F: FnMut(&u8) -> Res,
{
    let binary_vec = s.as_bytes()[r]
        .iter()
        .map(op_mapper)
        .collect::<Result<Vec<_>, _>>()?;
    let binary_string = std::str::from_utf8(&binary_vec)?;
    let decoded_number = u8::from_str_radix(binary_string, 2)?;
    Ok(decoded_number)
}

fn boarding_pass_to_seat_id(s: &str) -> Result<(u32, u32, u32), BoxedError> {
    let row = decode_string(s, 0..7, row_op_to_binary)? as u32;
    let column = decode_string(s, 7..10, col_op_to_binary)? as u32;
    Ok((row * 8 + column, row, column))
}

#[test]
fn test_p1() {
    assert_eq!(
        boarding_pass_to_seat_id("FBFBBFFRLR").ok(),
        Some((357, 44, 5))
    );
    assert_eq!(
        boarding_pass_to_seat_id("BFFFBBFRRR").ok(),
        Some((567, 70, 7))
    );
    assert_eq!(
        boarding_pass_to_seat_id("FFFBBBFRRR").ok(),
        Some((119, 14, 7))
    );
    assert_eq!(
        boarding_pass_to_seat_id("BBFFBBFRLL").ok(),
        Some((820, 102, 4))
    );
}

fn until_err<T, E>(err: &mut &mut Result<(), E>, item: Result<T, E>) -> Option<T> {
    match item {
        Ok(item) => Some(item),
        Err(e) => {
            **err = Err(e);
            None
        }
    }
}

fn solve_p1() -> Result<u32, BoxedError> {
    let data = helpers::get_data_from_file("d5").ok_or("Coudn't read file contents.")?;
    // https://morestina.net/blog/1607/fallible-iteration
    let mut err = Ok(());
    let max_seat_id = data
        .split_ascii_whitespace()
        .map(|s| boarding_pass_to_seat_id(s))
        .scan(&mut err, until_err)
        .map(|i| i.0)
        .max()
        .ok_or("No seats provided.")?;
    err?;
    println!("Max seat id is: {}", max_seat_id);
    Ok(max_seat_id)
}

fn solve_p2() -> Result<u32, BoxedError> {
    let data = helpers::get_data_from_file("d5").ok_or("Coudn't read file contents.")?;
    let mut err = Ok(());
    let mut seat_vec = data
        .split_ascii_whitespace()
        .map(|s| boarding_pass_to_seat_id(s))
        .scan(&mut err, until_err)
        .map(|i| i.0)
        .collect::<std::vec::Vec<u32>>();
    err?;
    seat_vec.sort_unstable();
    let seat_set: std::collections::HashSet<&u32> = seat_vec.iter().collect();
    let missing_seats = seat_vec
        .iter()
        .rev()
        .skip(1)
        .rev()
        .fold(vec![], |mut missing_seats, i| {
            if !seat_set.contains(&(*i + 1)) {
                missing_seats.push(Some(*i + 1));
            }
            missing_seats
        });
    assert_eq!(missing_seats.len(), 1);
    let needle_seat = missing_seats[0];
    println!("Your seat id is: {:?}", needle_seat);
    assert_eq!(needle_seat, Some(743));
    needle_seat.ok_or_else(|| "No empty seat found.".into())
}

fn handle_error<T>(r: Result<T, BoxedError>) {
    match r {
        Ok(_) => (),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn main() {
    handle_error(solve_p1());
    handle_error(solve_p2());
}

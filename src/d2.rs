use advent::helpers;
extern crate nom;

fn char_to_index(c: &u8) -> usize {
    (c - 'a' as u8) as usize
}

#[derive(Debug)]
struct ValidityArgs<'a> {
    password: &'a str,
    needle: u8,
    low: i32,
    high: i32,
}

fn parse_password_and_policy(i: &str) -> nom::IResult<&str, ValidityArgs> {
    // 2-9 c: ccccccccc
    let (i, low) =
        nom::combinator::map_res(nom::character::complete::digit1, |s: &str| s.parse::<i32>())(i)?;
    let (i, _) = nom::character::complete::char('-')(i)?;
    let (i, high) =
        nom::combinator::map_res(nom::character::complete::digit1, |s: &str| s.parse::<i32>())(i)?;
    let (i, _) = nom::character::complete::space1(i)?;
    let (i, needle) = nom::combinator::map(
        nom::character::complete::satisfy(|c| nom::character::is_alphabetic(c as u8)),
        |c: char| c as u8,
    )(i)?;
    let (i, _) = nom::character::complete::char(':')(i)?;
    let (i, _) = nom::character::complete::space1(i)?;
    let (i, password) = nom::character::complete::alpha1(i)?;
    return Ok((
        i,
        ValidityArgs {
            password,
            needle,
            low,
            high,
        },
    ));
}

fn is_password_valid(args: &ValidityArgs) -> bool {
    let ValidityArgs {
        password,
        needle,
        low,
        high,
    } = *args;
    let mut char_counter: [i32; 26] = [0; 26];
    password
        .as_bytes()
        .iter()
        .for_each(|c| char_counter[char_to_index(c)] += 1);
    let count = char_counter[char_to_index(&needle) as usize];
    if count >= low && count <= high {
        true
    } else {
        false
    }
}

fn solve_p1() {
    let mut valid_passwords: i64 = 0;
    let data = helpers::get_data_from_file("d2").expect("Coudn't read file contents.");
    data.lines().for_each(|line| {
        let (_, args) = parse_password_and_policy(line).expect("Couldn't parse line");
        let is_valid = is_password_valid(&args);
        if is_valid {
            valid_passwords += 1;
        }
    });
    println!("The number of valid passwords is: {}", valid_passwords);
}

#[test]
fn test_p1() {
    let cases = vec![
        (
            ValidityArgs {
                password: "abcde",
                needle: 'a' as u8,
                low: 1,
                high: 3,
            },
            true,
        ),
        (
            ValidityArgs {
                password: "cdefg",
                needle: 'b' as u8,
                low: 1,
                high: 3,
            },
            false,
        ),
        (
            ValidityArgs {
                password: "ccccccccc",
                needle: 'c' as u8,
                low: 2,
                high: 9,
            },
            true,
        ),
    ];

    cases.iter().for_each(|(c, expected_result)| {
        let is_valid = is_password_valid(c);
        assert_eq!(is_valid, *expected_result);
    });
}

fn main() {
    solve_p1();
}

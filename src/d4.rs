use advent::helpers;
use itertools::Itertools;

#[allow(unused)]
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_while},
    character::complete::{alphanumeric0, alphanumeric1, multispace0, multispace1, one_of},
    combinator::{all_consuming, map, map_res, not, opt, recognize},
    multi::{many0, many1, separated_list0},
    sequence::{pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

type Fields<'a> = std::collections::HashMap<&'a str, &'a str>;
type Passports<'a> = Vec<Passport<'a>>;

#[derive(Debug, Default)]
struct Passport<'a> {
    fields: Fields<'a>,
}

fn parse_field(i: &str) -> IResult<&str, (&str, &str)> {
    let (i, (name, value)) = separated_pair(
        alphanumeric1,
        tag(":"),
        recognize(pair(alt((alphanumeric1, tag("#"))), alphanumeric0)),
    )(i)?;
    // println!("n: {} v: {}", name, value);
    Ok((i, (name, value)))
}

fn parse_fields(i: &str) -> IResult<&str, Fields> {
    let (i, fields) = separated_list0(one_of(" \n"), parse_field)(i)?;
    // println!("fields: {:?}", fields);
    Ok((i, fields.into_iter().collect()))
}

fn parse_passports_whole(i: &str) -> IResult<&str, Passports> {
    // println!(">> whole input {:?}", i);
    let (i, passports) =
        separated_list0(tag("\n\n"), map(parse_fields, |fields| Passport { fields }))(i)?;
    // println!("passports: {:?}", passports);
    Ok((i, passports))
}

impl<'a> Passport<'a> {
    fn is_valid(&self) -> bool {
        let needles = vec!["byr", "iyr", "eyr", "hgt", "hcl", "ecl", "pid"];
        // println!("validating fields: {:?}", self.fields);
        needles
            .iter()
            .all(|needle| self.fields.contains_key(needle))
    }
}

impl<'a> From<&'a str> for Passport<'a> {
    fn from(s: &'a str) -> Self {
        Passport {
            fields: parse_fields(s).unwrap().1,
        }
    }
}

#[allow(unused)]
fn parse_passports_approach1(input: &str) -> Passports {
    let passports: Passports = input
        .trim()
        .split_terminator("\n\n")
        .map(str::trim)
        .map_into()
        .collect();
    // println!("{:?}", a);
    passports
}

fn parse_passports_approach2(input: &str) -> Passports {
    let passports: Passports = parse_passports_whole(input.trim()).unwrap().1;
    // println!("{:?}", passports);
    passports
}

fn count_valid_passports(passports: &Passports) -> usize {
    passports.iter().filter(|p| p.is_valid()).count()
}

fn solve_p1() {
    let data = helpers::get_data_from_file("d4").expect("Coudn't read file contents.");
    let passports = parse_passports_approach2(&data);
    let valid_count = count_valid_passports(&passports);
    println!("The number of valid passports is: {}", valid_count);
}

#[test]
fn test_p1() {
    let input = "
ecl:gry pid:860033327 eyr:2020 hcl:#fffffd
byr:1937 iyr:2017 cid:147 hgt:183cm

iyr:2013 ecl:amb cid:350 eyr:2023 pid:028048884
hcl:#cfa07d byr:1929

hcl:#ae17e1 iyr:2013
eyr:2024
ecl:brn pid:760753108 byr:1931
hgt:179cm

hcl:#cfa07d eyr:2025 pid:166559648
iyr:2011 ecl:brn hgt:59in    
    ";
    let passports = parse_passports_approach2(input);
    let valid_count = count_valid_passports(&passports);
    assert_eq!(valid_count, 2);
}

fn main() {
    solve_p1();
}

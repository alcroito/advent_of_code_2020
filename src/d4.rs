use advent::helpers;
use boolinator::Boolinator;
use itertools::{Either, Itertools};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while_m_n},
    character::complete::{alphanumeric0, alphanumeric1, digit1, one_of},
    combinator::{all_consuming, map, map_res, recognize},
    multi::separated_list0,
    sequence::{pair, preceded, separated_pair},
    IResult,
};

// Below are all articles and references used to come up with the final (not perfect)
// design of a custom error that integrates with nom::error::ParseError,
// nom::error::FromExternalError, std::num::ParseIntError, and all the magic needed
// to use the errors within nom's parser combinators.
// Specifically the denomify and map_err
// combinators.
// https://nick.groenen.me/posts/rust-error-handling/
// https://blog.burntsushi.net/rust-error-handling/
// https://doc.rust-lang.org/rust-by-example/error/iter_result.html#collect-all-valid-values-and-failures-with-partition
//   used together with itertools::Itertools::partion_map
// https://github.com/Geal/nom/blob/master/doc/error_management.md
// https://github.com/Geal/nom/blob/master/doc/nom_recipes.md#implementing-fromstr
// https://github.com/search?p=1&q=FromExternalError&type=Code
// https://github.com/JokeZhang/fuchsia/blob/d6e9dea8dca7a1c8fa89d03e131367e284b30d23/src/devices/bind/debugger/src/parser_common.rs#L93
// https://github.com/JokeZhang/fuchsia/blob/d6e9dea8dca7a1c8fa89d03e131367e284b30d23/src/devices/bind/debugger/src/parser_common.rs#L282
// https://github.com/KeyMaster-/rustak/blob/c0a1e30443ac8e194ca507541b794768cb6d7525/src/parse/mod.rs#L173
// https://github.com/kdl-org/kdl-rs/blob/ef630148fcd49bfd680609f9cb22f88520e922c6/src/error.rs#L29
// https://github.com/orogene/orogene/blob/5b9cf45a6711c64a0b212eb680fadf948733753c/crates/oro-package-spec/src/error.rs#L111
//
// There are still some rough edges and TODOs left, but that's for the future
// - Possibly fix inconsistency of storing input either in nom_error or in the input member
// - Allow wrapping a generic error via boxing (probably by introducing a kind that wraps an 'anyhow' error)
// - Figure out if it makes sense to add a context method and thus implement nom::error::ContextError
// - Get rid of the unreachable!() calls when unwrapping a nom::Err<T>.
// - Using nom::error::ErrorVerbose error within the customer error is clunky, but was a result
//   of experimentation to try and use nom::error::convert_error to get nicer backtrace info.
// - Not all functions are generic enough (don't use trait bounds) and hardcode the custom error type and parser input type.
// - Figure out how to use nom::Err::map() in map_err instead of explicit pattern matching.

#[derive(Debug, PartialEq)]
enum PassportParseErrorKind {
    InvalidYearNotWithinRange(u16, u16, u16),
    InvalidYearStringToIntConversion(std::num::ParseIntError),
    InvalidHeightUnit(),
    InvalidHeightNotWithinRange(u16, u16, u16, LengthUnit),
    InvalidHeightStringToIntConversion(std::num::ParseIntError),
    InvalidHairColor(),
    InvalidEyeColor(),
    InvalidPassportId(),
    InvalidCountryId(),
    // For generic string errors.
    Other(String),
    // Generic Nom error, will 99% of time be mapped to a more specific error above.
    Nom,
}

#[derive(Debug)]
struct PassportParseError<I> {
    kind: PassportParseErrorKind,
    input: Option<I>,
    nom_error: Option<nom::error::VerboseError<I>>,
}
type PassportParseErrorExact<'a> = PassportParseError<&'a str>;

impl<I> PassportParseError<I> {
    fn new(input: Option<I>, kind: PassportParseErrorKind) -> Self {
        PassportParseError {
            input,
            nom_error: None,
            kind,
        }
    }

    fn new_other(input: Option<I>, str: String) -> Self {
        PassportParseError::new(input, PassportParseErrorKind::Other(str))
    }
}

impl<'a, I> nom::error::ParseError<I> for PassportParseError<I> {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        PassportParseError {
            input: None,
            nom_error: Some(nom::error::VerboseError {
                errors: vec![(input, nom::error::VerboseErrorKind::Nom(kind))],
            }),
            kind: PassportParseErrorKind::Nom,
        }
    }

    fn append(input: I, kind: nom::error::ErrorKind, mut other: Self) -> Self {
        other.nom_error = match other.nom_error {
            Some(mut nom_error) => {
                nom_error
                    .errors
                    .push((input, nom::error::VerboseErrorKind::Nom(kind)));
                Some(nom_error)
            }
            None => Some(nom::error::VerboseError {
                errors: vec![(input, nom::error::VerboseErrorKind::Nom(kind))],
            }),
        };
        other
    }

    fn from_char(input: I, c: char) -> Self {
        PassportParseError {
            input: None,
            nom_error: Some(nom::error::VerboseError {
                errors: vec![(input, nom::error::VerboseErrorKind::Char(c))],
            }),
            kind: PassportParseErrorKind::Nom,
        }
    }
}

impl<'a> nom::error::FromExternalError<&'a str, PassportParseErrorExact<'a>>
    for PassportParseErrorExact<'a>
{
    fn from_external_error(
        _input: &'a str,
        _kind: nom::error::ErrorKind,
        e: PassportParseErrorExact<'a>,
    ) -> Self {
        e
    }
}

/// Wraps a parser and replaces its error by calling the given 'f' function.
/// nom provides map_res which is similar to Result::and_then,
/// but doesn't provide a map_err similar to Result::map_err.
fn map_err<'a, O, Parser, ErrorMapper, InitialError>(
    mut parser: Parser,
    mut f: ErrorMapper,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, PassportParseErrorExact>
where
    Parser: FnMut(&'a str) -> IResult<&'a str, O, InitialError>,
    ErrorMapper: FnMut(InitialError) -> PassportParseErrorExact<'a>,
{
    move |input: &'a str| {
        parser(input).map_err(|e| match e {
            nom::Err::Error(e2) => nom::Err::Error(f(e2)),
            nom::Err::Failure(e2) => nom::Err::Failure(f(e2)),
            nom::Err::Incomplete(_) => {
                unreachable!("Parser should never generate Incomplete errors")
            }
        })
    }
}

fn denomify_error<'a, ErrorMapper>(
    mut error_mapper: ErrorMapper,
) -> impl FnMut(PassportParseErrorExact<'a>) -> PassportParseErrorExact<'a>
where
    ErrorMapper: FnMut() -> PassportParseErrorKind,
{
    move |mut e: PassportParseErrorExact<'a>| {
        if PassportParseErrorKind::Nom == e.kind {
            e.kind = error_mapper()
        }
        e
    }
}

fn extract_nom_error<E>(err: nom::Err<E>) -> E {
    match err {
        nom::Err::Failure(x) | nom::Err::Error(x) => x,
        nom::Err::Incomplete(_) => unreachable!(),
    }
}

#[derive(Debug, PartialEq, Eq)]
enum LengthUnit {
    Centimetre,
    Inch,
}

#[derive(Debug, PartialEq, Eq)]
enum EyeColor {
    Amber,
    Blue,
    Brown,
    Gray,
    Green,
    Hazel,
    Other,
}

#[derive(Debug, PartialEq, Eq)]
struct PassportFieldValue<T>(T);

#[derive(Debug)]
enum PassportField {
    BirthYear(PassportFieldValue<u16>),
    IssueYear(PassportFieldValue<u16>),
    ExpirationYear(PassportFieldValue<u16>),
    Height(PassportFieldValue<(u16, LengthUnit)>),
    HairColor(PassportFieldValue<String>),
    EyeColor(PassportFieldValue<EyeColor>),
    PassportId(PassportFieldValue<String>),
    CountryId(PassportFieldValue<Option<String>>),
}

const BIRTH_YEAR_KEY: &str = "byr";
const ISSUE_YEAR_KEY: &str = "iyr";
const EXPIRATION_YEAR_KEY: &str = "eyr";
const HEIGHT_KEY: &str = "hgt";
const HAIR_COLOR_KEY: &str = "hcl";
const EYE_COLOR_KEY: &str = "ecl";
const PASSPORT_ID_KEY: &str = "pid";
const COUNTRY_ID_KEY: &str = "cid";

impl PassportField {
    fn parse_from_field_type_and_value<'a>(
        (field_type, i): (&'a str, &'a str),
    ) -> IResult<&'a str, PassportField, PassportParseErrorExact> {
        match field_type {
            BIRTH_YEAR_KEY => PassportField::parse_birth_year(i)
                .map(|(i, year)| (i, PassportField::BirthYear(year))),
            ISSUE_YEAR_KEY => PassportField::parse_issue_year(i)
                .map(|(i, year)| (i, PassportField::IssueYear(year))),
            EXPIRATION_YEAR_KEY => PassportField::parse_expiration_year(i)
                .map(|(i, year)| (i, PassportField::ExpirationYear(year))),
            HEIGHT_KEY => {
                PassportField::parse_height(i).map(|(i, year)| (i, PassportField::Height(year)))
            }
            HAIR_COLOR_KEY => PassportField::parse_hair_color(i)
                .map(|(i, color)| (i, PassportField::HairColor(color))),
            EYE_COLOR_KEY => PassportField::parse_eye_color(i)
                .map(|(i, color)| (i, PassportField::EyeColor(color))),
            PASSPORT_ID_KEY => PassportField::parse_passport_id(i)
                .map(|(i, id)| (i, PassportField::PassportId(id))),
            COUNTRY_ID_KEY => {
                PassportField::parse_country_id(i).map(|(i, id)| (i, PassportField::CountryId(id)))
            }
            _ => Err(nom::Err::Error(PassportParseError::new_other(
                Some(field_type),
                format!("Invalid field key: {}", field_type),
            ))),
        }
    }

    #[allow(unused)]
    fn parse(i: &str) -> IResult<&str, PassportField, PassportParseErrorExact> {
        let (_i, field_and_value) = parse_field_permissive(i)?;
        PassportField::parse_from_field_type_and_value(field_and_value)
    }

    fn parse_birth_year(
        i: &str,
    ) -> IResult<&str, PassportFieldValue<u16>, PassportParseErrorExact> {
        PassportField::parse_year(i, 1920..=2002)
    }

    fn parse_issue_year(
        i: &str,
    ) -> IResult<&str, PassportFieldValue<u16>, PassportParseErrorExact> {
        PassportField::parse_year(i, 2010..=2020)
    }

    fn parse_expiration_year(
        i: &str,
    ) -> IResult<&str, PassportFieldValue<u16>, PassportParseErrorExact> {
        PassportField::parse_year(i, 2020..=2030)
    }

    fn parse_year(
        i: &str,
        range: std::ops::RangeInclusive<usize>,
    ) -> IResult<&str, PassportFieldValue<u16>, PassportParseErrorExact> {
        let parse_digits = take_while_m_n(4, 4, |c: char| c.is_ascii_digit());
        map_res(parse_digits, |digits| {
            u16::from_str_radix(digits, 10)
                .map_err(|e| PassportParseError {
                    input: Some(i),
                    nom_error: None,
                    kind: PassportParseErrorKind::InvalidYearStringToIntConversion(e),
                })
                .and_then(|year| {
                    Some(year)
                        .filter(|&y| y >= *range.start() as u16 && y <= *range.end() as u16)
                        .map(PassportFieldValue)
                        .ok_or_else(|| PassportParseError {
                            input: Some(i),
                            nom_error: None,
                            kind: PassportParseErrorKind::InvalidYearNotWithinRange(
                                year,
                                *range.start() as u16,
                                *range.end() as u16,
                            ),
                        })
                })
        })(i)
    }

    fn parse_height(
        i: &str,
    ) -> IResult<&str, PassportFieldValue<(u16, LengthUnit)>, PassportParseErrorExact> {
        let (i, height) = map_res(digit1, |digits: &str| {
            digits.parse::<u16>().map_err(|e| PassportParseError {
                input: Some(i),
                nom_error: None,
                kind: PassportParseErrorKind::InvalidHeightStringToIntConversion(e),
            })
        })(i)?;

        map_res(
            map_err(
                alt((tag("in"), tag("cm"))),
                denomify_error(PassportParseErrorKind::InvalidHeightUnit),
            ),
            move |unit_type| match unit_type {
                "in" => {
                    if height >= 59 && height <= 76 {
                        Ok(PassportFieldValue((height, LengthUnit::Inch)))
                    } else {
                        Err(PassportParseError {
                            input: Some(i),
                            nom_error: None,
                            kind: PassportParseErrorKind::InvalidHeightNotWithinRange(
                                height,
                                59,
                                76,
                                LengthUnit::Inch,
                            ),
                        })
                    }
                }
                "cm" => {
                    if height >= 150 && height <= 193 {
                        Ok(PassportFieldValue((height, LengthUnit::Centimetre)))
                    } else {
                        Err(PassportParseError {
                            input: Some(i),
                            nom_error: None,
                            kind: PassportParseErrorKind::InvalidHeightNotWithinRange(
                                height,
                                150,
                                193,
                                LengthUnit::Centimetre,
                            ),
                        })
                    }
                }
                _ => unreachable!(),
            },
        )(i)
    }

    fn parse_hair_color(
        i: &str,
    ) -> IResult<&str, PassportFieldValue<String>, PassportParseErrorExact> {
        map_err(
            map(
                preceded(
                    tag("#"),
                    take_while_m_n(6, 6, |c: char| c.is_ascii_hexdigit()),
                ),
                |s: &str| PassportFieldValue(s.to_owned()),
            ),
            denomify_error(PassportParseErrorKind::InvalidHairColor),
        )(i)
    }

    fn parse_eye_color(
        i: &str,
    ) -> IResult<&str, PassportFieldValue<EyeColor>, PassportParseErrorExact> {
        map_res(
            map_err(
                alt((
                    tag("amb"),
                    tag("blu"),
                    tag("brn"),
                    tag("gry"),
                    tag("grn"),
                    tag("hzl"),
                    tag("oth"),
                )),
                denomify_error(PassportParseErrorKind::InvalidEyeColor),
            ),
            |color: &str| match color {
                "amb" => Ok(PassportFieldValue(EyeColor::Amber)),
                "blu" => Ok(PassportFieldValue(EyeColor::Blue)),
                "brn" => Ok(PassportFieldValue(EyeColor::Brown)),
                "gry" => Ok(PassportFieldValue(EyeColor::Gray)),
                "grn" => Ok(PassportFieldValue(EyeColor::Green)),
                "hzl" => Ok(PassportFieldValue(EyeColor::Hazel)),
                "oth" => Ok(PassportFieldValue(EyeColor::Other)),
                _ => unreachable!(),
            },
        )(i)
    }

    fn parse_passport_id(
        i: &str,
    ) -> IResult<&str, PassportFieldValue<String>, PassportParseErrorExact> {
        map_err(
            all_consuming(map(
                take_while_m_n(9, 9, |c: char| c.is_ascii_digit()),
                |digits: &str| PassportFieldValue(digits.to_owned()),
            )),
            denomify_error(PassportParseErrorKind::InvalidPassportId),
        )(i)
    }

    fn parse_country_id(
        i: &str,
    ) -> IResult<&str, PassportFieldValue<Option<String>>, PassportParseErrorExact> {
        map_err(
            map(alphanumeric1, |country_id: &str| {
                PassportFieldValue(Some(country_id.to_owned()))
            }),
            denomify_error(PassportParseErrorKind::InvalidCountryId),
        )(i)
    }
}

type Fields<'a> = std::collections::HashMap<&'a str, &'a str>;
type Passports<'a> = Vec<Passport<'a>>;
type PassportResults<'a> = (Vec<Passport<'a>>, Vec<PassportParseErrorExact<'a>>);

type StrictFields = Vec<PassportField>;

#[derive(Debug)]
struct Passport<'a> {
    fields: Fields<'a>,
}

fn passport_has_valid_field_names<V>(field_map: &std::collections::HashMap<&str, V>) -> bool {
    let needles = vec!["byr", "iyr", "eyr", "hgt", "hcl", "ecl", "pid"];
    needles.iter().all(|needle| field_map.contains_key(needle))
}

impl<'a> Passport<'a> {
    fn new(fields: Fields<'a>) -> Self {
        Passport { fields }
    }

    fn is_valid(&self) -> bool {
        passport_has_valid_field_names(&self.fields)
    }
}

impl<'a> std::convert::TryFrom<&'a str> for Passport<'a> {
    type Error = PassportParseErrorExact<'a>;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        let res = parse_fields_permissive(s);
        res.map(|(_, fields)| Passport::new(fields))
            .map_err(extract_nom_error)
    }
}

#[derive(Debug)]
struct StrictPassport {
    fields: StrictFields,
}

impl StrictPassport {
    fn from_permissive<'a>(p: Passport<'a>) -> Result<Self, Vec<PassportParseErrorExact<'a>>> {
        let (fields, errors): (std::collections::HashMap<_, _>, Vec<_>) = p
            .fields
            .into_iter()
            .map(|field_and_value| {
                (
                    field_and_value.0,
                    PassportField::parse_from_field_type_and_value(field_and_value),
                )
            })
            .partition_map(|(field_key, res)| match res {
                Ok(v) => Either::Left((field_key, v.1)),
                Err(e) => Either::Right(extract_nom_error(e)),
            });

        passport_has_valid_field_names(&fields).as_result_from(
            || StrictPassport {
                fields: fields.into_iter().map(|(_, field)| field).collect(),
            },
            || errors,
        )
    }
}

fn parse_field_permissive(i: &str) -> IResult<&str, (&str, &str), PassportParseErrorExact> {
    let (i, (name, value)) = separated_pair(
        alphanumeric1,
        tag(":"),
        recognize(pair(alt((alphanumeric1, tag("#"))), alphanumeric0)),
    )(i)?;
    // println!("n: {} v: {}", name, value);
    Ok((i, (name, value)))
}

fn parse_fields_permissive<'a>(
    i: &'a str,
) -> IResult<&'a str, Fields<'a>, PassportParseErrorExact> {
    let (i, fields) = separated_list0(one_of(" \n"), parse_field_permissive)(i)?;
    // println!("fields: {:?}", fields);
    Ok((i, fields.into_iter().collect()))
}

fn parse_passports_whole(i: &str) -> IResult<&str, Passports, PassportParseErrorExact> {
    // println!(">> whole input {:?}", i);
    let (i, passports) =
        separated_list0(tag("\n\n"), map(parse_fields_permissive, Passport::new))(i)?;
    // println!("passports: {:?}", passports);
    Ok((i, passports))
}

#[allow(unused)]
fn parse_passports_approach1(input: &str) -> PassportResults {
    let (passports, errors): (Vec<_>, Vec<_>) = input
        .trim()
        .split_terminator("\n\n")
        .map(str::trim)
        .map(std::convert::TryInto::<Passport>::try_into)
        .partition_map(|r| match r {
            Ok(v) => Either::Left(v),
            Err(v) => Either::Right(v),
        });
    // println!("{:?}", passports);
    (passports, errors)
}

fn parse_passports_approach2(input: &str) -> PassportResults {
    let passports_result = parse_passports_whole(input.trim())
        .map(|(_, p_vec)| p_vec)
        .map_err(extract_nom_error);

    let (passports, errors) = match passports_result {
        Ok(p_vec) => (p_vec, vec![]),
        Err(e) => (vec![], vec![e]),
    };

    // println!("{:?}", passports);
    (passports, errors)
}

fn count_permissive_passports(passports: &[Passport]) -> usize {
    passports.iter().filter(|p| p.is_valid()).count()
}

fn count_valid_passports_with_valid_fields(input: &str) -> usize {
    let (passports, errors) = parse_passports_approach2(input);

    if !errors.is_empty() {
        println!(
            "Encountered errors when parsing permissive passports: {:?}",
            errors
        );
        return 0;
    }

    let (passports, strict_errors): (Vec<_>, Vec<_>) = passports
        .into_iter()
        .map(StrictPassport::from_permissive)
        .partition_map(|r| match r {
            Ok(v) => Either::Left(v),
            Err(v) => Either::Right(v),
        });

    strict_errors.into_iter().for_each(|one_passport_errors| {
        eprintln!("Strict passport parsing failed: {:?}", one_passport_errors);
    });
    passports.len()
}

fn solve_p1() {
    let data = helpers::get_data_from_file("d4").expect("Coudn't read file contents.");
    let (passports, errors) = parse_passports_approach2(&data);

    if !errors.is_empty() {
        println!(
            "Encountered errors when parsing permissive passports: {:?}",
            errors
        );
    } else {
        let valid_count = count_permissive_passports(&passports);
        println!("Permissive passport count is: {}", valid_count);
    }
}

fn solve_p2() {
    let data = helpers::get_data_from_file("d4").expect("Coudn't read file contents.");
    let len = count_valid_passports_with_valid_fields(&data);
    println!("Strict passport count is: {}", len);
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
    let valid_count = count_permissive_passports(&passports.0);
    assert_eq!(valid_count, 2);
}

#[test]
fn test_p2() {
    let input = "2002";
    let field = PassportField::parse_birth_year(input).unwrap().1;
    assert_eq!(field, PassportFieldValue::<u16>(2002));

    let input = "2003";
    let field = PassportField::parse_birth_year(input);
    assert!(field.is_err());

    let input = "60in";
    let PassportFieldValue::<(u16, LengthUnit)>((length, unit)) =
        PassportField::parse_height(input).unwrap().1;
    assert_eq!(length, 60);
    assert_eq!(unit, LengthUnit::Inch);

    let input = "190cm";
    let PassportFieldValue::<(u16, LengthUnit)>((length, unit)) =
        PassportField::parse_height(input).unwrap().1;
    assert_eq!(length, 190);
    assert_eq!(unit, LengthUnit::Centimetre);

    let input = "190in";
    let field = PassportField::parse_height(input);
    assert!(field.is_err());

    let input = "190";
    let field = PassportField::parse_height(input);
    assert!(field.is_err());

    let input = "brn";
    let field = PassportField::parse_eye_color(input).unwrap().1;
    assert_eq!(field, PassportFieldValue::<EyeColor>(EyeColor::Brown));

    let input = "wat";
    let field = PassportField::parse_eye_color(input);
    assert!(field.is_err());

    let input = "000000001";
    let field = PassportField::parse_passport_id(input).unwrap().1;
    assert_eq!(field, PassportFieldValue::<String>("000000001".to_owned()));

    let input = "0123456789";
    let field = PassportField::parse_passport_id(input);
    assert!(field.is_err());

    let input = "
eyr:1972 cid:100
hcl:#18171d ecl:amb hgt:170 pid:186cm iyr:2018 byr:1926

iyr:2019
hcl:#602927 eyr:1967 hgt:170cm
ecl:grn pid:012533040 byr:1946

hcl:dab227 iyr:2012
ecl:brn hgt:182cm pid:021572410 eyr:2020 byr:1992 cid:277

hgt:59cm ecl:zzz
eyr:2038 hcl:74454a iyr:2023
pid:3556412378 byr:2007
    ";
    let len = count_valid_passports_with_valid_fields(input);
    assert_eq!(len, 0);

    let input = "
pid:087499704 hgt:74in ecl:grn iyr:2012 eyr:2030 byr:1980
hcl:#623a2f

eyr:2029 ecl:blu cid:129 byr:1989
iyr:2014 pid:896056539 hcl:#a97842 hgt:165cm

hcl:#888785
hgt:164cm byr:2001 iyr:2015 cid:88
pid:545766238 ecl:hzl
eyr:2022

iyr:2010 hgt:158cm hcl:#b6652a ecl:blu byr:1944 eyr:2021 pid:093154719
    ";
    let len = count_valid_passports_with_valid_fields(input);
    assert_eq!(len, 4);
}

fn main() {
    solve_p1();
    solve_p2();
}

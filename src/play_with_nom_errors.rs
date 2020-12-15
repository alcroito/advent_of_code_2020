#[derive(Debug)]
enum MyParseError<'a, I> {
    InvalidFieldValue(&'a str),
    FailedConversion(String),
    Nom(nom::error::VerboseError<I>), // Nom(I, nom::error::ErrorKind),
}

impl<'a, I> MyParseError<'a, I> {
    fn invalid_field_value(field_val: &'a str) -> Self {
        MyParseError::InvalidFieldValue(field_val)
    }

    fn failed_conversion<E>(conversion_error: E) -> Self
    where
        E: std::fmt::Display,
    {
        MyParseError::FailedConversion(conversion_error.to_string())
    }
}

impl<'a, I> nom::error::ParseError<I> for MyParseError<'a, I> {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        MyParseError::Nom(nom::error::VerboseError {
            errors: vec![(input, nom::error::VerboseErrorKind::Nom(kind))],
        })
    }

    fn append(input: I, kind: nom::error::ErrorKind, mut other: Self) -> Self {
        match other {
            MyParseError::Nom(nom::error::VerboseError { ref mut errors }) => {
                errors.push((input, nom::error::VerboseErrorKind::Nom(kind)))
            }
            _ => (),
        };
        other
    }

    fn from_char(input: I, c: char) -> Self {
        MyParseError::Nom(nom::error::VerboseError {
            errors: vec![(input, nom::error::VerboseErrorKind::Char(c))],
        })
    }
}

type Res<'a> = nom::IResult<&'a str, &'a str, MyParseError<'a, &'a str>>;
type ResNoInput<'a> = Result<i8, nom::Err<MyParseError<'a, &'a str>>>;

fn parse_field(i: &str) -> Res {
    let (i, _) = nom::bytes::complete::tag("hello:")(i)?;
    let (i, value) = nom::character::complete::digit1(i)?;
    Ok((i, value))
}

fn parse_field_verbose_error(i: &str) -> nom::IResult<&str, &str, nom::error::VerboseError<&str>> {
    let (i, _) = nom::bytes::complete::tag("hello:")(i)?;
    let (i, value) = nom::character::complete::digit1(i)?;
    Ok((i, value))
}

fn convert_field_to_int(res: Res) -> ResNoInput {
    res.and_then(|(_, val)| {
        val.parse::<i8>()
            .map_err(|e| nom::Err::Error(MyParseError::failed_conversion(e)))
            .and_then(|val_int| {
                Some(val_int)
                    .filter(|&x| x < 4)
                    .ok_or_else(|| nom::Err::Error(MyParseError::invalid_field_value(val)))
            })
    })
}

fn handle_maybe_int(initial_input: &str, field: ResNoInput) {
    field
        .map(|val| println!("val is: {}", val))
        .map_err(|err| match err {
            nom::Err::Error(MyParseError::Nom(verbose_error)) => eprintln!(
                "Error was:\n{}",
                nom::error::convert_error(initial_input, verbose_error)
            ),
            _ => eprintln!("Error was: {}", err),
        })
        .ok();
}

fn play() {
    let my_err = MyParseError::<&str>::invalid_field_value("fieldbomp_val");
    println!("{:?}", my_err);

    let input = "hello:1";
    let parsed = parse_field(input);
    let maybe_int = convert_field_to_int(parsed);
    handle_maybe_int(input, maybe_int);

    let input = "hello:5";
    let parsed = parse_field(input);
    let maybe_int = convert_field_to_int(parsed);
    handle_maybe_int(input, maybe_int);

    let input = "hello:123456789";
    let parsed = parse_field(input);
    let maybe_int = convert_field_to_int(parsed);
    handle_maybe_int(input, maybe_int);

    let input = "hello:fa";
    let parsed = parse_field(input);
    let maybe_int = convert_field_to_int(parsed);
    handle_maybe_int(input, maybe_int);

    let input = "hello:fa";
    let parsed = parse_field_verbose_error(input);
    match parsed {
        Err(nom::Err::Error(e)) => {
            eprintln!("errors were: {}", nom::error::convert_error(input, e));
        }
        _ => (),
    }
}

fn play2() {
    // Initial code
    let parsed = "123"
        .parse::<i32>()
        .map_err(|_| "parse error")
        .and_then(|val| {
            if val < 4 {
                Ok(val)
            } else {
                Err("Not a good number")
            }
        });
    println!("{:?}", parsed);

    // Desired code
    // let parsed =  "123".parse::<i32>()
    // .map_err(|_| "parse error")
    // .conditional_and_then(|x| x < 4, |x| Ok(x), |_| Err("not a good number"));
    // println!("{:?}", parsed);

    // Proposed code
    let _parsed = "123"
        .parse::<i32>()
        .map_err(|_| "parse error")
        .and_then(|val| {
            Some(val)
                .filter(|&x| x < 4)
                .ok_or_else(|| "Not a good number")
        });

    // Advised code.
    let _parsed = match "123".parse::<i32>() {
        Ok(val) if val < 4 => Ok(val),
        Ok(_) => Err("Not a good number"),
        Err(_) => Err("parse error"),
    };
}

fn main() {
    play2();
    play();
}

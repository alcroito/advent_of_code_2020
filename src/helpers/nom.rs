#[derive(Debug)]

// A type that wraps nom's nom::error::VerboseError and implements
// ParseError, FromExternalError, and ContextError.
// Can be used in nom's IResult.
// Advantage, can be used with nom::error::convert_error.
// Disadvantage, leaks memory when appending errors converted from external errors
// due to limitation in VerboseErrorKind.
pub struct NomError<I>(nom::error::VerboseError<I>);

impl<I> NomError<I> {
    pub fn into_verbose_string(self, i: I) -> String
    where
        I: core::ops::Deref<Target = str>,
    {
        nom::error::convert_error(i, self.0)
    }

    pub fn into_anyhow(self, i: I) -> anyhow::Error
    where
        I: core::ops::Deref<Target = str>,
    {
        anyhow::anyhow!("{}", self.into_verbose_string(i))
    }
}

impl<I> nom::error::ParseError<I> for NomError<I> {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        Self(nom::error::VerboseError {
            errors: vec![(input, nom::error::VerboseErrorKind::Nom(kind))],
        })
    }

    fn append(input: I, kind: nom::error::ErrorKind, mut other: Self) -> Self {
        other
            .0
            .errors
            .push((input, nom::error::VerboseErrorKind::Nom(kind)));
        other
    }

    fn from_char(input: I, c: char) -> Self {
        Self(nom::error::VerboseError {
            errors: vec![(input, nom::error::VerboseErrorKind::Char(c))],
        })
    }
}

impl<I, E> nom::error::FromExternalError<I, E> for NomError<I>
where
    E: std::fmt::Display + 'static,
{
    fn from_external_error(input: I, _kind: nom::error::ErrorKind, e: E) -> Self
    where
        E: std::fmt::Display + 'static,
    {
        // WARNING: this leaks memory.
        // There's no other way to convert a String to a &'static str.
        // And unfortunately nom::error::VerboseErrorKind::Context doesn't take an owned String.
        // The proper way would be to re-implement our own error kind and VerboserError that can store a String,
        // but then we can't use nom::error::convert_error :(
        // So we'd have to copy-paste that function as well.
        let leaked_external_error = Box::leak(format!("{}", e).into_boxed_str());
        Self(nom::error::VerboseError {
            errors: vec![(
                input,
                nom::error::VerboseErrorKind::Context(leaked_external_error),
            )],
        })
    }
}

impl<I> nom::error::ContextError<I> for NomError<I> {
    fn add_context(input: I, ctx: &'static str, mut other: Self) -> Self {
        other
            .0
            .errors
            .push((input, nom::error::VerboseErrorKind::Context(ctx)));
        other
    }
}

// Modified copy of nom's dbg_dmp that works with &str instead of &[u8].
pub fn dbg_dmp<'a, F, O, E: core::fmt::Debug>(
    mut f: F,
    context: &'static str,
) -> impl FnMut(&'a str) -> nom::IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> nom::IResult<&'a str, O, E>,
{
    move |i: &'a str| match f(i) {
        Err(e) => {
            println!("{}: Error({:?}) at:\n{}", context, e, i);
            Err(e)
        }
        a => a,
    }
}

// A type similar to NomError. Mostyle copy-pastes and reimplements
// most of nom::error::VerboseError with addition to allow holding non
// static owned context Strings, to avoid leaks.

#[derive(Clone, Debug, PartialEq)]
pub enum NomError2Kind {
    Context(String),
    Char(char),
    Nom(nom::error::ErrorKind),
}

#[derive(Clone, Debug, PartialEq)]
pub struct NomError2<I> {
    pub errors: std::vec::Vec<(I, NomError2Kind)>,
}

impl<I> nom::error::ParseError<I> for NomError2<I> {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        Self {
            errors: vec![(input, NomError2Kind::Nom(kind))],
        }
    }

    fn append(input: I, kind: nom::error::ErrorKind, mut other: Self) -> Self {
        other.errors.push((input, NomError2Kind::Nom(kind)));
        other
    }

    fn from_char(input: I, c: char) -> Self {
        Self {
            errors: vec![(input, NomError2Kind::Char(c))],
        }
    }
}

impl<I, E> nom::error::FromExternalError<I, E> for NomError2<I>
where
    E: std::fmt::Display + 'static,
{
    fn from_external_error(input: I, _kind: nom::error::ErrorKind, e: E) -> Self
    where
        E: std::fmt::Display + 'static,
    {
        Self {
            errors: vec![(input, NomError2Kind::Context(format!("{}", e)))],
        }
    }
}

impl<I> nom::error::ContextError<I> for NomError2<I> {
    fn add_context(input: I, ctx: &'static str, mut other: Self) -> Self {
        other
            .errors
            .push((input, NomError2Kind::Context(ctx.to_owned())));
        other
    }
}

impl<I: std::fmt::Display> std::fmt::Display for NomError2<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Parse error:")?;
        for (input, error) in &self.errors {
            match error {
                NomError2Kind::Nom(e) => writeln!(f, "{:?} at: {}", e, input)?,
                NomError2Kind::Char(c) => writeln!(f, "expected '{}' at: {}", c, input)?,
                NomError2Kind::Context(s) => writeln!(f, "in section '{}', at: {}", s, input)?,
            }
        }

        Ok(())
    }
}

impl<I: std::fmt::Display> NomError2<I> {
    pub fn into_verbose_string(self, i: I) -> String
    where
        I: core::ops::Deref<Target = str>,
    {
        convert_error(i, self)
    }

    pub fn into_anyhow(self, i: I) -> anyhow::Error
    where
        I: core::ops::Deref<Target = str>,
    {
        anyhow::anyhow!("{}", self.into_verbose_string(i))
    }
}

pub fn convert_error<I: core::ops::Deref<Target = str>>(input: I, e: NomError2<I>) -> String {
    use nom::Offset;
    use std::fmt::Write;

    let mut result = String::new();

    for (i, (substring, kind)) in e.errors.iter().enumerate() {
        let offset = input.offset(substring);

        if input.is_empty() {
            match kind {
                NomError2Kind::Char(c) => {
                    write!(&mut result, "{}: expected '{}', got empty input\n\n", i, c)
                }
                NomError2Kind::Context(s) => {
                    write!(&mut result, "{}: in {}, got empty input\n\n", i, s)
                }
                NomError2Kind::Nom(e) => {
                    write!(&mut result, "{}: in {:?}, got empty input\n\n", i, e)
                }
            }
        } else {
            let prefix = &input.as_bytes()[..offset];

            // Count the number of newlines in the first `offset` bytes of input
            let line_number = bytecount::count(prefix, b'\n') + 1;

            // println!("substring:'{}'\nprefix:'{}'", substring.to_string(), std::str::from_utf8(prefix).unwrap());
            // Find the line that includes the subslice:
            // Find the *last* newline before the substring starts
            let line_begin = prefix
                .iter()
                .rev()
                .position(|&b| b == b'\n')
                .map(|pos| offset - pos)
                .unwrap_or(0);

            // Find the full line after that newline
            let line = input[line_begin..]
                .lines()
                .next()
                .unwrap_or(&input[line_begin..])
                .trim_end();

            // The (1-indexed) column number is the offset of our substring into that line
            let column_number = line.offset(substring) + 1;

            // Get the before and after lines, to provide some additional context.
            // let before_line_pos = prefix[..line_begin-1].iter().rev().position(|&b| b == b'\n').map(|pos| offset - pos - 1).unwrap_or(0);
            // let before_line = input[before_line_pos..line_begin].lines().next().unwrap_or(&input[before_line_pos..]);
            // println!("line_begin: {}\nbefore_line_pos: {}\noffset: {}\nrange: {:?}", line_begin, before_line_pos, offset, (..line_begin));
            // println!("before_line: {}", before_line);
            // println!("char at before_line_pos: {}", input[before_line_pos..before_line_pos+1].to_string());

            // let mut after_line_iter = input[line_begin..].lines();
            // after_line_iter.next();
            // let after_line = after_line_iter.next().unwrap_or("").trim_end();
            // println!("after_line: {}", after_line);

            match kind {
                NomError2Kind::Char(c) => {
                    if let Some(actual) = substring.chars().next() {
                        write!(
                            &mut result,
                            "{i}: at line {line_number}:\n\
                 {line}\n\
                 {caret:>column$}\n\
                 expected '{expected}', found {actual}\n\n",
                            i = i,
                            line_number = line_number,
                            line = line,
                            caret = '^',
                            column = column_number,
                            expected = c,
                            actual = actual,
                        )
                    } else {
                        write!(
                            &mut result,
                            "{i}: at line {line_number}:\n\
                 {line}\n\
                 {caret:>column$}\n\
                 expected '{expected}', got end of input\n\n",
                            i = i,
                            line_number = line_number,
                            line = line,
                            caret = '^',
                            column = column_number,
                            expected = c,
                        )
                    }
                }
                NomError2Kind::Context(s) => write!(
                    &mut result,
                    "{i}: at line {line_number}, in {context}:\n\
               {line}\n\
               {caret:>column$}\n\n",
                    i = i,
                    line_number = line_number,
                    context = s,
                    line = line,
                    caret = '^',
                    column = column_number,
                ),
                NomError2Kind::Nom(e) => write!(
                    &mut result,
                    "{i}: at line {line_number}, in {nom_err:?}:\n\
             {line}\n\
             {caret:>column$}\n\n",
                    i = i,
                    line_number = line_number,
                    nom_err = e,
                    line = line,
                    caret = '^',
                    column = column_number,
                    // before_line = before_line,
                    // after_line = after_line,
                ),
            }
        }
        // Because `write!` to a `String` is infallible, this `unwrap` is fine.
        .unwrap();
    }

    result
}

#[allow(dead_code)]
pub mod helpers {
    use std::fs;
    pub fn get_data_from_file(name: &str) -> Option<String> {
        let path = format!("data/{}.txt", name);

        match fs::read_to_string(path) {
            Ok(s) => Some(s),
            Err(e) => {
                println!("Error reading data: {}", e);
                None
            }
        }
    }

    pub fn get_data_from_file_res(name: &str) -> std::io::Result<String> {
        let path = format!("data/{}.txt", name);
        fs::read_to_string(path)
    }

    pub fn lines_to_longs(contents: &str) -> Vec<i64> {
        let mut ints = Vec::new();
        for s in contents.split_ascii_whitespace() {
            ints.push(s.parse::<i64>().unwrap());
        }
        ints
    }

    pub fn ints_to_longs(ints: &[i32]) -> Vec<i64> {
        let longs: Vec<i64>;
        longs = ints.iter().map(|&x| x as i64).collect();
        longs
    }

    pub fn csv_string_to_ints(contents: &str) -> Vec<i32> {
        let mut ints = Vec::new();
        for s in contents.split(',') {
            ints.push(s.trim().parse::<i32>().unwrap());
        }
        ints
    }

    pub mod nom {
        #[derive(Debug)]
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
    }
}

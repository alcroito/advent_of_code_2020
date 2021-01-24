use nom::bytes::complete::tag;
use nom::error::Error;

struct NomParserWrapper<F> {
    f: F
}

impl<F> NomParserWrapper<F> {
    fn new(f: F) -> Self {
        NomParserWrapper {
            f
        }
    }
}

type InputType<'a> = &'a str;
type DynParser<'a> = dyn FnMut(InputType<'a>) -> nom::IResult<InputType<'a>, InputType<'a>> + 'a;
type BoxedParser<'a> = Box<DynParser<'a>>;
type NomParserWrapperExact1<'a> = NomParserWrapper<BoxedParser<'a>>;

type DynParser2<'a> = dyn FnMut(InputType<'a>) -> InputType<'a> + 'static;
type BoxedParser2<'a> = Box<DynParser2<'a>>;
type NomParserWrapperExact2<'a> = NomParserWrapper<BoxedParser2<'a>>;

fn make_nom_wrapper_1<'a>() -> NomParserWrapperExact1<'a> {
    let c = tag::<&str, &str, Error<&str>>("c");
    let b: BoxedParser = Box::new(c);
    NomParserWrapperExact1::new(b)
}

fn get_fn_mut() -> impl FnMut(InputType) -> InputType {
    |a: InputType| a
}


fn make_nom_wrapper_2<'a>() -> NomParserWrapperExact2<'a> {
    let c = get_fn_mut();
    let b: BoxedParser2 = Box::new(c);
    NomParserWrapperExact2::new(b)
}

fn main() {
    let _n_1 = make_nom_wrapper_1().f;
    let _n_2 = make_nom_wrapper_2().f;
}
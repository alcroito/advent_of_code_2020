use nom::bytes::complete::tag;
use nom::error::Error;
use nom::IResult;
use nom::Parser;
struct NomParserWrapper<F> {
    f: F,
}

impl<F> NomParserWrapper<F> {
    fn new(f: F) -> Self {
        NomParserWrapper { f }
    }
}

struct NomParserWrapper3<'a, 'b> {
    f: BoxedParser3<'a, 'b>,
}

impl<'a, 'b> NomParserWrapper3<'a, 'b> {
    fn new(f: BoxedParser3<'a, 'b>) -> Self {
        NomParserWrapper3 { f }
    }
}

impl<'a, 'b> Parser<InputType<'a>, InputType<'a>, NomError<'a>> for NomParserWrapper3<'a, 'b> {
    fn parse(&mut self, i: InputType<'a>) -> IResult<InputType<'a>, InputType<'a>, NomError<'a>> {
        self.f.parse(i)
    }
}

type InputType<'a> = &'a str;
type NomError<'a> = Error<InputType<'a>>;

type DynParser<'a> = dyn FnMut(InputType<'a>) -> nom::IResult<InputType<'a>, InputType<'a>> + 'a;
type BoxedParser<'a> = Box<DynParser<'a>>;
type NomParserWrapperExact1<'a> = NomParserWrapper<BoxedParser<'a>>;

type DynParser2<'a> = dyn FnMut(InputType<'a>) -> InputType<'a> + 'static;
type BoxedParser2<'a> = Box<DynParser2<'a>>;
type NomParserWrapperExact2<'a> = NomParserWrapper<BoxedParser2<'a>>;

type DynParser3<'a, 'b> =
    dyn FnMut(InputType<'a>) -> nom::IResult<InputType<'a>, InputType<'a>> + 'b;
type BoxedParser3<'a, 'b> = Box<DynParser3<'a, 'b>>;

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

fn make_nom_wrapper_3() -> NomParserWrapper3<'static, 'static> {
    let c = tag::<&str, &str, Error<&str>>("c");
    let b: BoxedParser3 = Box::new(c);
    NomParserWrapper3::new(b)
}

fn make_nom_wrapper_4<'a: 'b, 'b>() -> NomParserWrapper3<'a, 'b> {
    let c = tag::<&str, &str, Error<&str>>("c");
    let b: BoxedParser3 = Box::new(c);
    NomParserWrapper3::new(b)
}

// Weird behavior inbound.
// Generally the &str input parameter's 'a lifetime should be decoupled from the lifetime of the FnMut object trait 'b.
// Unfortunately rust has some weird behavior / bug as described in https://github.com/rust-lang/rust/issues/79415
// Returning an impl FnMut results in it being bound by the lifetimes of any generic T parameters encountered in the closure's
// input (or output I guess).
// This means that the returned dyn FnMut lifetime is bound to it's input lifetime.
// Which is why the following 2 signatures do not compile
// while the others do.
// fn make_nom_wrapper5<'a, 'b>() -> NomParserWrapper<Box<dyn FnMut(&'a str) -> nom::IResult<&'a str, &'a str> + 'b>> {     // <-- broken
// fn make_nom_wrapper5<'a>() -> NomParserWrapper<Box<dyn FnMut(&'a str) -> nom::IResult<&'a str, &'a str> + 'static>> {    // <-- broken
// fn make_nom_wrapper5<'a: 'b, 'b>() -> NomParserWrapper<Box<dyn FnMut(&'a str) -> nom::IResult<&'a str, &'a str> + 'b>> { // <-- works
// fn make_nom_wrapper5<'a>() -> NomParserWrapper<Box<dyn FnMut(&'a str) -> nom::IResult<&'a str, &'a str> + 'a>> {         // <-- works
//     let c = |input| {
//         let tag_contents: &'static str = "c";
//         tag(tag_contents)(input)
//     };
//     let b = Box::new(c);
//     NomParserWrapper::new(b)
// }

// A workaround I found is to wrap the returned impl FnMut into another closure, as demonstrated below.
// That gets rid of the 'a: 'b dependency and seems to work fine. There's probably some perfomance penalty,
// but meh.
type NomParserWrapperExact5<'a, 'b> =
    NomParserWrapper<Box<dyn FnMut(&'a str) -> nom::IResult<&'a str, &'a str> + 'b>>;
fn make_nom_wrapper5<'a, 'b>() -> NomParserWrapperExact5<'a, 'b> {
    let c = |input| tag("c")(input);
    let b = Box::new(c);
    NomParserWrapper::new(b)
}

fn main() {
    let _n_1 = make_nom_wrapper_1().f;
    let _n_2 = make_nom_wrapper_2().f;
    let mut p_3 = make_nom_wrapper_3();
    let message = "hello";
    let res = p_3.f.parse(message);
    dbg!(res.is_err());
    let _p_4 = make_nom_wrapper_4();
    let mut p_5 = make_nom_wrapper5();
    let message = "hello";
    let res = p_5.f.parse(message);
    dbg!(res.is_err());
}

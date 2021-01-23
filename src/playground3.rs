use std::cell::RefCell;
use std::rc::Rc;

use nom::combinator::recognize;
use nom::character::complete::char as nom_char;
use nom::sequence::pair;
use nom::error::Error;
use nom::Parser;
use nom::IResult;

type InputType<'a> = &'a str;
type NomError<'a> = Error<InputType<'a>>;
type DynParser<'a> = dyn FnMut(InputType<'a>) -> IResult<InputType<'a>, InputType<'a>> + 'a;
type BoxedParser<'a> = Box<DynParser<'a>>;

struct NomParserWrapper<F> {
    f: Rc<RefCell<F>>
}

impl<F> Clone for NomParserWrapper<F> {
    fn clone(&self) -> Self {
        NomParserWrapper {
            f: self.f.clone(),
        }
    }
}

impl<F> NomParserWrapper<F> {
    fn new(f: F) -> Self {
        NomParserWrapper {
            f: Rc::new(RefCell::new(f))
        }
    }
}

impl<I, O1, E, F: Parser<I, O1, E>> Parser<I, O1, E> for NomParserWrapper<F> {
    fn parse(&mut self, i: I) -> IResult<I, O1, E> {
        self.f.borrow_mut().parse(i)
    }
}

fn nom_parser_wrapper_new<'a, F>(f: F) -> NomParserWrapper<F>
where F: Parser<InputType<'a>, InputType<'a>, NomError<'a>> {
    NomParserWrapper::new(f)
}

fn main() {
    let char_parser = nom_char('3');
    let char_parser_rc = NomParserWrapper::new(char_parser);
    let recognize_parser = recognize(char_parser_rc);
    let mut recognize_parser_rc = nom_parser_wrapper_new(recognize_parser);
    let mut recognize_parser_rc_2 = recognize_parser_rc.clone();

    let parsed = recognize_parser_rc.parse("3");
    dbg!(parsed.is_ok());
    let parsed = recognize_parser_rc_2.parse("a");
    dbg!(parsed.is_err());

    let char_parser = || recognize(nom_char('c'));
    let first_wrapper = NomParserWrapper::new(char_parser());
    let mut parser_accumulator: BoxedParser = Box::new(recognize(first_wrapper));
    for _ in 1..3 {
        parser_accumulator = Box::new(recognize(pair(parser_accumulator, char_parser())));
    }
    let mut compunded_parser = nom_parser_wrapper_new(parser_accumulator);
    let parsed = compunded_parser.parse("cccc");
    dbg!(parsed.is_ok());
}
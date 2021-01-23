use advent::helpers;
use anyhow::{Context, Result};
use derive_more::Display;
use itertools::Itertools;
use std::cell::RefCell;
use std::rc::Rc;

use nom::{Parser, combinator::recognize, multi::count};
use nom::IResult;

type RuleId = usize;
type Subrule = Vec<usize>;
type SubruleRef<'a> = &'a [usize];
type Message = String;
type Messages = Vec<Message>;
type SubruleAlternatives = Vec<Subrule>;
type Rules = std::collections::HashMap<RuleId, Rule>;

type InputType<'a> = &'a str;
type NomError<'a> = nom::error::Error<InputType<'a>>;
type DynParser<'a> = dyn FnMut(InputType<'a>) -> nom::IResult<InputType<'a>, InputType<'a>> + 'a;
type BoxedParser<'a> = Box<DynParser<'a>>;
type NomParserWrapperExact<'a> = NomParserWrapper<BoxedParser<'a>>;
// type NomParserMap<'a> = std::collections::HashMap<RuleId, BoxedParser<'a>>;

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

#[derive(Debug, Display)]
enum Rule {
    #[display(fmt = "{}", _0)]
    Char(char),
    #[display(fmt = "{:?}", _0)]
    Alternatives(SubruleAlternatives),
}

fn parse_rules_and_messages(s: &str) -> (Rules, Messages) {
    let s = s.trim();
    let sep = "\n\n";
    let rules_end_idx = s.find(sep).unwrap();
    let rules_str = &s[0..rules_end_idx];
    let messages_str = &s[rules_end_idx + sep.len()..];

    let rules = rules_str
        .lines()
        .map(|l| {
            let mut l = l.split(':');
            let rule_id = l.next().unwrap().parse::<usize>().unwrap();

            let mut subrules_it = l.next().unwrap().trim().split(" | ");
            let subrule_1_str = subrules_it.next().unwrap();
            let final_rule;
            if subrule_1_str.starts_with('"') {
                final_rule = Some(Rule::Char(subrule_1_str.chars().nth(1).unwrap()));
            } else {
                let subrule_collector = |sub_str: &str| {
                    sub_str
                        .split_whitespace()
                        .map(|c| c.parse::<usize>().unwrap())
                        .collect_vec()
                };
                let mut subrules = vec![subrule_collector(subrule_1_str)];
                for subrule in subrules_it {
                    subrules.push(subrule_collector(subrule));
                }
                final_rule = Some(Rule::Alternatives(subrules));
            }
            (rule_id, final_rule.unwrap())
        })
        .collect::<Rules>();

    let messages = messages_str
        .lines()
        .map(|l| l.to_string())
        .collect::<Messages>();

    (rules, messages)
}

fn add_loop_to_rules(r: &mut Rules) {
    if let Some(v) = r.get_mut(&8) {
        *v = Rule::Alternatives(vec![vec![42], vec![1000]])
    };
    r.insert(1000, Rule::Alternatives(vec![vec![42, 8]]));

    if let Some(v) = r.get_mut(&11) {
        *v = Rule::Alternatives(vec![vec![42, 31], vec![42, 11, 31]])
    };

    // // Expand '11 -> 42 31 | 42 11 31' rule into a limited number
    // // of non-looping alternatives up to a specific k.
    // // r.insert(2000, Rule::Alternatives(vec![vec![42, 31]]));
    // let mut r11_alternatives = vec![];
    // for i in 1..=4 {
    //     let first = std::iter::repeat(42).take(i);
    //     let second = std::iter::repeat(31).take(i);
    //     r11_alternatives.push(first.chain(second).collect_vec());
    // }
    // if let Some(v) = r.get_mut(&11) {
    //     *v = Rule::Alternatives(r11_alternatives)
    // };
}

fn is_message_valid_wrapper(m: &str, r: &Rules) -> bool {
    let mut rules_applied = Vec::<usize>::new();
    let (matches, final_idx) = is_message_valid(m, r, 0, 0, 0, &mut rules_applied);
    if !matches {
        return false;
    }
    final_idx == m.len()
}

fn alt_count(r: &Rules, rule_idx: usize) -> usize {
    let rule = &r[&rule_idx];
    match rule {
        Rule::Char(..) => 1,
        Rule::Alternatives(subrules) => subrules.len(),
    }
}

fn check_if_matches_subrule(
    m: &str,
    r: &Rules,
    subrule: &[usize],
    message_idx: usize,
    rules_applied: &mut Vec<usize>,
) -> (bool, usize) {
    let subrule_cartesian_iter = subrule.iter().map(|rule_idx| 0..alt_count(r, *rule_idx)).multi_cartesian_product();

    // let blob = subrule_cartesian_iter.clone().collect_vec();
    // let a = 4 + 4;
    
    for candidate_subrule_indices in subrule_cartesian_iter {
        let mut current_idx = message_idx;
        let rules_applied_copy = rules_applied.clone();
        let mut valid_cartesian_choice = true;

        for (new_rule_pos, new_rule_idx) in subrule.iter().enumerate() {
            let subrule_to_apply = candidate_subrule_indices[new_rule_pos];
            if new_rule_idx == &8 {
                // println!("looping 8");
            }
            if new_rule_idx == &11 {
                // println!("looping 11");
            }
            let (matches, returned_message_idx) = is_message_valid(m, r, current_idx, *new_rule_idx, subrule_to_apply, rules_applied);
            if matches {
                current_idx = returned_message_idx;
            } else {
                while rules_applied.len() != rules_applied_copy.len() {
                    rules_applied.pop();
                }
                valid_cartesian_choice = false;
                break;
            }
        }

        if valid_cartesian_choice {
            return (true, current_idx)
        }
    }
    (false, message_idx)
}

fn is_message_valid(m: &str, r: &Rules, message_idx: usize, rule_idx: usize, subrule_to_apply: usize, rules_applied: &mut Vec<usize>) -> (bool, usize) {
    let rule = &r[&rule_idx];

    // let rules_applied = format!("{},{}", rules_applied, rule_idx);
    rules_applied.push(rule_idx);
    // println!("m_i: {:2} {:2}:{}, \n  applied: {:?} len {}", message_idx, rule_idx, rule, rules_applied, rules_applied.len());

    if message_idx >= m.len() {
        // println!("m_i too long");
        rules_applied.pop();
        return (false, message_idx)
    }
    // println!("  match:   {}      m is: {}", m.chars().nth(message_idx).unwrap(), &m[0..message_idx]);

    let res = match rule {
        Rule::Char(c) => {
            let target_char = m.chars().nth(message_idx).unwrap();
            let matches = target_char == *c;
            let return_idx = if matches {
                message_idx + 1
            } else {
                message_idx
            };
            (matches, return_idx)
        }
        Rule::Alternatives(subrules) => {
            check_if_matches_subrule(m, r, &subrules[subrule_to_apply], message_idx, rules_applied)
        },
    };
    if !res.0 {
        rules_applied.pop();
    }
    // println!("  res  :   {}", res.0);
    res
}

fn build_nom_subrole_parser<'a>(r: &Rules, s: SubruleRef)
-> NomParserWrapperExact<'a>
    
{
    let mut subrule_it = s.iter();
    let first_rule_idx = subrule_it.next().unwrap();
    let mut first_p_boxed: BoxedParser = Box::new(recognize(build_regular_nom_parser(r, *first_rule_idx)));

    for new_rule_idx in subrule_it {
        let new_p_boxed = Box::new(recognize(build_regular_nom_parser(r, *new_rule_idx)));
        let sequenced = nom::sequence::pair(first_p_boxed, new_p_boxed);
        first_p_boxed = Box::new(recognize(sequenced));
    }
    nom_parser_wrapper_new(first_p_boxed)
}

fn build_nom_alternative_parser<'a>(r: &Rules, 
    new_alternative: SubruleRef, 
    first_alternative_boxed_p: BoxedParser<'a>, 
    ) 
    -> BoxedParser<'a>
    {
    let new_alternative_boxed_p: BoxedParser = Box::new(recognize(build_nom_subrole_parser(r, new_alternative)));
    let alted = nom::branch::alt((first_alternative_boxed_p, new_alternative_boxed_p));
    let alted: BoxedParser = Box::new(recognize(alted));
    alted
}

fn build_nom_parser_8(repeat_count: usize, p_42: NomParserWrapperExact)
-> NomParserWrapperExact
 {
    // Special case looping parser 8.
    let p = p_42;
    let p = recognize(p);
    let p = count(p, repeat_count);
    let p: BoxedParser = Box::new(recognize(p));
    nom_parser_wrapper_new(p)
}

fn build_nom_parser_11<'a>(repeat_count: usize, p_31: NomParserWrapperExact<'a>, p_42: NomParserWrapperExact<'a>)
-> NomParserWrapperExact<'a> {
    // Special case looping parser 11.
    let p_42 = p_42;
    let p_31 = p_31;

    let p_42 = count(p_42, repeat_count);
    let p_42 = recognize(p_42);

    let p_31 = count(p_31, repeat_count);
    let p_31 = recognize(p_31);

    let p = nom::sequence::pair(p_42, p_31);
    let p: BoxedParser = Box::new(recognize(p));

    nom_parser_wrapper_new(p)
}

fn build_regular_nom_parser<'a>(r: &Rules, rule_idx: usize) 
-> NomParserWrapperExact<'a>
{
    let rule = &r[&rule_idx];
    let res = match rule {
        Rule::Char(c) => {
            let p = nom::character::complete::char::<&str, nom::error::Error<&str>>(*c);
            let p: BoxedParser = Box::new(recognize(p));
            NomParserWrapper::new(p)
        }
        Rule::Alternatives(alternatives) => {
            let mut alternatives_it = alternatives.iter();
            let alternative = alternatives_it.next().unwrap();
            let mut first_alternative_boxed_p: BoxedParser = Box::new(recognize(build_nom_subrole_parser(r, alternative)));

            for new_alternative in alternatives_it {
                first_alternative_boxed_p = Box::new(recognize(build_nom_alternative_parser(r, new_alternative, first_alternative_boxed_p)));
            }
            NomParserWrapper::new(first_alternative_boxed_p)
        },
    };
    res
}

fn is_message_valid_using_nom<'a>(m: &'a str, p_31: NomParserWrapperExact<'a>, p_42: NomParserWrapperExact<'a>) -> bool

 {
    let repeat_cartesian_iter = vec![1..=5, 1..=5].into_iter().multi_cartesian_product();

    for repeat_counts in repeat_cartesian_iter {
        let mut nom_p_8 = build_nom_parser_8(repeat_counts[0], p_42.clone());
        let mut nom_p_11 = build_nom_parser_11(repeat_counts[1], p_31.clone(), p_42.clone());
        let res = nom_p_8.parse(m);
        let res = res.and_then(|(input, _output)|{
            // dbg!((&input, &_output));
            nom_p_11.parse(input)
        });
        // dbg!(&res);
        let res = res.map(|(input, _)| input.is_empty()).unwrap_or(false);
        // println!("is valid: {}", res);
        if res {
            return res
        }
    
    }

    false
}

// fn prepare_part2_sub_parsers(r: &Rules, nom_map: &mut NomParserMap) {
//     let p_31 = build_regular_nom_parser(r, 31);
//     let p_31 = Box::new(nom::combinator::recognize(p_31));
//     nom_map.insert(31, p_31);

//     let p_42 = build_regular_nom_parser(r, 42);
//     let p_42 = Box::new(nom::combinator::recognize(p_42));
//     nom_map.insert(42, p_42);
// }

fn count_valid_messages(s: &str) -> usize {
    let (rules, messages) = parse_rules_and_messages(s);
    messages
        .iter()
        .map(|m| is_message_valid_wrapper(m, &rules))
        .filter(|is_valid| *is_valid)
        .count()
}

fn count_valid_messages_p2(s: &str) -> usize {
    let (mut rules, messages) = parse_rules_and_messages(s);
    add_loop_to_rules(&mut rules);
    // let mut nom_map = NomParserMap::new();
    // prepare_part2_sub_parsers(&rules, &mut nom_map);

    let p_31: NomParserWrapperExact = build_regular_nom_parser(&rules, 31);
    let p_42: NomParserWrapperExact = build_regular_nom_parser(&rules, 42);
    
    dbg!(&messages[0]);
    messages
        .iter()
        .map(|m| {
            let v = is_message_valid_using_nom(m, p_31.clone(), p_42.clone());
            println!("m: {} valid: {}", m, v);
            v
        })
        .filter(|is_valid| *is_valid)
        .count()
}

fn solve_p1() -> Result<()> {
    let input = helpers::get_data_from_file_res("d19").context("Coudn't read file contents.")?;
    let result = count_valid_messages(&input);
    println!("The number of messages that match the rules is: {}", result);
    Ok(())
}

fn solve_p2() -> Result<()> {
    let input = helpers::get_data_from_file_res("d19").context("Coudn't read file contents.")?;
    let result = count_valid_messages_p2(&input);
    println!(
        "The number of messages that match the rules with loops is: {}",
        result
    );
    Ok(())
}

fn main() -> Result<()> {
    solve_p1().ok();
    solve_p2().ok();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p1() {
        macro_rules! test {
            ($expr: literal, $solution: expr) => {
                let input = $expr;
                assert_eq!(count_valid_messages(input), $solution)
            };
        }

        test!(
            r#"
0: 4 1 5
1: 2 3 | 3 2
2: 4 4 | 5 5
3: 4 5 | 5 4
4: "a"
5: "b"

ababbb
bababa
abbbab
aaabbb
aaaabbb"#,
            2
        );

        test!(
            r#"
0: 1 2
1: "a"
2: "b"

ab"#,
            1
        );
    }

    #[test]
    fn test_p2() {
        macro_rules! test {
            ($expr: literal, $solution: expr) => {
                let input = $expr;
                assert_eq!(count_valid_messages_p2(input), $solution)
            };
        }

        test!(
            r#"
42: 9 14 | 10 1
9: 14 27 | 1 26
10: 23 14 | 28 1
1: "a"
11: 42 31
5: 1 14 | 15 1
19: 14 1 | 14 14
12: 24 14 | 19 1
16: 15 1 | 14 14
31: 14 17 | 1 13
6: 14 14 | 1 14
2: 1 24 | 14 4
0: 8 11
13: 14 3 | 1 12
15: 1 | 14
17: 14 2 | 1 7
23: 25 1 | 22 14
28: 16 1
4: 1 1
20: 14 14 | 1 15
3: 5 14 | 16 1
27: 1 6 | 14 18
14: "b"
21: 14 1 | 1 14
25: 1 1 | 1 14
22: 14 14
8: 42
26: 14 22 | 1 20
18: 15 15
7: 14 5 | 1 21
24: 14 1

bbabbbbaabaabba
"#,
            1
        );

// bbabbbbaabaabba
// abbbbabbbbaaaababbbbbbaaaababb

// test!(
//     r#"
// 0: 8 11
// 8: 42 | 42 8
// 11: 42 31 | 42 11 31
// 42: 1 | 14
// 11: 1
// 31: 1
// 1: "a"
// 14: "b"

// bbbb"#,
//     1
// );

// test!(
//     r#"
// 8: 42 | 42 8
// 42: 1 | 14
// 11: 1
// 31: 1
// 1: "a"
// 14: "b"

// baa"#,
//     1
// );
    }
}

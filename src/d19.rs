use advent::helpers;
use anyhow::{Context, Result};
use derive_more::Display;
use itertools::Itertools;
use std::cell::RefCell;
use std::rc::Rc;

use nom::branch::alt;
use nom::sequence::pair;
use nom::IResult;
use nom::{combinator::recognize, multi::count, Parser};

type RuleId = usize;
type RuleSequence = Vec<RuleId>;
type RuleSequenceRef<'a> = &'a [RuleId];
type Message = String;
type Messages = Vec<Message>;
type RuleAlternatives = Vec<RuleSequence>;
type RulesMap = std::collections::HashMap<RuleId, Rule>;

type InputType<'a> = &'a str;
type NomError<'a> = nom::error::Error<InputType<'a>>;
type DynParser<'a, 't> =
    dyn FnMut(InputType<'a>) -> nom::IResult<InputType<'a>, InputType<'a>> + 't;
type BoxedParser<'a, 't> = Box<DynParser<'a, 't>>;
type NomParserWrapperExact<'a, 't> = NomParserWrapper<BoxedParser<'a, 't>>;
type NomParserMap<'a, 't> = std::collections::HashMap<RuleId, NomParserWrapperExact<'a, 't>>;

// This struct wraps a boxed nom parser and allows us to memoize certain
// prebuilt parsers in a hashmap. These can then be used to build further parsers
// as building blocks. Useful for part 2 when we build repeating parsers, and thus
// save a lot of time on not rebuilding the base parsers.
// The Rc is needed because the same base parser can be reused by a root parser.
// The RefCell is necessary because nom parsers need to be mut FnMut, so we need
// both sharing and interior mutability.
struct NomParserWrapper<F> {
    f: Rc<RefCell<F>>,
}

impl<F> Clone for NomParserWrapper<F> {
    fn clone(&self) -> Self {
        NomParserWrapper { f: self.f.clone() }
    }
}

impl<F> NomParserWrapper<F> {
    fn new(f: F) -> Self {
        NomParserWrapper {
            f: Rc::new(RefCell::new(f)),
        }
    }
}

impl<I, O1, E, F: Parser<I, O1, E>> Parser<I, O1, E> for NomParserWrapper<F> {
    fn parse(&mut self, i: I) -> IResult<I, O1, E> {
        self.f.borrow_mut().parse(i)
    }
}

#[derive(Debug, Display)]
enum Rule {
    #[display(fmt = "{}", _0)]
    Char(char),
    #[display(fmt = "{:?}", _0)]
    Alternatives(RuleAlternatives),
}

fn parse_rules_and_messages(s: &str) -> (RulesMap, Messages) {
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

            let mut alternatives_it = l.next().unwrap().trim().split(" | ");
            let alternative_1_str = alternatives_it.next().unwrap();
            let final_rule;
            if alternative_1_str.starts_with('"') {
                final_rule = Some(Rule::Char(alternative_1_str.chars().nth(1).unwrap()));
            } else {
                let rule_sequence_collector = |sub_str: &str| {
                    sub_str
                        .split_whitespace()
                        .map(|c| c.parse::<usize>().unwrap())
                        .collect_vec()
                };
                let mut alternatives = vec![rule_sequence_collector(alternative_1_str)];
                alternatives.extend(
                    alternatives_it.map(|alternative| rule_sequence_collector(alternative)),
                );
                final_rule = Some(Rule::Alternatives(alternatives));
            }
            (rule_id, final_rule.unwrap())
        })
        .collect::<RulesMap>();

    let messages = messages_str
        .lines()
        .map(|l| l.to_string())
        .collect::<Messages>();

    (rules, messages)
}

fn add_loop_to_rules(r: &mut RulesMap) {
    if let Some(v) = r.get_mut(&8) {
        *v = Rule::Alternatives(vec![vec![42], vec![1000]])
    };
    r.insert(1000, Rule::Alternatives(vec![vec![42, 8]]));

    if let Some(v) = r.get_mut(&11) {
        *v = Rule::Alternatives(vec![vec![42, 31], vec![2000]])
    };
    r.insert(2000, Rule::Alternatives(vec![vec![42, 11, 31]]));
}

fn alt_count(r: &RulesMap, rule_id: usize) -> usize {
    let rule = &r[&rule_id];
    match rule {
        Rule::Char(..) => 1,
        Rule::Alternatives(alternatives) => alternatives.len(),
    }
}

fn check_if_matches_sequence(
    m: &str,
    r: &RulesMap,
    sequence: RuleSequenceRef,
    message_pos: usize,
    rules_applied: &mut Vec<RuleId>,
    rules_left: &mut Vec<RuleId>,
) -> (bool, usize) {
    // If the sequence is [10, 20] and rule 10 has 1 alternative and rule 20 has 2 alternatives,
    // the iterator goes through [0, 0] and [0, 1] where the numbers represent which alternative
    // of the rule to try.
    let cartesian_iter = sequence
        .iter()
        .map(|rule_id| 0..alt_count(r, *rule_id))
        .multi_cartesian_product()
        .collect_vec();

    for candidate_alternative_ids in cartesian_iter {
        let mut current_message_pos = message_pos;
        let rules_applied_copy = rules_applied.clone();
        let rules_left_copy = rules_left.clone();
        let mut valid_cartesian_choice = true;

        rules_left.extend(sequence.iter().rev());
        for (sequence_pos, sequence_rule_id) in sequence.iter().enumerate() {
            let alternative_to_apply = candidate_alternative_ids[sequence_pos];
            if sequence_rule_id == &8 {
                // println!("looping 8");
            }
            if sequence_rule_id == &11 {
                // println!("looping 11");
            }
            let (is_match, returned_message_idx) = is_message_valid_using_recursive_descent(
                m,
                r,
                current_message_pos,
                *sequence_rule_id,
                alternative_to_apply,
                rules_applied,
                rules_left,
            );
            if is_match {
                current_message_pos = returned_message_idx;
                rules_left.pop();
            } else {
                while rules_applied.len() != rules_applied_copy.len() {
                    rules_applied.pop();
                }
                while rules_left.len() != rules_left_copy.len() {
                    rules_left.pop();
                }
                valid_cartesian_choice = false;
                break;
            }
        }

        if valid_cartesian_choice {
            return (true, current_message_pos);
        }
    }
    (false, message_pos)
}

fn is_message_valid_using_recursive_descent(
    m: &str,
    r: &RulesMap,
    message_pos: usize,
    rule_id: usize,
    alternative_to_apply: usize,
    rules_applied: &mut Vec<RuleId>,
    rules_left: &mut Vec<RuleId>,
) -> (bool, usize) {
    let rule = &r[&rule_id];

    // let rules_applied = format!("{},{}", rules_applied, rule_idx);
    rules_applied.push(rule_id);
    // println!("m_i: {:2} {:2}:{}, \n  applied: {:?} len {}", message_pos, rule_idx, rule, rules_applied, rules_applied.len());

    if message_pos >= m.len() {
        // println!("m_i too long");
        rules_applied.pop();
        return (false, message_pos);
    }
    // if message_pos >= m.len() - 3 {
    // println!("  match:   {}      m is: {}", m.chars().nth(message_pos).unwrap(), &m[0..message_pos]);
    // }

    let res = match rule {
        Rule::Char(c) => {
            let target_char = m.chars().nth(message_pos).unwrap();
            let matches = target_char == *c;
            let return_pos = if matches {
                message_pos + 1
            } else {
                message_pos
            };
            if return_pos == m.len() {
                // println!("Final char '{}' at {} mess len {}", c, return_pos, m.len());
            }
            (matches, return_pos)
        }
        Rule::Alternatives(alternatives) => check_if_matches_sequence(
            m,
            r,
            &alternatives[alternative_to_apply],
            message_pos,
            rules_applied,
            rules_left,
        ),
    };
    if !res.0 {
        rules_applied.pop();
    }
    // println!("  res  :   {}", res.0);
    res
}

fn set_of_matched_messages_for_rule_id<'a>(
    messages: Vec<&'a str>,
    r: &RulesMap,
    rule_id: RuleId,
) -> Vec<&'a str> {
    messages
        .iter()
        .map(|message| is_message_valid_using_list_of_suffixes(message, r, rule_id))
        .flatten()
        .collect_vec()
}

fn is_message_valid_using_list_of_suffixes<'a>(
    m: &'a str,
    r: &RulesMap,
    rule_id: RuleId,
) -> Vec<&'a str> {
    if m.is_empty() {
        return vec![];
    }

    let rule = &r[&rule_id];
    match rule {
        Rule::Char(ch) => {
            if m.chars().next().unwrap() == *ch {
                vec![&m[1..]]
            } else {
                vec![]
            }
        }
        Rule::Alternatives(alternatives) => alternatives
            .iter()
            .map(|candidate_sequence| {
                candidate_sequence
                    .iter()
                    .fold(vec![m], |next_messages, sequence_rule_id| {
                        set_of_matched_messages_for_rule_id(next_messages, r, *sequence_rule_id)
                    })
            })
            .flatten()
            .collect_vec(),
    }
}

fn wrap_nom_parser<'a, F>(f: F) -> NomParserWrapper<F>
where
    F: Parser<InputType<'a>, InputType<'a>, NomError<'a>>,
{
    NomParserWrapper::new(f)
}

fn build_nom_sequence_parser<'a: 't, 't>(r: &RulesMap, s: RuleSequenceRef) -> BoxedParser<'a, 't> {
    s.iter()
        .map(|rule_id| build_regular_nom_parser(r, *rule_id))
        .fold1(|prev_p, next_p| Box::new(recognize(pair(prev_p, next_p))))
        .unwrap()
}

fn build_nom_alternative_parser<'a: 't, 't>(
    prev_alternative: BoxedParser<'a, 't>,
    next_alternative: BoxedParser<'a, 't>,
) -> BoxedParser<'a, 't> {
    let alted = alt((prev_alternative, next_alternative));
    let alted: BoxedParser = Box::new(alted);
    alted
}

fn build_nom_parser_8<'a: 't, 'm, 't>(
    repeat_count: usize,
    nom_map: &'m NomParserMap<'a, 't>,
) -> NomParserWrapperExact<'a, 't> {
    // Special case looping parser 8.
    let p = nom_map.get(&42).unwrap().clone();
    let p = recognize(p);
    let p = count(p, repeat_count);
    let p: BoxedParser = Box::new(recognize(p));
    wrap_nom_parser(p)
}

fn build_nom_parser_11<'a: 't, 'm, 't>(
    repeat_count: usize,
    nom_map: &'m NomParserMap<'a, 't>,
) -> NomParserWrapperExact<'a, 't> {
    // Special case looping parser 11.
    let p_42 = nom_map.get(&42).unwrap().clone();
    let p_31 = nom_map.get(&31).unwrap().clone();

    let p_42 = recognize(count(p_42, repeat_count));
    let p_31 = recognize(count(p_31, repeat_count));
    let p = pair(p_42, p_31);
    let p: BoxedParser = Box::new(recognize(p));

    wrap_nom_parser(p)
}

// Unfortunately rust has some weird behavior / bug as described in
// https://github.com/rust-lang/rust/issues/79415 which is why we need the 'a: 't lifetime bound.
fn build_regular_nom_parser<'a: 't, 't>(r: &RulesMap, rule_id: usize) -> BoxedParser<'a, 't> {
    let rule = &r[&rule_id];
    let res = match rule {
        Rule::Char(c) => {
            let p = nom::character::complete::char(*c);
            let p: BoxedParser = Box::new(recognize(p));
            p
        }
        Rule::Alternatives(alternatives) => alternatives
            .iter()
            .map(|sequence_rule_ids| build_nom_sequence_parser(r, sequence_rule_ids))
            .fold1(|prev_alternative, next_alternative| {
                build_nom_alternative_parser(prev_alternative, next_alternative)
            })
            .unwrap(),
    };
    res
}

fn rule_shortest_matching_len(r: &RulesMap, rule_id: RuleId) -> usize {
    let rule = &r[&rule_id];
    match rule {
        Rule::Char(_) => 1,
        Rule::Alternatives(alternatives) => alternatives
            .iter()
            .map(|sequence_rule_ids| {
                sequence_rule_ids
                    .iter()
                    .map(|seq_rule_id| rule_shortest_matching_len(r, *seq_rule_id))
                    .sum()
            })
            .fold1(|prev_alternative: usize, next_alternative: usize| {
                prev_alternative.max(next_alternative)
            })
            .unwrap(),
    }
}

#[allow(unused)]
fn is_message_valid_using_nom<'a: 't, 'm, 't>(
    r: &RulesMap,
    m: &'a str,
    nom_map: &'m NomParserMap<'a, 't>,
) -> bool {
    // In order to match the message with rules that have loops, we consider an instance
    // of a parser where each repeating rule is fixed to a certain repeat count. This means
    // we'll have a cartesian product of repeat counts for each looping parser
    // e.g. (1, 1), (1, 2), (1, 3), ..., (2, 1), (2, 1), ... aka infinite.
    // Not all repeat counts will be valid for a certain message. We can pre-compute
    // what's the shortest message length that a parser can match given a specific repeat count
    // and if the shortest message length matched by the parser is longer than the input message
    // we immediately know that increasing the repeat count won't help.
    // By doing this early check for both repeat counts we can determine when to stop testing
    // candidate parsers and return false if none of the parsers matched so far.
    let p31_shortest_len = rule_shortest_matching_len(r, 31);
    let p42_shortest_len = rule_shortest_matching_len(r, 42);

    for p_8_repeat_count in 1.. {
        let p8_shortest_len = p_8_repeat_count * p42_shortest_len;
        let p11_one_shortest_len = p42_shortest_len + p31_shortest_len;
        let shortest_len = p8_shortest_len + p11_one_shortest_len;
        if shortest_len > m.len() {
            break;
        }

        for p_11_repeat_count in 1.. {
            let p11_shortest_len = p_11_repeat_count * p11_one_shortest_len;
            let shortest_len = p8_shortest_len + p11_shortest_len;
            if shortest_len > m.len() {
                break;
            }

            let mut p_8 = build_nom_parser_8(p_8_repeat_count, nom_map);
            let mut p_11 = build_nom_parser_11(p_11_repeat_count, nom_map);

            let res = p_8
                .parse(m)
                .and_then(|(input, _output)| p_11.parse(input))
                .map(|(input, _)| {
                    // If the input is empty, it means the parser matched the message fully.
                    input.is_empty()
                })
                .unwrap_or(false);
            if res {
                return res;
            }
        }
    }
    false
}

fn prepare_part2_sub_parsers<'a: 't, 'm, 't>(r: &RulesMap, nom_map: &'m mut NomParserMap<'a, 't>) {
    if r.get(&31).is_none() || r.get(&42).is_none() {
        return;
    }

    let p_31 = build_regular_nom_parser(r, 31);
    let p_31 = wrap_nom_parser(p_31);
    nom_map.insert(31, p_31);

    let p_42 = build_regular_nom_parser(r, 42);
    let p_42 = wrap_nom_parser(p_42);
    nom_map.insert(42, p_42);
}

// This won't work for part 2 (and it's surprising that it works for part 1)
// because it doesn't actually consider all possibilities.
// That's because check_if_matches_sequence returns early when it finds a candidate
// combination that matches some prefix. But it's possible that the suffix might
// not be matched in which case the code won't try to consider other combinations
// that happened after the early return.
// is_message_valid_using_list_of_suffixes_wrapper takes to check all possible
// branches.
fn is_message_valid_using_recursive_descent_wrapper(r: &RulesMap, m: &str) -> bool {
    let mut rules_applied = Vec::<RuleId>::new();
    let mut rules_left = Vec::<RuleId>::new();
    let (is_match, final_matched_idx) = is_message_valid_using_recursive_descent(
        m,
        r,
        0,
        0,
        0,
        &mut rules_applied,
        &mut rules_left,
    );
    if !is_match {
        return false;
    }
    final_matched_idx == m.len()
}

// Generic approach that works with any non-left recursive rules.
fn is_message_valid_using_list_of_suffixes_wrapper(r: &RulesMap, m: &str) -> bool {
    // An empty message means that the recursive matcher consumed the whole message.
    is_message_valid_using_list_of_suffixes(m, &r, 0)
        .iter()
        .any(|msg| msg.is_empty())
}

fn count_valid_messages(s: &str) -> usize {
    let (rules, messages) = parse_rules_and_messages(s);
    messages
        .iter()
        .map(|m| is_message_valid_using_recursive_descent_wrapper(&rules, m))
        .filter(|is_valid| *is_valid)
        .count()
}

fn count_valid_messages_p2(s: &str) -> usize {
    let (mut rules, messages) = parse_rules_and_messages(s);
    add_loop_to_rules(&mut rules);

    // Memoize part 2 special parsers for quicker reconstruction
    // of the final parser.
    let mut nom_map = NomParserMap::new();
    prepare_part2_sub_parsers(&rules, &mut nom_map);

    dbg!(&messages[0]);
    messages
        .iter()
        .map(|m| {
            // let v = is_message_valid_using_nom(&rules, m, &nom_map);
            let v = is_message_valid_using_list_of_suffixes_wrapper(&rules, m);
            println!("m: {} valid: {:?}", m, v);
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
babbbbaabbbbbabbbbbbaabaaabaaa
aaabbbbbbaaaabaababaabababbabaaabbababababaaa
bbbbbbbaaaabbbbaaabbabaaa
bbbababbbbaaaaaaaabbababaaababaabab
ababaaaaaabaaab
ababaaaaabbbaba
baabbaaaabbaaaababbaababb
abbbbabbbbaaaababbbbbbaaaababb
aaaaabbaabaaaaababaa
aaaabbaabbaaaaaaabbbabbbaaabbaabaaa
aabbbbbaabbbaaaaaabbbbbababaaaaabbaaabba
"#,
            12
        );
    }
}

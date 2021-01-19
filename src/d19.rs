use advent::helpers;
use anyhow::{Context, Result};
use derive_more::Display;
use itertools::Itertools;

type RuleId = usize;
type Subrule = Vec<usize>;
type Message = String;
type Messages = Vec<Message>;

#[derive(Debug, Display)]
enum Rule {
    #[display(fmt = "{}", _0)]
    Char(char),
    #[display(fmt = "{:?}", _0)]
    Unary(Subrule),
    #[display(fmt = "{:?} | {:?}", _0, _1)]
    BinaryAlt(Subrule, Subrule),
}
type Rules = std::collections::HashMap<RuleId, Rule>;

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
                let subrule_1 = subrule_collector(subrule_1_str);
                if let Some(s2) = subrules_it.next() {
                    let subrule_2 = subrule_collector(s2);
                    final_rule = Some(Rule::BinaryAlt(subrule_1, subrule_2));
                } else {
                    final_rule = Some(Rule::Unary(subrule_1));
                }
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
        *v = Rule::BinaryAlt(vec![42], vec![42, 8])
    };
    if let Some(v) = r.get_mut(&11) {
        *v = Rule::BinaryAlt(vec![42, 31], vec![42, 11, 31])
    };
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
        Rule::Unary(..) => 1,
        Rule::BinaryAlt(..) => 2,
    }
}

fn check_if_matches_subrule(
    m: &str,
    r: &Rules,
    subrule: &[usize],
    message_idx: usize,
    rules_applied: &mut Vec<usize>,
) -> (bool, usize) {
    let subrule_cartesian_iter = subrule.iter().map(|rule_idx| 1..=alt_count(r, *rule_idx)).multi_cartesian_product();

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
        Rule::Unary(subrule) => check_if_matches_subrule(m, r, subrule, message_idx, rules_applied),
        Rule::BinaryAlt(subrule_1, subrule_2) => {
            match subrule_to_apply {
                1 => {
                    check_if_matches_subrule(m, r, subrule_1, message_idx, rules_applied)
                },
                2 => {
                    check_if_matches_subrule(m, r, subrule_2, message_idx, rules_applied)
                },
                _ => unreachable!(),
            }
        }
    };
    if !res.0 {
        rules_applied.pop();
    }
    // println!("  res  :   {}", res.0);
    res
}

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
    dbg!(&messages[0]);
    messages
        .iter()
        .map(|m| {
            let v = is_message_valid_wrapper(m, &rules);
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

abbbbabbbbaaaababbbbbbaaaababb"#,
            1
        );

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
    }
}

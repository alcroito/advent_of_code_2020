use std::ops::RangeInclusive;

use advent::helpers;
use anyhow::{Context, Result};
use itertools::Itertools;
use pest::Parser;
use pest_derive::Parser;

type FieldValue = u64;
type Ticket = Vec<FieldValue>;
type Tickets = Vec<Ticket>;
type RuleRange = RangeInclusive<FieldValue>;
type RuleRangePair = (RuleRange, RuleRange);
type RuleName = String;
type Rules = std::collections::HashMap<RuleName, RuleRangePair>;

#[derive(Debug)]
struct State {
    your_ticket: Ticket,
    nearby_tickets: Tickets,
    rules: Rules,
}

#[derive(Parser)]
#[grammar = "d16.pest"]
pub struct TicketDocumentParser;

fn parse_document(s: &str) -> State {
    let mut your_ticket = vec![];
    let mut nearby_tickets: Tickets = vec![];
    let mut rules = Rules::new();

    let p = TicketDocumentParser::parse(Rule::document, s)
        .expect("Parsing failed")
        .next()
        .expect("No document");

    for section in p.into_inner() {
        match section.as_rule() {
            Rule::ticket_rules => {
                let ticket_rules = section.into_inner();
                ticket_rules.for_each(|rule| {
                    let mut rule = rule.into_inner();
                    let rule_name = rule.next().unwrap().as_str().to_string();
                    let rule_ranges = rule.next().unwrap().into_inner();

                    let ranges: RuleRangePair = rule_ranges
                        .map(|range| {
                            let range: (FieldValue, FieldValue) = range
                                .into_inner()
                                .map(|range_values| range_values.as_str().parse().unwrap())
                                .collect_tuple()
                                .unwrap();
                            range.0..=range.1
                        })
                        .collect_tuple()
                        .unwrap();
                    rules.insert(rule_name, ranges);
                });
            }
            Rule::your_ticket => {
                let ticket_values = section.into_inner().next().unwrap().into_inner();
                your_ticket = ticket_values
                    .map(|pair| pair.as_str().parse::<FieldValue>().unwrap())
                    .collect_vec();
            }
            Rule::nearby_tickets => {
                let tickets = section.into_inner();
                tickets.for_each(|ticket_values| {
                    let one_ticket = ticket_values
                        .into_inner()
                        .map(|pair| pair.as_str().parse::<FieldValue>().unwrap())
                        .collect_vec();

                    nearby_tickets.push(one_ticket);
                });
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }
    State {
        your_ticket,
        nearby_tickets,
        rules,
    }
}

fn compute_ticket_scanning_error_rate(s: &State) -> u64 {
    // Find biggest value in nearby tickets, and pre-allocate lookup table.
    let max_value = *s.nearby_tickets.iter().flatten().max().unwrap();
    let mut valid_values = vec![false; max_value as usize + 1];

    // Fill lookup table with valid values.
    s.rules.iter().for_each(|(_, range_pair)| {
        range_pair
            .0
            .clone()
            .chain(range_pair.1.clone())
            .for_each(|v| {
                valid_values[v as usize] = true;
            });
    });

    // Sum invalid values.
    s.nearby_tickets
        .iter()
        .flatten()
        .filter(|&&v| !valid_values[v as usize])
        .sum()
}

fn solve_p1() -> Result<()> {
    let input = helpers::get_data_from_file_res("d16").context("Coudn't read file contents.")?;
    let s = parse_document(&input);
    let result = compute_ticket_scanning_error_rate(&s);
    println!("The ticket scanning error rate is: {}", result);
    Ok(())
}

fn solve_p2() -> Result<()> {
    let _input = helpers::get_data_from_file_res("d16").context("Coudn't read file contents.")?;
    Ok(())
}

fn main() -> Result<()> {
    solve_p1()?;
    solve_p2()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p1() {
        let input = "class: 1-3 or 5-7
row: 6-11 or 33-44
seat: 13-40 or 45-50

your ticket:
7,1,14

nearby tickets:
7,3,47
40,4,50
55,2,20
38,6,12";
        let s = parse_document(input);
        let result = compute_ticket_scanning_error_rate(&s);
        assert_eq!(result, 71);
    }

    #[test]
    fn test_p2() {}
}

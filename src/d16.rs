use std::ops::RangeInclusive;

use advent::helpers;
use anyhow::{Context, Result};
use core::iter::once;
use itertools::Itertools;
use pest::Parser;
use pest_derive::Parser;

type FieldValue = u64;
type Ticket = Vec<FieldValue>;
type TicketRef<'a> = &'a [FieldValue];
type Tickets = Vec<Ticket>;

type RuleRange = RangeInclusive<FieldValue>;
type RuleRangePair = (RuleRange, RuleRange);
type RuleName = String;
type Rules = Vec<RuleRangePair>;
type RuleNames = Vec<RuleName>;

type ExpandedRange = Vec<bool>;
type ExpandedRanges = Vec<ExpandedRange>;
type ExpandedRangesRef<'a> = &'a [ExpandedRange];

type RuleToFieldMap = Vec<usize>;

#[derive(Debug)]
struct State {
    your_ticket: Ticket,
    nearby_tickets: Tickets,
    rules: Rules,
    rule_names: RuleNames,
}

#[derive(Parser)]
#[grammar = "d16.pest"]
pub struct TicketDocumentParser;

fn parse_document(s: &str) -> State {
    let mut your_ticket = Ticket::new();
    let mut nearby_tickets = Tickets::new();
    let mut rules = Rules::new();
    let mut rule_names = RuleNames::new();

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
                    rules.push(ranges);
                    rule_names.push(rule_name);
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
        rule_names,
    }
}

fn compute_biggest_value(s: &State) -> u64 {
    let max_value_tickets = *s.nearby_tickets.iter().flatten().max().unwrap();
    let max_value_rules = s
        .rules
        .iter()
        .map(|rule_range_pair| {
            once(&rule_range_pair.0)
                .chain(once(&rule_range_pair.1))
                .map(|range| range.clone().max().unwrap())
                .max()
                .unwrap()
        })
        .max()
        .unwrap();
    max_value_tickets.max(max_value_rules)
}

fn prepare_valid_value_lookup_table(s: &State) -> Vec<bool> {
    // Find biggest value from input data, and pre-allocate lookup
    // table with that many elements.
    let max_value = compute_biggest_value(s);
    let mut valid_values = vec![false; max_value as usize + 1];

    // Fill lookup table with valid values of all ranges in all rules.
    s.rules.iter().for_each(|range_pair| {
        range_pair
            .0
            .clone()
            .chain(range_pair.1.clone())
            .for_each(|v| {
                valid_values[v as usize] = true;
            });
    });
    valid_values
}

fn compute_ticket_scanning_error_rate(s: &State) -> u64 {
    let valid_values = prepare_valid_value_lookup_table(s);

    // Sum invalid values by checking each value in the valid values lookup table.
    s.nearby_tickets
        .iter()
        .flatten()
        .filter(|&&v| !valid_values[v as usize])
        .sum()
}

fn remove_invalid_tickets(s: &mut State) {
    let valid_values = prepare_valid_value_lookup_table(s);
    s.nearby_tickets
        .retain(|x| !x.iter().any(|&v| !valid_values[v as usize]));
}

fn prepare_per_rule_valid_values_lookup_table(s: &State) -> ExpandedRanges {
    // Create a lookup table of valid values for each separate rule.
    s.rules
        .iter()
        .map(|range_pair| {
            let max_value = range_pair
                .0
                .clone()
                .chain(range_pair.1.clone())
                .max()
                .unwrap();
            let mut valid_values = vec![false; max_value as usize + 1];
            range_pair
                .0
                .clone()
                .chain(range_pair.1.clone())
                .for_each(|v| {
                    valid_values[v as usize] = true;
                });
            valid_values
        })
        .collect_vec()
}

fn validate_ticket_field_using_rule(
    ticket: TicketRef,
    field_id: usize,
    expanded_ranges: ExpandedRangesRef,
    rule_id: usize,
) -> bool {
    // Extract the field_id of a ticket, and check if it's
    // valid according to the rule specified by rule_id.
    let ticket_field_value = *ticket.get(field_id).unwrap();
    // println!("    {}", ticket_field_value);
    *expanded_ranges[rule_id]
        .get(ticket_field_value as usize)
        .unwrap_or(&false)
}

fn deduce_fields(s: &State) -> RuleToFieldMap {
    type FieldIdCandidates = std::collections::HashSet<usize>;
    type FieldIdCandidatesForRules = Vec<FieldIdCandidates>;
    type UnmappedRules = std::collections::HashSet<usize>;

    // Create lookup table for each rule, for fast validity checking.
    let rule_expanded_ranges = prepare_per_rule_valid_values_lookup_table(s);

    let rule_id_iter = 0..s.rules.len();
    let field_id_iter = rule_id_iter.clone();
    // Each rule has all field ids as possible candidates.
    let mut candidates_for_rules: FieldIdCandidatesForRules =
        vec![field_id_iter.collect(); s.rules.len()];

    // All rules start as being unmapped.
    let mut unmapped_rules: UnmappedRules = rule_id_iter.collect();

    // Once mapped to a field id, they are store here.
    let mut rule_to_field_map: RuleToFieldMap = vec![0; s.rules.len()];

    // While unmapped rules exist.
    while !unmapped_rules.is_empty() {
        // Retain those rules which we can't yet map, due to having more than
        // one possible field candidate which fits the rule ranges.
        unmapped_rules.retain(|&rule_id| {
            // println!("Rule id {}: {:?}" , rule_id, s.rules[rule_id]);
            // Retain all only those field candidates for which all ticket values respect the rule.
            candidates_for_rules[rule_id].retain(|&candidate_field_id| {
                // Validate that all the ticket values of a certain field respect the rule range.
                // If at least one value does not respect the rule, the candidate will be discarded.
                s.nearby_tickets.iter().all(|ticket| {
                    validate_ticket_field_using_rule(
                        ticket,
                        candidate_field_id,
                        &rule_expanded_ranges,
                        rule_id,
                    )
                    // println!("    ticket: {:?} field_id: {} res: {}\n", ticket, candidate_field_id, is_valid_ticket_field);
                })
            });
            // When only one field candidate is left, we finally found which field id the rule maps to.
            // Remove this field id candidate for the other rules.
            if candidates_for_rules[rule_id].len() == 1 {
                let deduced_field_id = *candidates_for_rules[rule_id].iter().next().unwrap();
                candidates_for_rules
                    .iter_mut()
                    .enumerate()
                    .filter(|(i, _)| i != &rule_id)
                    .for_each(|(_, candidates)| {
                        candidates.remove(&deduced_field_id);
                    });
                rule_to_field_map[rule_id] = deduced_field_id;
                // println!("Found rule id {} corresponds to {}. Removing as candidate.", rule_id, deduced_field_id);
            }
            // println!();
            // Retain those rules, which there is more than one candidate.
            // The next loop iteration will reduce the number of candidates.
            candidates_for_rules[rule_id].len() > 1
        });
        // dbg!(&candidates_for_rules);
        // dbg!(&unmapped_rules);
    }
    // dbg!(&rule_to_field_map);
    // TODO: The current loop logic can be further optimized by precomputing the validity of
    // all values for each field against each rule in one big matrix. In that case, the each
    // loop iteration will only further reduce candidates by elimination, without having to
    // recompute the validity of each field / value / rule combo. It works fast enough
    // as-is though.
    rule_to_field_map
}

fn multiply_departure_fields(s: &State, rule_to_field_mapping: &[usize]) -> u64 {
    s.rule_names
        .iter()
        .enumerate()
        .filter(|(_, rule_name)| rule_name.starts_with("departure"))
        .map(|(i, _)| {
            let field_id = rule_to_field_mapping[i];
            s.your_ticket[field_id]
        })
        .product()
}

fn solve_p1() -> Result<()> {
    let input = helpers::get_data_from_file_res("d16").context("Coudn't read file contents.")?;
    let s = parse_document(&input);
    let result = compute_ticket_scanning_error_rate(&s);
    println!("The ticket scanning error rate is: {}", result);
    Ok(())
}

fn solve_p2() -> Result<()> {
    let input = helpers::get_data_from_file_res("d16").context("Coudn't read file contents.")?;
    let mut s = parse_document(&input);
    remove_invalid_tickets(&mut s);
    deduce_fields(&s);
    let rule_to_field_map = deduce_fields(&s);
    let result = multiply_departure_fields(&s, &rule_to_field_map);
    println!("The product of the six departure fields is: {}", result);
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
        let input = "\
class: 1-3 or 5-7
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
    fn test_p2() {
        let input = "class: 0-1 or 4-19
row: 0-5 or 8-19
seat: 0-13 or 16-19

your ticket:
11,12,13

nearby tickets:
3,9,18
15,1,5
5,14,9";
        let mut s = parse_document(input);
        remove_invalid_tickets(&mut s);
        let rule_to_field_map = deduce_fields(&s);
        let result = multiply_departure_fields(&s, &rule_to_field_map);
        assert_eq!(result, 1);
    }
}

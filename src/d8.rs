use advent::helpers;
use advent::helpers::nom::NomError2;
use anyhow::{Context, Result};
use std::convert::TryFrom;

#[allow(unused_imports)]
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while_m_n},
    character::complete::{alphanumeric0, alphanumeric1, digit1, multispace0, multispace1, one_of},
    combinator::{all_consuming, map, map_res, recognize},
    error::context,
    multi::{separated_list0, separated_list1},
    sequence::{pair, preceded, separated_pair, terminated},
    IResult,
};

#[derive(Debug, Clone, Copy)]
enum Instr {
    Nop(i32),
    Acc(i32),
    Jmp(i32),
}
type Instructions = Vec<Instr>;
type AccumulatorType = i32;

#[derive(Debug, Clone)]
struct Computer {
    ip: usize,
    instructions: Instructions,
    acc: AccumulatorType,
}

#[derive(Debug, PartialEq)]
enum ReturnStatus {
    Regular,
    Loop,
}

type EvalResult = (ReturnStatus, AccumulatorType);

type NomErrorExact<'a> = NomError2<&'a str>;

fn parse_argument(i: &str) -> IResult<&str, i32, NomErrorExact> {
    map_res(
        recognize(pair(alt((tag("+"), tag("-"))), digit1)),
        |s: &str| s.parse::<i32>(),
    )(i)
}

fn parse_instruction(i: &str) -> IResult<&str, Instr, NomErrorExact> {
    let parse_nop = context(
        "nop",
        map(
            separated_pair(
                tag("nop"),
                nom::character::complete::char(' '),
                parse_argument,
            ),
            |(_, arg)| Instr::Nop(arg),
        ),
    );
    let parse_acc = context(
        "acc",
        map(
            separated_pair(
                tag("acc"),
                nom::character::complete::char(' '),
                parse_argument,
            ),
            |(_, arg)| Instr::Acc(arg),
        ),
    );
    let parse_jmp = context(
        "jmp",
        map(
            separated_pair(
                tag("jmp"),
                nom::character::complete::char(' '),
                parse_argument,
            ),
            |(_, arg)| Instr::Jmp(arg),
        ),
    );
    alt((parse_acc, parse_jmp, parse_nop))(i)
}

fn parse_instructions(i: &str) -> IResult<&str, Instructions, NomErrorExact> {
    let parse_instructions = separated_list0(multispace1, parse_instruction);
    all_consuming(terminated(
        preceded(multispace0, parse_instructions),
        multispace0,
    ))(i)
}

impl<'a> std::convert::TryFrom<&'a str> for Computer {
    type Error = anyhow::Error;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        use nom::Finish;

        Ok(Computer::from_instructions(
            parse_instructions(s)
                .finish()
                .map_err(|e| e.into_anyhow(s))?
                .1,
        ))
    }
}

impl Computer {
    fn from_instructions(instructions: Instructions) -> Self {
        Computer {
            ip: 0,
            instructions,
            acc: 0,
        }
    }

    fn eval_instruction(&mut self, i: Instr) -> usize {
        match i {
            Instr::Acc(ref arg) => {
                self.acc += *arg;
                self.ip + 1
            }
            Instr::Jmp(ref arg) => (self.ip as i32 + *arg) as usize,
            Instr::Nop(_) => self.ip + 1,
        }
    }

    fn evaluate_until_loop(&mut self) -> EvalResult {
        let mut executed_set = std::collections::HashSet::<usize>::new();
        let max_ip = self.instructions.len();
        loop {
            if self.ip >= max_ip {
                return (ReturnStatus::Regular, self.acc);
            }
            if executed_set.contains(&self.ip) {
                return (ReturnStatus::Loop, self.acc);
            }
            executed_set.insert(self.ip);
            let instr = self.instructions[self.ip];
            self.ip = self.eval_instruction(instr);
        }
    }

    fn fix_loop_and_eval(&self) -> EvalResult {
        self.instructions
            .iter()
            .enumerate()
            .filter(|(_, val)| matches!(val, Instr::Jmp(_) | Instr::Nop(_)))
            .find_map(|(i, _)| {
                let mut new_c = self.clone();
                {
                    let instr = &mut new_c.instructions[i];
                    if let Instr::Nop(x) = instr {
                        *instr = Instr::Jmp(*x);
                    } else if let Instr::Jmp(x) = instr {
                        *instr = Instr::Nop(*x);
                    }
                }
                let (status, acc) = new_c.evaluate_until_loop();
                match status {
                    ReturnStatus::Loop => None,
                    ReturnStatus::Regular => Some((status, acc)),
                }
            })
            .expect("Expected to find a fixed loop program")
    }
}

fn solve_p1() -> Result<()> {
    let data = helpers::get_data_from_file_res("d8").context("Coudn't read file contents.")?;
    let mut c = Computer::try_from(data.as_str()).expect("Invalid computer program\n");
    println!(
        "Accumulator value before encountering loop is: {}",
        c.evaluate_until_loop().1
    );
    Ok(())
}

fn solve_p2() -> Result<()> {
    let data = helpers::get_data_from_file_res("d8").context("Coudn't read file contents.")?;
    let c = Computer::try_from(data.as_str()).expect("Invalid computer program\n");
    let (_, acc) = c.fix_loop_and_eval();
    println!("Accumulator value after fixing loop is: {}", acc);
    Ok(())
}

fn main() -> Result<()> {
    solve_p1().ok();
    solve_p2()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p1() {
        let input = "
        nop +0
        acc +1
        jmp +4
        acc +3
        jmp -3
        acc -99
        acc +1
        jmp -4
        acc +6
        ";
        let mut c = Computer::try_from(input).expect("Invalid computer program\n");
        assert_eq!(c.evaluate_until_loop().1, 5);
    }

    #[test]
    fn test_p2() {
        let input = "
        nop +0
        acc +1
        jmp +4
        acc +3
        jmp -3
        acc -99
        acc +1
        jmp -4
        acc +6
        ";
        let c = Computer::try_from(input).expect("Invalid computer program\n");
        let (status, acc) = c.fix_loop_and_eval();
        assert_eq!(status, ReturnStatus::Regular);
        assert_eq!(acc, 8);
    }
}

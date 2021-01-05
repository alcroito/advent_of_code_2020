use advent::helpers;
use anyhow::{Context, Result};
use derive_more::Display;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::str::FromStr;

#[derive(Debug, Display, Clone)]
enum BitOp {
    #[display(fmt = "{}", _0)]
    Override(char),
    #[display(fmt = "X")]
    Pass,
}
type BitOps = Vec<BitOp>;

#[derive(Debug, Display, Clone)]
enum BitOpV2 {
    #[display(fmt = "0")]
    Pass,
    #[display(fmt = "1")]
    OverrideWithOne,
    #[display(fmt = "X")]
    Floating,
}
type BitOpsV2 = Vec<BitOpV2>;

#[derive(Debug, Display)]
enum Op {
    #[display(fmt = "mask = {}", _0)]
    SetMask(Mask),
    #[display(fmt = "{}", _0)]
    WriteMemory(WriteMemoryArgs),
}
type Ops = Vec<Op>;

#[derive(Debug, Display)]
enum OpV2 {
    #[display(fmt = "mask = {}", _0)]
    SetMask(MaskV2),
    #[display(fmt = "{}", _0)]
    WriteMemory(WriteMemoryArgs),
}
type OpsV2 = Vec<OpV2>;

#[derive(Debug, Clone)]
struct Mask {
    bit_ops: BitOps,
}

#[derive(Debug, Clone)]
struct MaskV2 {
    bit_ops: BitOpsV2,
    floating_op_indices: Vec<usize>,
}

#[derive(Debug, Display)]
#[display(fmt = "mem[{}] = {}", address, value)]
struct WriteMemoryArgs {
    address: u64,
    value: u64,
}

#[derive(Debug, Default, Display)]
#[display(fmt = "{}\n{:?}", mask, memory)]
struct Memory {
    memory: std::collections::HashMap<u64, u64>,
    mask: Mask,
}

#[derive(Debug, Default, Display)]
#[display(fmt = "{}\n{:?}", mask, memory)]
struct MemoryV2 {
    memory: std::collections::HashMap<u64, u64>,
    mask: MaskV2,
}

impl FromStr for BitOp {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().next() {
            None => anyhow::bail!("No mask character"),
            Some('X') => Ok(BitOp::Pass),
            Some('0') => Ok(BitOp::Override('0')),
            Some('1') => Ok(BitOp::Override('1')),
            Some(e) => anyhow::bail!(format!("Invalid mask character: {}", e)),
        }
    }
}

impl FromStr for BitOpV2 {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(BitOpV2::from_bit_op(s.parse::<BitOp>()?))
    }
}

impl BitOpV2 {
    fn from_bit_op(op: BitOp) -> BitOpV2 {
        match op {
            BitOp::Pass => BitOpV2::Floating,
            BitOp::Override(c) => match c {
                '0' => BitOpV2::Pass,
                '1' => BitOpV2::OverrideWithOne,
                _ => unreachable!(),
            },
        }
    }
}

impl FromStr for WriteMemoryArgs {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"mem\[(\d+)\] = (\d+)").unwrap();
        }
        // let re: Regex = Regex::new(r"mem\[(\d+)\] = (\d+)").unwrap();
        // mem[7001] = 347
        let caps = RE
            .captures(s)
            .ok_or_else(|| anyhow::anyhow!("No regex match found for memory args"))?;
        let address = caps
            .get(1)
            .ok_or_else(|| anyhow::anyhow!("No match for address"))?
            .as_str()
            .parse::<u64>()?;
        let value = caps
            .get(2)
            .ok_or_else(|| anyhow::anyhow!("No match for value"))?
            .as_str()
            .parse::<u64>()?;
        Ok(WriteMemoryArgs { address, value })
    }
}

impl FromStr for Mask {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"mask = ([01X]+)").unwrap();
        }
        // let re: Regex = Regex::new(r"mask = ([01X]+)").unwrap();
        // mask = XXXXXXXXXXXXXXXXXXXXXXXXXXXXX1XXXX0X
        let caps = RE
            .captures(s)
            .ok_or_else(|| anyhow::anyhow!("No regex match found for mask"))?;
        let maybe_mask = caps
            .get(1)
            .ok_or_else(|| anyhow::anyhow!("No match for mask"))?
            .as_str();
        if maybe_mask.len() != 36 {
            anyhow::bail!(format!(
                "Mask {} has invalid length: {}",
                maybe_mask,
                maybe_mask.len()
            ));
        }
        let bit_ops = maybe_mask
            .trim()
            .chars()
            .map(|c| c.to_string().parse::<BitOp>())
            .try_collect()?;
        Ok(Mask { bit_ops })
    }
}

impl FromStr for MaskV2 {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(MaskV2::from_mask(s.parse::<Mask>()?))
    }
}

impl MaskV2 {
    fn from_mask(mask: Mask) -> MaskV2 {
        let bit_ops = mask
            .bit_ops
            .into_iter()
            .map(BitOpV2::from_bit_op)
            .collect_vec();

        // Pre-compute floating bit indicies, which will be modified
        // when applying the mask to an address.
        let floating_op_indices = bit_ops
            .iter()
            .enumerate()
            .filter_map(|(index, op)| {
                if let BitOpV2::Floating = op {
                    Some(index)
                } else {
                    None
                }
            })
            .collect_vec();
        MaskV2 {
            bit_ops,
            floating_op_indices,
        }
    }
}

impl FromStr for Op {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s[0..3] {
            "mas" => Ok(Op::SetMask(s.parse()?)),
            "mem" => Ok(Op::WriteMemory(s.parse()?)),
            _ => anyhow::bail!("Invalid operation"),
        }
    }
}

impl FromStr for OpV2 {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s[0..3] {
            "mas" => Ok(OpV2::SetMask(s.parse()?)),
            "mem" => Ok(OpV2::WriteMemory(s.parse()?)),
            _ => anyhow::bail!("Invalid operation"),
        }
    }
}

impl std::fmt::Display for Mask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for op in &self.bit_ops {
            write!(f, "{}", op)?;
        }
        Ok(())
    }
}

impl Default for Mask {
    fn default() -> Self {
        Mask {
            bit_ops: vec![BitOp::Pass; 36],
        }
    }
}

impl std::fmt::Display for MaskV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for op in &self.bit_ops {
            write!(f, "{}", op)?;
        }
        Ok(())
    }
}

impl Default for MaskV2 {
    fn default() -> Self {
        MaskV2 {
            bit_ops: vec![BitOpV2::Pass; 36],
            floating_op_indices: vec![],
        }
    }
}

#[allow(unused)]
fn print_ops(ops: &[Op]) {
    for op in ops {
        println!("{}", op);
    }
}

#[allow(unused)]
fn print_memory(mem: &Memory) {
    println!("{}", mem);
}

impl Memory {
    fn apply_ops(&mut self, ops: &[Op]) {
        ops.iter().for_each(|op| match op {
            Op::SetMask(mask) => self.mask = mask.clone(),
            Op::WriteMemory(args) => {
                self.memory
                    .insert(args.address, apply_mask(args.value, &self.mask));
            }
        });
    }
}

impl MemoryV2 {
    fn apply_ops(&mut self, ops: &[OpV2]) {
        ops.iter().for_each(|op| match op {
            OpV2::SetMask(mask) => self.mask = mask.clone(),
            OpV2::WriteMemory(args) => {
                let mask = self.mask.clone();
                apply_mask_v2(args.address, &mask).for_each(|address| {
                    self.memory.insert(address, args.value);
                })
            }
        });
    }
}

fn apply_mask(value: u64, mask: &Mask) -> u64 {
    let value_str = format!("{:036b}", value);
    let masked_value_str = value_str
        .chars()
        .zip(mask.bit_ops.iter())
        .map(|(digit, bit_op)| match bit_op {
            BitOp::Pass => digit,
            BitOp::Override(o) => *o,
        })
        .collect::<String>();
    u64::from_str_radix(&masked_value_str, 2).expect("Invalid string to int conversion")
}

fn apply_mask_v2(value: u64, mask: &MaskV2) -> impl Iterator<Item = u64> + '_ {
    let value_str = format!("{:036b}", value);
    let masked_value_str = value_str
        .chars()
        .zip(mask.bit_ops.iter())
        .map(|(digit, bit_op)| match bit_op {
            BitOpV2::Pass => digit,
            BitOpV2::OverrideWithOne => '1',
            BitOpV2::Floating => '0',
        })
        .collect::<String>();
    // Generate all subsets of indices that should be set to 1
    // and modify a clone of the mask string with the modified indices.
    mask.floating_op_indices
        .iter()
        .powerset()
        .map(move |indices| {
            let mut mask_str = masked_value_str.clone();
            let mut bytes = std::mem::take(&mut mask_str).into_bytes();
            indices.iter().for_each(|&i| bytes[*i] = b'1');
            let mask_str =
                String::from_utf8(bytes).expect("Invalid utf8 bytes to string conversion");
            u64::from_str_radix(&mask_str, 2).expect("Invalid string to int conversion")
        })
}

fn parse_writes_and_masks(s: &str) -> anyhow::Result<Ops> {
    s.trim().lines().map(|l| l.parse::<Op>()).try_collect()
}

fn parse_writes_and_masks_v2(s: &str) -> anyhow::Result<OpsV2> {
    s.trim().lines().map(|l| l.parse::<OpV2>()).try_collect()
}

fn compute_sum_of_all_values_in_memory(s: &str) -> u64 {
    let ops = parse_writes_and_masks(s).expect("Invalid ops");
    let mut memory = Memory::default();
    memory.apply_ops(&ops);
    memory.memory.iter().map(|e| e.1).sum()
}

fn compute_sum_of_all_values_in_memory_v2(s: &str) -> u64 {
    let ops = parse_writes_and_masks_v2(s).expect("Invalid ops");
    let mut memory = MemoryV2::default();
    memory.apply_ops(&ops);
    memory.memory.iter().map(|e| e.1).sum()
}

fn solve_p1() -> Result<()> {
    let input = helpers::get_data_from_file_res("d14").context("Coudn't read file contents.")?;
    let result = compute_sum_of_all_values_in_memory(&input);
    println!("The sum of all values in memory is: {}", result);
    Ok(())
}

fn solve_p2() -> Result<()> {
    let input = helpers::get_data_from_file_res("d14").context("Coudn't read file contents.")?;
    let result = compute_sum_of_all_values_in_memory_v2(&input);
    println!(
        "The sum of all values in memory using decoder V2 is: {}",
        result
    );
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
        let input = "mask = XXXXXXXXXXXXXXXXXXXXXXXXXXXXX1XXXX0X
mem[8] = 11
mem[7] = 101
mem[8] = 0";
        let result = compute_sum_of_all_values_in_memory(input);
        assert_eq!(result, 165);
    }

    #[test]
    fn test_p2() {
        let input = "mask = 000000000000000000000000000000X1001X
mem[42] = 100
mask = 00000000000000000000000000000000X0XX
mem[26] = 1";
        let result = compute_sum_of_all_values_in_memory_v2(input);
        assert_eq!(result, 208);
    }
}

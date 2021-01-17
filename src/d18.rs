use advent::helpers;
use anyhow::{Context, Result};
use derive_more::Display;
use itertools::Itertools;

type LiteralType = u64;

#[derive(Debug, Display)]
enum BinaryOpKind {
    #[display(fmt = "+")]
    Add,
    #[display(fmt = "*")]
    Mul,
}

#[derive(Debug)]
enum MathExpr {
    Literal(LiteralType),
    BinaryOp(Box<MathExpr>, Box<MathExpr>, BinaryOpKind),
}

enum PrecedenceKind {
    Equal,
    GreaterAdd,
}

fn is_paren(c: &char) -> bool {
    *c == '(' || *c == ')'
}

impl BinaryOpKind {
    fn get_precedence(&self, precedence_kind: &PrecedenceKind) -> u8 {
        match precedence_kind {
            PrecedenceKind::Equal => 1,
            PrecedenceKind::GreaterAdd => match self {
                BinaryOpKind::Add => 2,
                BinaryOpKind::Mul => 1,
            },
        }
    }
}

fn char_to_binary_op_kind(c: &char) -> BinaryOpKind {
    match c {
        '+' => BinaryOpKind::Add,
        '*' => BinaryOpKind::Mul,
        _ => unreachable!(),
    }
}

fn make_binary_op(c: &char, operands: &mut Vec<MathExpr>) {
    let arg_1 = Box::new(operands.pop().unwrap());
    let arg_2 = Box::new(operands.pop().unwrap());
    let op_kind = char_to_binary_op_kind(c);
    operands.push(MathExpr::BinaryOp(arg_1, arg_2, op_kind));
}

/// Returns a token iterator for given string, essentially splitting at whitespace and parenthesis,
/// while keeping the parenthesis.
/// Takes   '1 + (2 * 3)'
/// Returns ['1', '+', '(', '2', '*', '3', ')']
fn make_tokenizer(s: &str) -> impl std::iter::Iterator<Item = &str> + '_ {
    s.split_whitespace()
        .map(|token| {
            // poor's man split_including_delim() that keeps the paranthesis delimiters as values.
            let mut last_paren_idx = 0;
            let mut parens_and_tokens = token
                .match_indices(|c: char| is_paren(&c))
                .map(|(idx, matched)| {
                    let res = if last_paren_idx != idx {
                        vec![&token[last_paren_idx..idx], matched]
                    } else {
                        vec![matched]
                    };
                    last_paren_idx = idx + matched.len();
                    res
                })
                .flatten()
                .collect_vec();
            if last_paren_idx < token.len() {
                parens_and_tokens.push(&token[last_paren_idx..])
            }
            parens_and_tokens
        })
        .flatten()
}

fn parse_string_to_math_expr(s: &str, precedence_kind: &PrecedenceKind) -> MathExpr {
    let mut operands = Vec::<MathExpr>::new();
    let mut ops = Vec::<char>::new();
    let tokenizer = make_tokenizer(s);

    tokenizer.for_each(|token| {
        // Implementation of shunting-yard.
        match token.chars().next().unwrap() {
            lit @ '0'..='9' => {
                let lit = MathExpr::Literal(lit.to_digit(10).unwrap() as LiteralType);
                operands.push(lit);
            }
            open_paren @ '(' => {
                ops.push(open_paren);
            }
            ')' => {
                while !ops.is_empty() {
                    let op_char = ops.pop().unwrap();
                    match op_char {
                        '(' => break,
                        _ => make_binary_op(&op_char, &mut operands),
                    }
                }
            }
            op_kind_char @ '+' | op_kind_char @ '*' => {
                while !ops.is_empty() {
                    let top_stack_op_char = ops.last().unwrap();
                    match top_stack_op_char {
                        '(' => break,
                        _ => {
                            let stack_top_op_precedence_is_higher =
                                char_to_binary_op_kind(top_stack_op_char)
                                    .get_precedence(precedence_kind)
                                    >= char_to_binary_op_kind(&op_kind_char)
                                        .get_precedence(precedence_kind);
                            if stack_top_op_precedence_is_higher {
                                make_binary_op(top_stack_op_char, &mut operands);
                                ops.pop();
                            } else {
                                break;
                            }
                        }
                    }
                }
                ops.push(op_kind_char);
            }
            _ => unreachable!(),
        };
    });

    // Assemble the AST from the remaining operators.
    while let Some(op_char) = ops.pop() {
        make_binary_op(&op_char, &mut operands);
    }

    operands.pop().unwrap()
}

fn reduce_math_expr(expr: &MathExpr) -> LiteralType {
    match expr {
        MathExpr::Literal(lit) => *lit,
        MathExpr::BinaryOp(arg_1, arg_2, op_kind) => {
            let arg_1_reduced = reduce_math_expr(arg_1.as_ref());
            let arg_2_reduced = reduce_math_expr(arg_2.as_ref());
            match op_kind {
                BinaryOpKind::Add => arg_1_reduced + arg_2_reduced,
                BinaryOpKind::Mul => arg_1_reduced * arg_2_reduced,
            }
        }
    }
}

impl std::fmt::Display for MathExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MathExpr::Literal(lit) => {
                write!(f, "{}", lit)?;
            }
            MathExpr::BinaryOp(arg_1, arg_2, op_kind) => {
                write!(f, "({} {} {})", arg_1.as_ref(), op_kind, arg_2.as_ref())?;
            }
        }
        Ok(())
    }
}

fn eval_math_expr(s: &str, precedence_kind: &PrecedenceKind) -> i64 {
    let expr = parse_string_to_math_expr(s, precedence_kind);
    println!("{} = {}", expr, reduce_math_expr(&expr));
    reduce_math_expr(&expr) as i64
}

fn eval_homework_as_sum_of_expr(s: &str, precedence_kind: &PrecedenceKind) -> i64 {
    s.lines().map(|l| eval_math_expr(l, precedence_kind)).sum()
}

fn eval_homework_as_sum_of_expr_equal_precedence(s: &str) -> i64 {
    eval_homework_as_sum_of_expr(s, &PrecedenceKind::Equal)
}

fn eval_homework_as_sum_of_expr_greater_add_precedence(s: &str) -> i64 {
    eval_homework_as_sum_of_expr(s, &PrecedenceKind::GreaterAdd)
}

fn solve_p1() -> Result<()> {
    let input = helpers::get_data_from_file_res("d18").context("Coudn't read file contents.")?;
    let result = eval_homework_as_sum_of_expr_equal_precedence(&input);
    println!(
        "The sum of the expression using regular precedence is: {}",
        result
    );
    Ok(())
}

fn solve_p2() -> Result<()> {
    let input = helpers::get_data_from_file_res("d18").context("Coudn't read file contents.")?;
    let result = eval_homework_as_sum_of_expr_greater_add_precedence(&input);
    println!(
        "The sum of the expression using GreaterAdd precedence is: {}",
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
                assert_eq!(
                    eval_homework_as_sum_of_expr_equal_precedence(input),
                    $solution
                )
            };
        }

        test!("1 + 2 * 3 + 4 * 5 + 6", 71);
        test!("1 + (2 * 3) + (4 * (5 + 6))", 51);
        test!("2 * 3 + (4 * 5)", 26);
        test!("5 + (8 * 3 + 9 + 3 * 4 * 3)", 437);
        test!("5 * 9 * (7 * 3 * 3 + 9 * 3 + (8 + 6 * 4))", 12240);
        test!("((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2", 13632);
    }

    #[test]
    fn test_p2() {
        macro_rules! test {
            ($expr: literal, $solution: expr) => {
                let input = $expr;
                assert_eq!(
                    eval_homework_as_sum_of_expr_greater_add_precedence(input),
                    $solution
                )
            };
        }

        test!("1 + 2 * 3 + 4 * 5 + 6", 231);
        test!("1 + (2 * 3) + (4 * (5 + 6))", 51);
        test!("2 * 3 + (4 * 5)", 46);
        test!("5 + (8 * 3 + 9 + 3 * 4 * 3)", 1445);
        test!("5 * 9 * (7 * 3 * 3 + 9 * 3 + (8 + 6 * 4))", 669060);
        test!("((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2", 23340);
    }
}

use std::{num::ParseIntError, str::FromStr};

use anyhow::anyhow;
use regex::{Captures, Regex};

pub type SolverResult<T, E = anyhow::Error> = anyhow::Result<T, E>;

pub fn eval_subexpression(expr: String, multiplication_re: &Regex) -> SolverResult<i64> {
    let expr = handle_multiplications(expr, &multiplication_re)?;

    Ok(handle_summations(expr)?)
}

pub fn handle_parentheses(
    expr: String,
    multiplication_re: &Regex,
    parenthesized_subexpr_re: &Regex,
) -> SolverResult<String> {
    calculate_and_replace(expr, parenthesized_subexpr_re, &|captures| {
        calculate_replacement_for_subexpression(&captures, multiplication_re)
    })
}

struct BinaryOperation<'a> {
    first_operand: i64,
    operator: &'a str,
    second_operand: i64,
}

impl<'a> BinaryOperation<'a> {
    fn from_captures(captures: &'a Captures) -> SolverResult<Self> {
        Ok(BinaryOperation {
            first_operand: captures
                .get(1)
                .ok_or(anyhow!("first operand not found"))?
                .as_str()
                .parse()?,
            operator: captures
                .get(2)
                .ok_or(anyhow!("operator not found"))?
                .as_str(),
            second_operand: captures
                .get(3)
                .ok_or(anyhow!("second operand not found"))?
                .as_str()
                .parse()?,
        })
    }
}

struct Replacement {
    start: usize,
    end: usize,
    new_value: String,
}

fn calculate_and_replace<F>(mut expr: String, re: &Regex, calculate_fn: &F) -> SolverResult<String>
where
    F: Fn(&Captures) -> SolverResult<Replacement>,
{
    while let Some(all_captures) = get_all_captures(&expr, re) {
        let replacements = all_captures
            .iter()
            .map(calculate_fn)
            .collect::<SolverResult<Vec<Replacement>>>()?;

        for replacement in replacements.iter().rev() {
            expr.replace_range(replacement.start..replacement.end, &replacement.new_value);
        }
    }

    Ok(expr)
}

fn calculate_replacement_for_multiplication(captures: &Captures) -> SolverResult<Replacement> {
    let result = eval_multiplication(&BinaryOperation::from_captures(&captures)?)?.to_string();

    let full_match = captures.get(0).unwrap();

    Ok(Replacement {
        start: full_match.start(),
        end: full_match.end(),
        new_value: result,
    })
}

fn calculate_replacement_for_subexpression(
    captures: &Captures,
    multiplication_re: &Regex,
) -> SolverResult<Replacement> {
    let subexpr = captures.get(1).unwrap().as_str();

    let subexpr_result =
        eval_subexpression(String::from_str(subexpr)?, multiplication_re)?.to_string();

    let full_match = captures.get(0).unwrap();

    Ok(Replacement {
        start: full_match.start(),
        end: full_match.end(),
        new_value: subexpr_result,
    })
}

fn eval_multiplication(binary_operation: &BinaryOperation) -> SolverResult<i64> {
    match binary_operation.operator {
        "*" => Ok(binary_operation.first_operand * binary_operation.second_operand),
        "/" => match binary_operation.second_operand {
            0 => return Err(anyhow!("division by zero")),
            nonzero => Ok(binary_operation.first_operand / nonzero),
        },
        _ => Err(anyhow!("invalid operator for multiplication")),
    }
}

fn get_all_captures<'a>(value: &'a str, pattern_re: &Regex) -> Option<Vec<Captures<'a>>> {
    let all_captures: Vec<Captures> = pattern_re.captures_iter(value).collect();

    if all_captures.is_empty() {
        return None;
    }

    Some(all_captures)
}

fn handle_multiplications(expr: String, multiplication_re: &Regex) -> SolverResult<String> {
    calculate_and_replace(expr, multiplication_re, &|captures| {
        calculate_replacement_for_multiplication(&captures)
    })
}

fn handle_summations(mut expr: String) -> SolverResult<i64> {
    if expr.starts_with('-') {
        expr.replace_range(0..0, "0");
    }

    expr = expr.replace("--", "+");
    expr = expr.replace("+-", "-");
    expr = expr.replace("-", "+-");

    Ok(expr
        .split('+')
        .map(|num_str| num_str.parse::<i64>())
        .collect::<Result<Vec<i64>, ParseIntError>>()?
        .iter()
        .sum())
}

use anyhow::anyhow;
use regex::Regex;

mod solver;

const EXPRESSION_PATTERN: &str = r"^[0-9\+\-\*\/\(\)]+$";
const MULTIPLICATION_PATTERN: &str = r"([0-9]+)([\*\/])([-]?[0-9]+)";
const PARENTHESIZED_SUBEXPRESSION_PATTERN: &str = r"[\(]([^\(\)]+)[\)]";
const SUBEXPRESSION_PATTERN: &str = r"^[^\(\)]+$";
const INVALID_PARENTHESES_PATTERN: &str = r"[0-9\)]\(";

pub struct ExpressionEvaluator {
    expression_re: Regex,
    multiplication_re: Regex,
    parenthesized_subexpr_re: Regex,
    subexpression_re: Regex,
    invalid_parentheses_re: Regex,
}

impl ExpressionEvaluator {
    pub fn new() -> anyhow::Result<Self> {
        let expression_re = Regex::new(EXPRESSION_PATTERN)?;
        let multiplication_re = Regex::new(MULTIPLICATION_PATTERN)?;
        let parenthesized_subexpr_re = Regex::new(PARENTHESIZED_SUBEXPRESSION_PATTERN)?;
        let subexpression_re = Regex::new(SUBEXPRESSION_PATTERN)?;
        let invalid_parentheses_re = Regex::new(INVALID_PARENTHESES_PATTERN)?;

        Ok(ExpressionEvaluator {
            expression_re,
            multiplication_re,
            parenthesized_subexpr_re,
            subexpression_re,
            invalid_parentheses_re,
        })
    }

    pub fn eval(&self, mut expr: String) -> anyhow::Result<i64> {
        expr.retain(|c| !c.is_whitespace());

        if !self.expression_re.is_match(&expr) {
            return Err(anyhow!("not a valid expression"));
        }

        if self.invalid_parentheses_re.is_match(&expr) {
            return Err(anyhow!(
                "opening parenthesis after digit or closing parenthesis"
            ));
        }

        // Check leading double hyphen, because subsequent transformations can produce it even in case of valid inputs and the solver is able to "solve" it
        // Therefore, if we want to differentiate between input and solver transformations, we have to do it here
        if expr.starts_with("--") {
            return Err(anyhow::anyhow!("starts with double hyphens"));
        }

        // Solve parenthesized subexpressions, and transform the expression in a way that there are no more parentheses
        let expr = solver::handle_parentheses(
            expr,
            &self.multiplication_re,
            &self.parenthesized_subexpr_re,
        )?;

        if !self.subexpression_re.is_match(&expr) {
            return Err(anyhow!("not a valid subexpression"));
        }

        // After parentheses are removed, the expression is itself a subexpression
        solver::eval_subexpression(expr, &self.multiplication_re)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    const LONG_EXPR : &str = "((1+(-2*(3-(4/(-5+6*(-7-(8/(-9+1))))))))+((11*(-12+13))/(14-(15*(-16+17))))-(18+(-19*(20-(21/(-22+23*(-24-(25/(-26+27))))))))+(28*(-29+(30/(31-(32*(-33+34))))))-(35+(-36*(37-(38/(-39+40*(-41-(42/(-43+44))))))))+(45*(-46+(47/(48-(49*(-50+51))))))-(52+(-53*(54-(55/(-56+57*(-58-(59/(-60+61))))))))+(62*(-63+(64/(65-(66*(-67+68))))))-(69+(-70*(71-(72/(-73+74*(-75-(76/(-77+78))))))))+(79*(-80+(81/(82-(83*(-84+85))))))-(86+(-87*(88-(89/(-90+91*(-92-(93/(-94+95))))))))+(96*(-97+(98/(99-(100*(-101+102))))))+(103+(-104*(105-(106/(-107+108*(-109-(110/(-111+112))))))))+(113*(-114+(115/(116-(117*(-118+119))))))-(120+(-121*(122-(123/(-124+125*(-126-(127/(-128+129))))))))+(130*(-131+(132/(133-(134*(-135+136)))))))";
    const SHORT_EXPR: &str = "2+3*(1+4/2)";

    fn eval_str(expr: &str) -> anyhow::Result<i64> {
        let evaluator = ExpressionEvaluator::new().unwrap();

        eval_str_custom(&evaluator, expr)
    }

    fn eval_str_custom(evaluator: &ExpressionEvaluator, expr: &str) -> anyhow::Result<i64> {
        evaluator.eval(String::from_str(expr)?)
    }

    #[test]
    fn calculates_correct_result() {
        let test_data = vec![
            ("1+1", 2),
            ("1-1", 0),
            ("1/1", 1),
            ("1*1", 1),
            ("1+(1-1)", 1),
            ("1000*1000", 1000000),
            ("-1-1", -2),
            ("-10/5", -2),
            ("-10*-10", 100),
            ("1+-1", 0),
            ("1--1", 2),
        ];

        for (expr, result) in test_data {
            assert_eq!(eval_str(expr).unwrap(), result);
        }
    }

    #[test]
    fn handles_whitespace() {
        assert_eq!(
            eval_str(" \t2\n      +3\t*(  1  + 4 /2)\t\t").unwrap(),
            eval_str("2+3*(1+4/2)").unwrap()
        );
    }

    #[test]
    fn handles_parentheses() {
        let test_data = vec![
            ("(1)", 1),
            ("(1-1)", 0),
            ("60/(((2+3)))", 12),
            ("3*(5-((2-4)-(-4)))", 9),
        ];

        for (expr, result) in test_data {
            assert_eq!(eval_str(expr).unwrap(), result);
        }
    }

    #[test]
    fn calculates_integer_division_correctly() {
        assert_eq!(eval_str("1/2").unwrap(), 0);
        assert_eq!(eval_str("25/12").unwrap(), 2);
        assert_eq!(eval_str("3/2*2").unwrap(), 2);
    }

    // Test error cases to guarantee that the program doesn't panic or return a number incorrectly

    #[test]
    fn rejects_zero_division() {
        assert!(eval_str("1/0").is_err());
    }

    #[test]
    fn handles_syntax_error() {
        assert!(eval_str("()").is_err());
        assert!(eval_str("(1+1").is_err());
        assert!(eval_str("1+1)").is_err());
        assert!(eval_str("1++1").is_err());
        assert!(eval_str("2(3+1)").is_err());
        assert!(eval_str("(2+2)(3+3)").is_err());
        assert!(eval_str("--1").is_err());
        assert!(eval_str("1+a").is_err());
        assert!(eval_str("asdf").is_err());
    }

    #[test]
    fn rejects_float() {
        assert!(eval_str("1.0+1").is_err());
        assert!(eval_str("3/2.0").is_err());
    }

    #[test]
    fn performance_short() {
        let evaluator = ExpressionEvaluator::new().unwrap();

        for _ in 0..1000 {
            assert!(eval_str_custom(&evaluator, SHORT_EXPR).is_ok());
        }
    }

    #[test]
    fn performance_long() {
        let evaluator = ExpressionEvaluator::new().unwrap();

        for _ in 0..1000 {
            assert!(eval_str_custom(&evaluator, LONG_EXPR).is_ok());
        }
    }

    #[test]
    fn performance_long_expr() {
        let mut expr = String::with_capacity(LONG_EXPR.len() * 1000 + 999);

        expr.push_str(LONG_EXPR);

        for i in 0..999 {
            expr.push_str(if i % 2 == 0 { "-" } else { "+" });
            expr.push_str(LONG_EXPR);
        }

        let evaluator = ExpressionEvaluator::new().unwrap();

        assert_eq!(evaluator.eval(expr).unwrap(), 0);
    }
}

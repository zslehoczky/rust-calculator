pub struct Config;

mod expression;

use expression::ExpressionEvaluator;

pub fn run(_config: Config) -> anyhow::Result<()> {
    let evaluator = ExpressionEvaluator::new()?;

    loop {
        let input = get_stdin()?;

        let result = evaluator.eval(input);

        print_expression_result(&result);
    }
}

fn get_stdin() -> anyhow::Result<String> {
    let mut input = String::new();

    std::io::stdin().read_line(&mut input)?;

    Ok(input)
}

fn print_expression_result(result: &anyhow::Result<i64>) {
    match result {
        Ok(solution) => {
            println!("{solution}");
        }
        Err(error) => {
            eprintln!("Error: {error}");
        }
    }
}

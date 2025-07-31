use basic_arithmetic_calculator as calculator;

fn main() {
    let config = calculator::Config;

    if let Err(error) = calculator::run(config) {
        eprintln!("Application error: {error}");

        std::process::exit(1);
    }
}

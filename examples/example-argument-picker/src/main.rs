//! Example Argument Picker
//!
//! > Demonstrates how to use Mingling's `picker` feature and `Picker` to extract typed arguments from the command line.
//!
//! Run:
//! ```bash
//! cargo run --manifest-path examples/example-argument-picker/Cargo.toml --quiet -- calc 1 + 1
//! cargo run --manifest-path examples/example-argument-picker/Cargo.toml --quiet -- calc 7 * 7
//! cargo run --manifest-path examples/example-argument-picker/Cargo.toml --quiet -- calc
//! cargo run --manifest-path examples/example-argument-picker/Cargo.toml --quiet -- calc 1
//! cargo run --manifest-path examples/example-argument-picker/Cargo.toml --quiet -- calc 1 +
//! cargo run --manifest-path examples/example-argument-picker/Cargo.toml --quiet -- calc 4 / 3
//! cargo run --manifest-path examples/example-argument-picker/Cargo.toml --quiet -- calc 4 / 3 --round
//! ```
//!
//! Output:
//! ```plaintext
//! Result: 2
//! Result: 49
//! Error: First number (number_a) was not provided.
//! Error: Operator was not provided.
//! Error: Second number (number_b) was not provided.
//! Result: 1.3333334
//! Result: 1
//! ```

use mingling::{
    consts::REMAINS,
    macros::route,
    picker::{
        IntoPicker, PickerArgResult, SinglePickable,
        parselib::{ParserStyle, UNIX_STYLE},
        value::Flag,
    },
    prelude::*,
};

// --------- IMPORTANT ---------
// Use picker::BasicProgramSetup instead of the original BasicProgramSetup
// It uses arg-picker to rewrite the logic of the original BasicProgramSetup
use mingling::setup::picker::BasicProgramSetup;

// --------- IMPORTANT ---------

dispatcher!("calc", EntryCalculate);

#[derive(Grouped, Default)]
pub struct ErrorNumberANotProvided;

#[derive(Grouped, Default)]
pub struct ErrorNumberBNotProvided;

#[derive(Grouped, Default)]
pub struct ErrorNumberOperatorNotProvided;

#[derive(Grouped, Default)]
pub struct ErrorDivisionByZero;

#[derive(Grouped, Wrap)]
pub struct StateAdd((f32, f32));

#[derive(Grouped, Wrap)]
pub struct StateSubtract((f32, f32));

#[derive(Grouped, Wrap)]
pub struct StateMultiply((f32, f32));

#[derive(Grouped, Wrap)]
pub struct StateDivide((f32, f32));

#[derive(Grouped, Wrap)]
pub struct ResultNumber(f32);

#[derive(Grouped)]
struct StateCalculate {
    number_a: f32,
    operator: Operator,
    number_b: f32,
}

#[derive(Debug, PartialEq, Eq)]
enum Operator {
    Plus,
    Dash,
    Slash,
    Star,
}

// --------- IMPORTANT ---------
// Define SinglePickable for type Operator
// This allows the type to be picked as an argument
impl SinglePickable for Operator {
    fn pick_single(str: Option<&str>) -> PickerArgResult<Self> {
        let Some(str) = str else {
            return PickerArgResult::NotFound;
        };
        let op = match str.chars().next() {
            Some('+') => Operator::Plus,
            Some('-') => Operator::Dash,
            Some('*') => Operator::Star,
            Some('/') => Operator::Slash,
            _ => return PickerArgResult::NotFound,
        };
        PickerArgResult::Parsed(op)
    }
}
// --------- IMPORTANT ---------

#[derive(Default, Clone)]
struct ResNumberDisplaySetting {
    round: bool,
}

fn main() {
    let mut program = ThisProgram::new();

    // Use ParserStyle to manage the arg-picker theme
    ParserStyle::set_global_style(&UNIX_STYLE);

    // Enable picker::BasicProgramSetup
    program.with_setup(BasicProgramSetup);

    // --------- IMPORTANT ---------
    // Pre-process global arguments before executing commands
    let (round, args) = program
        .take_args()
        //     Use arg![round: Flag] to indicate the `--round` | `-R` flag
        //     |
        //     vvvvvvvvvvvvvvvv
        .pick(&arg![round: Flag, 'R'])
        //    Use REMAINS to extract remaining arguments
        //    |
        //    vvvvvvvv
        .pick(&REMAINS)
        // Since Flag and REMAINS will not fail to parse,
        //   we can safely unwrap here
        .unwrap();
    program.replace_args(args.into());

    program.with_resource(ResNumberDisplaySetting { round: *round });
    // --------- IMPORTANT ---------

    program.exec_and_exit();
}

#[chain]
fn handle_calc(args: EntryCalculate) -> Next {
    // --------- IMPORTANT ---------
    let (number_a, operator, number_b) = route!(
        //                 Use the arg! macro to define a positional argument of type f32
        //                 |
        //                 vvvvvvvvvv
        args.pick_or_route(&arg![f32], || ErrorNumberANotProvided.to_chain())
            .pick_or_route(&arg![Operator], || {
                ErrorNumberOperatorNotProvided.to_chain()
            }) //                         Returns a routable type when not found or fails to parse
            //                            |
            //                            vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
            .pick_or_route(&arg![f32], || ErrorNumberBNotProvided.to_chain())
            // Use `to_result` to parse arguments
            //   and convert to Result<(Tuple, ...), Route> type
            .to_result()
    );
    // --------- IMPORTANT ---------

    if operator == Operator::Slash && number_b == 0. {
        return ErrorDivisionByZero.to_chain();
    }

    StateCalculate {
        number_a,
        operator,
        number_b,
    }
    .to_chain()
}

#[chain]
fn handle_state_calculate(state: StateCalculate) -> Next {
    match (state.operator, state.number_a, state.number_b) {
        (Operator::Plus, a, b) => StateAdd((a, b)).to_chain(),
        (Operator::Dash, a, b) => StateSubtract((a, b)).to_chain(),
        (Operator::Slash, a, b) => StateDivide((a, b)).to_chain(),
        (Operator::Star, a, b) => StateMultiply((a, b)).to_chain(),
    }
}

#[chain]
fn handle_state_add(state_add: StateAdd) -> ResultNumber {
    let (a, b) = state_add.0;
    ResultNumber(a + b)
}

#[chain]
fn handle_state_subtract(state_subtract: StateSubtract) -> ResultNumber {
    let (a, b) = state_subtract.0;
    ResultNumber(a - b)
}

#[chain]
fn handle_state_multiply(state_multiply: StateMultiply) -> ResultNumber {
    let (a, b) = state_multiply.0;
    ResultNumber(a * b)
}

#[chain]
fn handle_state_divide(state_divide: StateDivide) -> ResultNumber {
    let (a, b) = state_divide.0;
    ResultNumber(a / b)
}

#[renderer]
fn render_result_number(result: ResultNumber, setting: &ResNumberDisplaySetting) -> String {
    let round = setting.round;
    let result = if round { result.round() } else { result.0 };
    format!("Result: {}", result)
}

#[renderer]
fn render_error_division_by_zero(_: ErrorDivisionByZero) -> String {
    "Error: Division by zero is not allowed!".to_string()
}

#[renderer]
fn render_error_number_a_not_provided(_: ErrorNumberANotProvided) -> String {
    "Error: First number (number_a) was not provided.".to_string()
}

#[renderer]
fn render_error_number_b_not_provided(_: ErrorNumberBNotProvided) -> String {
    "Error: Second number (number_b) was not provided.".to_string()
}

#[renderer]
fn render_error_number_operator_not_provided(_: ErrorNumberOperatorNotProvided) -> String {
    "Error: Operator was not provided.".to_string()
}

gen_program!();

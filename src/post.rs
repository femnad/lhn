use regex::Regex;
use crate::post::Operation::Line;

const INDEX_REGEX: &str = r"[0-9]+";

enum Operation {
    Line(u32)
}

#[derive(Debug)]
struct ParseError {
    reason: String
}

impl ParseError {
    fn new(reason: &str) -> ParseError {
        ParseError { reason: reason.to_string() }
    }
}

fn parse(op_str: &str) -> Result<Operation, ParseError> {
    let re = Regex::new(INDEX_REGEX).unwrap();
    let tokens = op_str.split(" ").collect::<Vec<&str>>();
    let operation = *tokens.first().unwrap();
    let index = *tokens.last().unwrap();
    if operation.eq("line") && re.is_match(index) {
        let index: u32 = index.parse().unwrap();
        return Ok(Line(index));
    }
    return Err(ParseError::new("no matching operations"));
}

fn head(input: &str, count: u32) -> String {
    let mut lines = input.split("\n").into_iter();
    return lines.nth(count as usize).unwrap().to_string();
}

fn do_run_op(input: &str, operation: Operation) -> String {
    match operation {
        Line(count) => head(input, count),
    }
}

pub fn run_op(cmd_output: &str, post: &str) -> String {
    let operation = parse(post);
    if operation.is_err() {
        return cmd_output.to_string()
    }
    return do_run_op(cmd_output, operation.unwrap())
}

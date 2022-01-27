use crate::post::Operation::{Field, Line};
use regex::Regex;

const INDEX_REGEX: &str = r"[0-9]+";

enum Operation {
    Line(u32),
    Field(u32),
}

#[derive(Debug)]
struct ParseError {
    reason: String,
}

impl ParseError {
    fn new(reason: &str) -> ParseError {
        ParseError {
            reason: reason.to_string(),
        }
    }
}

fn parse(op_str: &str) -> Result<Operation, ParseError> {
    let re = Regex::new(INDEX_REGEX).unwrap();
    let tokens = op_str.split(' ').collect::<Vec<&str>>();
    let operation = *tokens.first().unwrap();
    let index = *tokens.last().unwrap();
    if operation.eq("line") && re.is_match(index) {
        let index: u32 = index.parse().unwrap();
        return Ok(Line(index));
    } else if operation.eq("field") && re.is_match(index) {
        let index: u32 = index.parse().unwrap();
        return Ok(Field(index));
    }
    Err(ParseError::new("no matching operations"))
}

fn head(input: &str, count: u32) -> String {
    let mut lines = input.split('\n');
    lines.nth(count as usize).unwrap().to_string()
}

fn cut(input: &str, count: u32) -> String {
    let mut lines = input.split(' ');
    lines.nth(count as usize).unwrap().to_string()
}

fn do_run_op(input: &str, operation: Operation) -> String {
    match operation {
        Line(count) => head(input, count),
        Field(count) => cut(input, count),
    }
}

fn do_run_ops(cmd_output: &str, ops: &str) -> String {
    let mut result = String::from(&(*cmd_output));
    let ops: Vec<&str> = ops.split('|').map(|op| op.trim()).collect();

    ops.iter().for_each(|op| {
        let op = parse(op);
        match op {
            Ok(operation) => result = do_run_op(result.as_str(), operation),
            Err(fail) => panic!("Error parsing operation: {}", fail.reason),
        }
    });
    result
}

pub fn run_op(cmd_output: &str, post: &str) -> String {
    do_run_ops(cmd_output, post)
}

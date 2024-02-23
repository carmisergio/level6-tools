use super::instructions::Statement;
use crate::logging::AssemblerErrorKind;
use nom::{
    bytes::complete::{tag, take_while1},
    error::{ErrorKind, ParseError},
    sequence::terminated,
    IResult,
};

#[derive(Debug)]
pub struct AssemblerParseError<'a> {
    _input: &'a str,
    pub kind: AssemblerErrorKind,
}

impl<'a> ParseError<&'a str> for AssemblerParseError<'a> {
    fn from_error_kind(input: &'a str, kind: ErrorKind) -> Self {
        Self {
            _input: input,
            kind: AssemblerErrorKind::Nom(kind),
        }
    }

    fn append(_input: &'a str, _kind: ErrorKind, other: Self) -> Self {
        other
    }
}

pub fn parse_label(input: &str) -> IResult<&str, &str> {
    terminated(parse_label_identifier, tag(":"))(input)
}

fn parse_label_identifier(input: &str) -> IResult<&str, &str> {
    take_while1(|chr: char| -> bool { chr.is_alphanumeric() || chr == '_' })(input)
}

pub fn parse_statement(input: &str) -> IResult<&str, Statement, AssemblerParseError> {
    // TODO create actual parsing function
    Ok((input, Statement::Org(1234)))
}

#[cfg(test)]
mod test {
    use super::*;
    use nom::Err;

    #[test]
    fn parse_label_succ() {
        let tests = [
            (
                "loop: ldr, something, something",
                "loop",
                " ldr, something, something",
            ),
            ("also_a_valid_label76:", "also_a_valid_label76", ""),
        ];
        for (input, exp_output, exp_remaining) in tests {
            let (remaining, output) = parse_label(input).unwrap();
            assert_eq!(output, exp_output);
            assert_eq!(remaining, exp_remaining);
        }
    }

    #[test]
    fn parse_label_err() {
        let tests = [("", false), ("abcde", false), ("lab&el:", false)];
        for (input, exp_failure) in tests {
            let err = parse_label(input).unwrap_err();

            match err {
                Err::Incomplete(_) => panic!(),
                Err::Error(_) => assert!(!exp_failure),
                Err::Failure(_) => assert!(exp_failure),
            }
        }
    }
}

use super::processing::{DefinitionChunk, SoruceLineKind};
use crate::logging::PreprocessorErrorKind;
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, tag_no_case},
    character::complete::{alphanumeric1, space0, space1},
    combinator::{map, opt},
    error::{ErrorKind, ParseError},
    multi::{many0, many1},
    sequence::{delimited, preceded},
    Err, IResult,
};
use std::path::PathBuf;

/////////////// RESERVED KEYWORDS ///////////////
const PREPRO_CHAR: &str = "%";
const KEYWORD_DEFINE: &str = "%define";
const KEYWORD_INCLUDE: &str = "%include";
/////////////////////////////////////////////////

#[derive(Debug)]
pub struct PreprocessorParseError<'a> {
    _input: &'a str,
    pub kind: PreprocessorErrorKind,
}

impl<'a> ParseError<&'a str> for PreprocessorParseError<'a> {
    fn from_error_kind(input: &'a str, _kind: ErrorKind) -> Self {
        Self {
            _input: input,
            kind: PreprocessorErrorKind::Nom,
        }
    }

    fn append(_input: &'a str, _kind: ErrorKind, other: Self) -> Self {
        other
    }
}

pub fn parse_source_line(input: &str) -> IResult<&str, SoruceLineKind, PreprocessorParseError> {
    alt((parse_define_line, parse_include_line, parse_code_line))(input)
}

/// Parses a normal code line
fn parse_code_line(input: &str) -> IResult<&str, SoruceLineKind, PreprocessorParseError> {
    map(parse_until_end_of_input, |res: &str| -> SoruceLineKind {
        SoruceLineKind::Code(res.to_string())
    })(input)
}

/// Parses %define directive
fn parse_define_line(input: &str) -> IResult<&str, SoruceLineKind, PreprocessorParseError> {
    // Match %define tag
    let (input, _) = preceded(space0, tag_no_case(KEYWORD_DEFINE))(input)?;

    // Get identifier
    let (input, identifier) = match preceded(space1, parse_identifier)(input) {
        Ok(res) => res,
        Err(_) => {
            return Err(Err::Failure(PreprocessorParseError {
                _input: input,
                kind: PreprocessorErrorKind::DefineMissingIdentifier,
            }))
        }
    };

    // Get value
    let (input, value) = match preceded(
        // Type annotation on first space0 is needed but I don't know why
        delimited(space0::<&str, PreprocessorParseError>, tag("="), space0),
        parse_until_end_of_input,
    )(input)
    {
        Ok(res) => res,
        Err(_) => {
            return Err(Err::Failure(PreprocessorParseError {
                _input: input,
                kind: PreprocessorErrorKind::DefineMissingValue(identifier.to_owned()),
            }))
        }
    };

    Ok((input, SoruceLineKind::Define(identifier, value.to_owned())))
}

/// Parses %include directive
fn parse_include_line(input: &str) -> IResult<&str, SoruceLineKind, PreprocessorParseError> {
    // Match %include tag
    let (input, _) = preceded(space0, tag_no_case(KEYWORD_INCLUDE))(input)?;

    // Get include file path
    let (input, file_path) =
        match delimited(space1, alt((parse_string_literal, is_not(" \t"))), space0)(input) {
            Ok(res) => res,
            Err(_) => {
                return Err(Err::Failure(PreprocessorParseError {
                    _input: input,
                    kind: PreprocessorErrorKind::IncludeMissingFilePath,
                }))
            }
        };

    Ok((input, SoruceLineKind::Include(PathBuf::from(file_path))))
}

/// Parse preprocessor identifier
fn parse_identifier(input: &str) -> IResult<&str, String> {
    map(many1(alt((alphanumeric1, is_a("_")))), |res| res.join(""))(input)
}

/// Parser that consumes any string of characters until end of input
fn parse_until_end_of_input<T, E: ParseError<T>>(input: &str) -> IResult<&str, &str, E> {
    Ok(("", input))
}

/// Parse double quote delimited string literal
fn parse_string_literal(input: &str) -> IResult<&str, &str> {
    let (input, res) = delimited(tag("\""), opt(is_not("\"")), tag("\""))(input)?;

    // Handle empty strings
    Ok((
        input,
        match res {
            Some(string) => string,
            None => "",
        },
    ))
}

pub fn parse_definitions_chunks(input: &str) -> IResult<&str, Vec<DefinitionChunk>> {
    many0(parse_definition_chunk)(input)
}

fn parse_definition_chunk(input: &str) -> IResult<&str, DefinitionChunk> {
    alt((parse_definition_reference, parse_code_chunk))(input)
}

fn parse_definition_reference(input: &str) -> IResult<&str, DefinitionChunk> {
    let (input, identifier) = preceded(tag(PREPRO_CHAR), parse_identifier)(input)?;

    Ok((input, DefinitionChunk::DefinitionReference(identifier)))
}

fn parse_code_chunk(input: &str) -> IResult<&str, DefinitionChunk> {
    let (input, code) = is_not("%")(input)?;

    Ok((input, DefinitionChunk::Code(code.to_owned())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_any_character() {
        let tests = [("abc123DEF456", "abc123DEF456", ""), (" ", " ", "")];
        for (input, exp_output, exp_remaining) in tests {
            let res: IResult<&str, &str, PreprocessorParseError> = parse_until_end_of_input(input);
            let (remaining, output) = res.unwrap();
            assert_eq!(output, exp_output);
            assert_eq!(remaining, exp_remaining);
        }
    }

    #[test]
    fn parse_string_literal_succ() {
        let tests = [
            ("\"test\"", "test", ""),
            ("\"ciaoABC123\"notthis   \"", "ciaoABC123", "notthis   \""),
            ("\"\"", "", ""),
        ];
        for (input, exp_output, exp_remaining) in tests {
            let (remaining, output) = parse_string_literal(input).unwrap();
            assert_eq!(output, exp_output);
            assert_eq!(remaining, exp_remaining);
        }
    }

    #[test]
    fn parse_string_literal_err() {
        let _ = parse_string_literal("nostringhere").unwrap_err();
    }

    #[test]
    fn parse_identifier_succ() {
        let tests = [
            ("NICE_IDENTIFIER", "NICE_IDENTIFIER", ""),
            ("alsoANiceIdentifier extra", "alsoANiceIdentifier", " extra"),
            ("test$ciao", "test", "$ciao"),
        ];
        for (input, exp_output, exp_remaining) in tests {
            let (remaining, output) = parse_identifier(input).unwrap();
            assert_eq!(output, exp_output);
            assert_eq!(remaining, exp_remaining);
        }
    }

    #[test]
    fn parse_identifier_err() {
        let tests = [" noidentifierhere", "", "$ciao"];
        for input in tests {
            parse_identifier(input).unwrap_err();
        }
    }

    #[test]
    fn parse_include_line_succ() {
        let tests = [
            (
                "%include ciao",
                SoruceLineKind::Include(PathBuf::from("ciao")),
                "",
            ),
            (
                "     %INCLUDE \"test\"     somextra",
                SoruceLineKind::Include(PathBuf::from("test")),
                "somextra",
            ),
        ];
        for (input, exp_output, exp_remaining) in tests {
            let (remaining, output) = parse_include_line(input).unwrap();
            assert_eq!(output, exp_output);
            assert_eq!(remaining, exp_remaining);
        }
    }

    #[test]
    fn parse_include_line_err() {
        let tests = [("", false), (" lda test test", false), ("%include", true)];
        for (input, exp_failure) in tests {
            let err = parse_include_line(input).unwrap_err();

            match err {
                Err::Incomplete(_) => panic!(),
                Err::Error(_) => assert!(!exp_failure),
                Err::Failure(_) => assert!(exp_failure),
            }
        }
    }

    #[test]
    fn parse_define_line_succ() {
        let tests = [
            (
                "%define _IDENTIFIER = This is a nice value",
                SoruceLineKind::Define("_IDENTIFIER".to_owned(), "This is a nice value".to_owned()),
                "",
            ),
            (
                "%DEFINE   definitelyAValidIdentifier=!=X0   test",
                SoruceLineKind::Define(
                    "definitelyAValidIdentifier".to_owned(),
                    "!=X0   test".to_owned(),
                ),
                "",
            ),
        ];
        for (input, exp_output, exp_remaining) in tests {
            let (remaining, output) = parse_define_line(input).unwrap();
            assert_eq!(output, exp_output);
            assert_eq!(remaining, exp_remaining);
        }
    }

    #[test]
    fn parse_define_line_err() {
        let tests = [
            ("", false),
            (" lda test test", false),
            ("%define", true),
            ("%define test", true),
            ("%define test clasdfs= value", true),
        ];
        for (input, exp_failure) in tests {
            let err = parse_define_line(input).unwrap_err();

            match err {
                Err::Incomplete(_) => panic!(),
                Err::Error(_) => assert!(!exp_failure),
                Err::Failure(_) => assert!(exp_failure),
            }
        }
    }

    #[test]
    fn parse_source_line_succ() {
        let tests = [
            (
                "   lda test, test",
                SoruceLineKind::Code("   lda test, test".to_owned()),
                "",
            ),
            (
                "%define TEST=ciao",
                SoruceLineKind::Define("TEST".to_owned(), "ciao".to_owned()),
                "",
            ),
            (
                "%include \"included.l6s\"",
                SoruceLineKind::Include(PathBuf::from("included.l6s")),
                "",
            ),
        ];
        for (input, exp_output, exp_remaining) in tests {
            let (remaining, output) = parse_source_line(input).unwrap();
            assert_eq!(output, exp_output);
            assert_eq!(remaining, exp_remaining);
        }
    }
}

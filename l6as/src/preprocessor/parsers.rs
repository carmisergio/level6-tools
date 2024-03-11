use super::preprocess::{DefinitionChunk, SourceLineBody};
use crate::logging::PreprocessorErrorKind;
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, tag_no_case, take},
    character::complete::{alphanumeric1, space0, space1},
    combinator::{map, opt, value},
    error::{ErrorKind, ParseError},
    multi::{fold_many0, many0, many1},
    sequence::{delimited, preceded, tuple},
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
    fn from_error_kind(input: &'a str, kind: ErrorKind) -> Self {
        Self {
            _input: input,
            kind: PreprocessorErrorKind::Nom(kind),
        }
    }

    fn append(_input: &'a str, _kind: ErrorKind, other: Self) -> Self {
        other
    }
}

pub fn parse_source_line(
    input: &str,
) -> IResult<&str, (SourceLineBody, String, String), PreprocessorParseError> {
    // Divide code from comment
    let (body_str, comment) = divide_comment(input);

    // Handle empty lines
    if body_str.len() < 1 {
        return Ok((
            "",
            (SourceLineBody::Empty, comment.to_owned(), "".to_owned()),
        ));
    }

    // Parse line body
    let (garbage, body) = parse_source_line_body(body_str)?;

    Ok(("", (body, comment.to_owned(), garbage.to_owned())))
}

fn parse_source_line_body(input: &str) -> IResult<&str, SourceLineBody, PreprocessorParseError> {
    alt((
        parse_define_line_body,
        parse_include_line_body,
        parse_code_line_body,
    ))(input)
}

fn parse_code_line_body(input: &str) -> IResult<&str, SourceLineBody, PreprocessorParseError> {
    match map(
        fold_many0(
            alt((
                parse_string_literal_block,
                map(value(" ", is_a(" \t")), |res| res.to_owned()),
                map(is_not(" \t"), |res: &str| res.to_owned()),
            )),
            String::new,
            |mut acc: String, item: String| {
                acc.push_str(&item);
                acc
            },
        ),
        |res: String| -> SourceLineBody { SourceLineBody::Code(res.trim().to_owned()) },
    )(input)
    {
        Ok(res) => Ok(res),
        Err(_err) => Err(Err::Failure(PreprocessorParseError {
            _input: input,
            kind: PreprocessorErrorKind::Unknown,
        })),
    }
}

/// Parses %define directive
fn parse_define_line_body(input: &str) -> IResult<&str, SourceLineBody, PreprocessorParseError> {
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
        is_not(";"),
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

    Ok((
        input,
        SourceLineBody::Define(identifier, value.trim().to_owned()),
    ))
}

/// Parses %include directive
fn parse_include_line_body(input: &str) -> IResult<&str, SourceLineBody, PreprocessorParseError> {
    // Match %include tag
    let (input, _) = preceded(space0, tag_no_case(KEYWORD_INCLUDE))(input)?;

    // Get include file path
    let (input, file_path) =
        match delimited(space1, alt((parse_string_literal, is_not(" \t;"))), space0)(input) {
            Ok(res) => res,
            Err(_) => {
                return Err(Err::Failure(PreprocessorParseError {
                    _input: input,
                    kind: PreprocessorErrorKind::IncludeMissingFilePath,
                }))
            }
        };

    Ok((input, SourceLineBody::Include(PathBuf::from(file_path))))
}

/// Parse preprocessor identifier
fn parse_identifier(input: &str) -> IResult<&str, String> {
    map(many1(alt((alphanumeric1, is_a("_")))), |res| res.join(""))(input)
}

fn divide_comment(input: &str) -> (&str, &str) {
    for (pos, character) in input.char_indices() {
        if character == ';' {
            // Return (code, comment)
            return (&input[0..pos].trim(), &input[pos..].trim());
        }
    }

    (input.trim(), "")
}

/// Parse double quote delimited string literal
fn parse_string_literal(input: &str) -> IResult<&str, &str> {
    map(
        delimited(tag("\""), opt(is_not("\"")), tag("\"")),
        |res| match res {
            Some(val) => val,
            None => "",
        },
    )(input)
}

fn parse_string_literal_block(input: &str) -> IResult<&str, String> {
    let (input, (_, cont, _)) = tuple((
        tag("\""),
        fold_many0(
            alt((
                map(is_not("\"\\"), |val: &str| val.to_owned()),
                parse_string_escaped_block,
            )),
            String::new,
            |mut acc: String, item| {
                acc.push_str(&item);
                acc
            },
        ),
        tag("\""),
    ))(input)?;

    Ok((input, format!("\"{}\"", cont)))
}

fn parse_string_escaped_block(input: &str) -> IResult<&str, String> {
    let (input, (escaper, escaped)) = tuple((tag("\\"), take(1 as usize)))(input)?;
    // let (input, (escaper, escaped)) = tuple((tag( as usize), take(1 as usize)))(input)?;
    Ok((input, escaper.to_owned() + escaped))
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
    let (input, code) = alt((
        map(is_not("%\""), |res: &str| res.to_owned()),
        parse_string_literal_block,
    ))(input)?;

    Ok((input, DefinitionChunk::Code(code.to_owned())))
}

#[cfg(test)]
mod tests {
    use super::{divide_comment, *};

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
            ("test;ciao", "test", ";ciao"),
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
                "%include ciao;test",
                SourceLineBody::Include(PathBuf::from("ciao")),
                ";test",
            ),
            (
                "     %INCLUDE \"test\"     somextra ; ciao",
                SourceLineBody::Include(PathBuf::from("test")),
                "somextra ; ciao",
            ),
        ];
        for (input, exp_output, exp_remaining) in tests {
            let (remaining, output) = parse_include_line_body(input).unwrap();
            assert_eq!(output, exp_output);
            assert_eq!(remaining, exp_remaining);
        }
    }

    #[test]
    fn parse_code_line_body_succ() {
        let tests = [
            (
                "     lda           $R1,\t 1234",
                SourceLineBody::Code("lda $R1, 1234".to_owned()),
                "",
            ),
            (
                "arg1     arg2,arg3\targ4",
                SourceLineBody::Code("arg1 arg2,arg3 arg4".to_owned()),
                "",
            ),
            ("", SourceLineBody::Code("".to_owned()), ""),
            ("              ", SourceLineBody::Code("".to_owned()), ""),
            // (
            //     "      \"    CIAO     \"        ",
            //     SourceLineBody::Code("\"    CIAO     \"".to_owned()),
            //     "",
            // ),
            (
                "ldr, \"string\"",
                SourceLineBody::Code("ldr, \"string\"".to_owned()),
                "",
            ),
        ];
        for (input, exp_output, exp_remaining) in tests {
            let (remaining, output) = parse_code_line_body(input).unwrap();
            assert_eq!(output, exp_output);
            assert_eq!(remaining, exp_remaining);
        }
    }

    #[test]
    fn parse_include_line_err() {
        let tests = [("", false), (" lda test test", false), ("%include", true)];
        for (input, exp_failure) in tests {
            let err = parse_include_line_body(input).unwrap_err();

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
                "%define _IDENTIFIER = This is a nice value ; And comment",
                SourceLineBody::Define("_IDENTIFIER".to_owned(), "This is a nice value".to_owned()),
                "; And comment",
            ),
            (
                "%DEFINE   definitelyAValidIdentifier=!=X0   test   ",
                SourceLineBody::Define(
                    "definitelyAValidIdentifier".to_owned(),
                    "!=X0   test".to_owned(),
                ),
                "",
            ),
        ];
        for (input, exp_output, exp_remaining) in tests {
            let (remaining, output) = parse_define_line_body(input).unwrap();
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
            let err = parse_define_line_body(input).unwrap_err();

            match err {
                Err::Incomplete(_) => panic!(),
                Err::Error(_) => assert!(!exp_failure),
                Err::Failure(_) => assert!(exp_failure),
            }
        }
    }

    #[test]
    fn parse_source_line_body_succ() {
        let tests = [
            (
                "   lda test, test",
                SourceLineBody::Code("lda test, test".to_owned()),
                "",
            ),
            (
                "%define TEST=ciao",
                SourceLineBody::Define("TEST".to_owned(), "ciao".to_owned()),
                "",
            ),
            (
                "%include \"included.l6s\"",
                SourceLineBody::Include(PathBuf::from("included.l6s")),
                "",
            ),
        ];
        for (input, exp_output, exp_remaining) in tests {
            let (remaining, output) = parse_source_line_body(input).unwrap();
            assert_eq!(output, exp_output);
            assert_eq!(remaining, exp_remaining);
        }
    }

    #[test]
    fn test_divide_comment() {
        let tests = [
            ("; comment", "", "; comment"),
            ("code ; comment", "code", "; comment"),
            ("code ", "code", ""),
        ];
        for (input, exp_code, exp_comment) in tests {
            let (code, comment) = divide_comment(input);
            assert_eq!(code, exp_code);
            assert_eq!(comment, exp_comment);
        }
    }

    #[test]
    fn parse_string_literal_block_succ() {
        let tests = [
            (
                "\"this is a string literal\"",
                "\"this is a string literal\"",
                "",
            ),
            ("\"String\" notstring", "\"String\"", " notstring"),
            (
                "\"String with \\\" \" notstring",
                "\"String with \\\" \"",
                " notstring",
            ),
        ];
        for (input, exp_output, exp_remaining) in tests {
            let (remaining, output) = parse_string_literal_block(input).unwrap();
            assert_eq!(output, exp_output);
            assert_eq!(remaining, exp_remaining);
        }
    }
}

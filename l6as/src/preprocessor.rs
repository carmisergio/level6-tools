use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, tag_no_case},
    character::complete::{alphanumeric1, space0, space1},
    combinator::{map, opt},
    error::{ErrorKind, ParseError},
    multi::{many1, separated_list0},
    sequence::{delimited, pair, preceded},
    Err, IResult,
};
use std::path::PathBuf;
use std::vec;

use super::logging::{
    print_preprocessor_error, print_preprocessor_warning, PreprocessorError, PreprocessorWarning,
    PreprocessorWarningKind,
};

/////////////// RESERVED KEYWORDS ///////////////
// const PREPRO_CHAR: char = '%';
const KEYWORD_DEFINE: &str = "%define";
const KEYWORD_INCLUDE: &str = "%include";
const KEYWORD_MACRO: &str = "%macro";
const KEYWORD_END_MACRO: &str = "%endm";
/////////////////////////////////////////////////

#[derive(Debug)]
pub enum PreprocessorParseErrorKind {
    IncludeMissingFilePath,
    DefineMissingIdentifier,
    DefineMissingValue(String),
    MacroMissingIdentifier,
    MacroMalformedArguments(String),
    Nom,
}

#[derive(Debug)]
struct PreprocessorParseError<'a> {
    _input: &'a str,
    kind: PreprocessorParseErrorKind,
}

impl<'a> ParseError<&'a str> for PreprocessorParseError<'a> {
    fn from_error_kind(input: &'a str, _kind: ErrorKind) -> Self {
        Self {
            _input: input,
            kind: PreprocessorParseErrorKind::Nom,
        }
    }

    fn append(_input: &'a str, _kind: ErrorKind, other: Self) -> Self {
        other
    }
}

#[derive(Debug, PartialEq)]
pub enum SourceLine {
    Define(String, String),
    Include(PathBuf),
    Macro(String, Vec<String>),
    EndMacro,
    Code(String),
}

/// Parses input source code to a vector of `SourceLine`s
pub fn parse_source_lines(input: &str) -> Result<Vec<SourceLine>, Vec<SourceLine>> {
    let mut lines: Vec<SourceLine> = vec![];

    // Keep track of if an error has been encountered
    let mut error_encountered = false;

    // Get all lines from file
    for (line_n, raw_line) in input.lines().enumerate() {
        // Parse line
        let (remaining, line) = match parse_source_line(raw_line) {
            Ok(line) => line,
            Err(err) => {
                match err {
                    Err::Failure(err) => print_preprocessor_error(PreprocessorError {
                        line_n,
                        file_name: PathBuf::from("test.l6s"),
                        line: raw_line.to_owned(),
                        kind: err.kind,
                    }),
                    Err::Error(_) => {}
                    Err::Incomplete(_) => {}
                }

                // An error has been encountered
                error_encountered = true;

                continue;
            }
        };

        // Check if there is still unparsed stuff
        if remaining.len() > 0 {
            print_preprocessor_warning(PreprocessorWarning {
                line_n,
                file_name: PathBuf::from("test.l6s"),
                line: raw_line.to_owned(),
                kind: PreprocessorWarningKind::GarbageAtEndOfLine(remaining.to_owned()),
            });
        }

        lines.push(line);
    }

    // Return results
    match error_encountered {
        false => Ok(lines),
        true => Err(lines),
    }
}

fn parse_source_line(input: &str) -> IResult<&str, SourceLine, PreprocessorParseError> {
    alt((
        parse_define_line,
        parse_include_line,
        parse_macro_definition_line,
        parse_endm_line,
        parse_code_line,
    ))(input)
}

/// Parses a normal code line
fn parse_code_line(input: &str) -> IResult<&str, SourceLine, PreprocessorParseError> {
    map(any_character, |res: &str| -> SourceLine {
        SourceLine::Code(res.to_string())
    })(input)
}

/// Parses %define directive
fn parse_define_line(input: &str) -> IResult<&str, SourceLine, PreprocessorParseError> {
    // Match %define tag
    let (input, _) = preceded(space0, tag_no_case(KEYWORD_DEFINE))(input)?;

    // Get identifier
    let (input, identifier) = match preceded(space1, parse_identifier)(input) {
        Ok(res) => res,
        Err(_) => {
            return Err(Err::Failure(PreprocessorParseError {
                _input: input,
                kind: PreprocessorParseErrorKind::DefineMissingIdentifier,
            }))
        }
    };

    // Get value
    let (input, value) = match preceded(
        // Type annotation on first space0 is needed but I don't know why
        delimited(space0::<&str, PreprocessorParseError>, tag("="), space0),
        any_character,
    )(input)
    {
        Ok(res) => res,
        Err(_) => {
            return Err(Err::Failure(PreprocessorParseError {
                _input: input,
                kind: PreprocessorParseErrorKind::DefineMissingValue(identifier.to_owned()),
            }))
        }
    };

    Ok((input, SourceLine::Define(identifier, value.to_owned())))
}

/// Parses %include directive
fn parse_include_line(input: &str) -> IResult<&str, SourceLine, PreprocessorParseError> {
    // Match %include tag
    let (input, _) = preceded(space0, tag_no_case(KEYWORD_INCLUDE))(input)?;

    // Get include file path
    let (input, file_path) =
        match delimited(space1, alt((parse_string_literal, is_not(" \t"))), space0)(input) {
            Ok(res) => res,
            Err(_) => {
                return Err(Err::Failure(PreprocessorParseError {
                    _input: input,
                    kind: PreprocessorParseErrorKind::IncludeMissingFilePath,
                }))
            }
        };

    Ok((input, SourceLine::Include(PathBuf::from(file_path))))
}

/// Parses %macro directive
fn parse_macro_definition_line(input: &str) -> IResult<&str, SourceLine, PreprocessorParseError> {
    // Match %macro tag
    let (input, _) = preceded(space0, tag_no_case(KEYWORD_MACRO))(input)?;

    // Get identifier
    let (input, identifier) = match delimited(space1, parse_identifier, space0)(input) {
        Ok(res) => res,
        Err(_) => {
            return Err(Err::Failure(PreprocessorParseError {
                _input: input,
                kind: PreprocessorParseErrorKind::MacroMissingIdentifier,
            }))
        }
    };

    // Get argument list
    let (input, arguments) = match delimited(
        pair(tag("("), space0),
        separated_list0(pair(tag(","), space0), parse_identifier),
        pair(space0, tag(")")),
    )(input)
    {
        Ok(res) => res,
        Err(_) => {
            return Err(Err::Failure(PreprocessorParseError {
                _input: input,
                kind: PreprocessorParseErrorKind::MacroMalformedArguments(identifier),
            }))
        }
    };

    // TODO fix this bad code
    let mut args_string: Vec<String> = vec![];
    for arg in arguments {
        args_string.push(arg.to_owned());
    }
    Ok((input, SourceLine::Macro(identifier, args_string)))
}

/// Parses %endm directive
fn parse_endm_line(input: &str) -> IResult<&str, SourceLine, PreprocessorParseError> {
    map(
        delimited(space0, tag_no_case(KEYWORD_END_MACRO), space0),
        |_| SourceLine::EndMacro,
    )(input)
}

/// Parse preprocessor identifier
fn parse_identifier(input: &str) -> IResult<&str, String, PreprocessorParseError> {
    map(many1(alt((alphanumeric1, is_a("_")))), |res| res.join(""))(input)
}

/// Parser that consumes any string of characters until end of input
fn any_character<T, E: ParseError<T>>(input: &str) -> IResult<&str, &str, E> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_any_character() {
        let tests = [("abc123DEF456", "abc123DEF456", ""), (" ", " ", "")];
        for (input, exp_output, exp_remaining) in tests {
            let res: IResult<&str, &str, PreprocessorParseError> = any_character(input);
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
    fn parse_endm_line_succ() {
        let tests = [
            ("%endm", SourceLine::EndMacro, ""),
            ("   %ENDM extra", SourceLine::EndMacro, "extra"),
        ];
        for (input, exp_output, exp_remaining) in tests {
            let (remaining, output) = parse_endm_line(input).unwrap();
            assert_eq!(output, exp_output);
            assert_eq!(remaining, exp_remaining);
        }
    }

    #[test]
    fn parse_endm_line_err() {
        let tests = ["noendmacro", "", "ciao %endm", "%orazio"];
        for input in tests {
            parse_endm_line(input).unwrap_err();
        }
    }

    #[test]
    fn parse_include_line_succ() {
        let tests = [
            (
                "%include ciao",
                SourceLine::Include(PathBuf::from("ciao")),
                "",
            ),
            (
                "     %INCLUDE \"test\"     somextra",
                SourceLine::Include(PathBuf::from("test")),
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
                SourceLine::Define("_IDENTIFIER".to_owned(), "This is a nice value".to_owned()),
                "",
            ),
            (
                "%DEFINE   definitelyAValidIdentifier=!=X0   test",
                SourceLine::Define(
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
    fn parse_macro_definition_line_succ() {
        let tests = [
            (
                "%macro DO_THINGS(a,defg, an_argument   )",
                SourceLine::Macro(
                    "DO_THINGS".to_owned(),
                    vec!["a".to_owned(), "defg".to_owned(), "an_argument".to_owned()],
                ),
                "",
            ),
            (
                "%MACRO ident()",
                SourceLine::Macro("ident".to_owned(), vec![]),
                "",
            ),
        ];
        for (input, exp_output, exp_remaining) in tests {
            let (remaining, output) = parse_macro_definition_line(input).unwrap();
            assert_eq!(output, exp_output);
            assert_eq!(remaining, exp_remaining);
        }
    }

    #[test]
    fn parse_macro_definition_line_err() {
        let tests = [
            ("", false),
            (" lda test test", false),
            ("%macro", true),
            ("%macro id", true),
            ("%macro ciao(", true),
            ("%macro ciao(,)", true),
        ];
        for (input, exp_failure) in tests {
            let err = parse_macro_definition_line(input).unwrap_err();

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
                SourceLine::Code("   lda test, test".to_owned()),
                "",
            ),
            (
                "%define TEST=ciao",
                SourceLine::Define("TEST".to_owned(), "ciao".to_owned()),
                "",
            ),
            (
                "%include \"included.l6s\"",
                SourceLine::Include(PathBuf::from("included.l6s")),
                "",
            ),
            (
                "%macro TEST(a, b, c)",
                SourceLine::Macro(
                    "TEST".to_owned(),
                    vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
                ),
                "",
            ),
            ("%endm", SourceLine::EndMacro, ""),
        ];
        for (input, exp_output, exp_remaining) in tests {
            let (remaining, output) = parse_source_line(input).unwrap();
            assert_eq!(output, exp_output);
            assert_eq!(remaining, exp_remaining);
        }
    }

    #[test]
    fn parse_source_lines_succ() {
        let input = "%macro TEST()\n   INST\n%endm\n%define def=12\n    CODE this is some code ;Comment   \n%include file_path";

        let res = parse_source_lines(input).unwrap();

        assert_eq!(res[0], SourceLine::Macro("TEST".to_owned(), vec![]));
        assert_eq!(res[1], SourceLine::Code("   INST".to_owned()));
        assert_eq!(res[2], SourceLine::EndMacro);
        assert_eq!(
            res[3],
            SourceLine::Define("def".to_owned(), "12".to_owned())
        );
        assert_eq!(
            res[4],
            SourceLine::Code("    CODE this is some code ;Comment   ".to_owned())
        );
        assert_eq!(res[5], SourceLine::Include(PathBuf::from("file_path")));
    }

    #[test]
    fn parse_source_lines_err() {
        let input = "%macro TEST()\n   INST\n%endm\n%define\n    CODE this is some code ;Comment   \n%include file_path";

        parse_source_lines(input).unwrap_err();
    }
}

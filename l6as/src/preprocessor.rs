use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, tag_no_case},
    character::complete::{alphanumeric1, space0, space1},
    combinator::map,
    multi::{many1, separated_list0},
    sequence::{delimited, pair, tuple},
    IResult,
};
use std::path::PathBuf;
use std::vec;

/*
 * TODO
 * Add custom error types to handle stuff like missing macro name, etc
 */

/////////////// RESERVED KEYWORDS ///////////////
// const PREPRO_CHAR: char = '%';
const KEYWORD_DEFINE: &str = "%define";
const KEYWORD_INCLUDE: &str = "%include";
const KEYWORD_MACRO: &str = "%macro";
const KEYWORD_END_MACRO: &str = "%endm";
/////////////////////////////////////////////////

#[derive(Debug, PartialEq)]
pub enum SourceLine {
    Define(String, String),
    Include(PathBuf),
    Macro(String, Vec<String>),
    EndMacro,
    Code(String),
}

/// Separates a string to a vector of strings, one per line
pub fn parse_source_lines(input: &str) -> Result<Vec<SourceLine>, String> {
    let mut lines: Vec<SourceLine> = vec![];

    // Get all lines from file
    for (line_n, raw_line) in input.lines().enumerate() {
        // Parse line
        let (remaining, line) = match parse_source_line(raw_line) {
            Ok(line) => line,
            Err(_) => return Err(format!("Line division error on line {}", line_n)),
        };

        // Check if there is still unparsed stuff
        if remaining.len() > 0 {
            println!("Warning: garbage on end of line {}", line_n);
        }

        lines.push(line);
    }

    Ok(lines)
}

fn parse_source_line(input: &str) -> IResult<&str, SourceLine> {
    alt((
        parse_define_line,
        parse_include_line,
        parse_macro_definition_line,
        parse_endm_line,
        parse_code_line,
    ))(input)
}

/// Parses a normal code line
fn parse_code_line(input: &str) -> IResult<&str, SourceLine> {
    map(any_character, |res: &str| -> SourceLine {
        SourceLine::Code(res.to_string())
    })(input)
}

/// Parses %define directive
fn parse_define_line(input: &str) -> IResult<&str, SourceLine> {
    // Parse definition
    map(
        tuple((
            space0,
            tag_no_case(KEYWORD_DEFINE),
            space1,
            parse_identifier,
            space0,
            tag("="),
            space0,
            any_character,
        )),
        |(_, _, _, identifier, _, _, _, value)| SourceLine::Define(identifier, value.to_owned()),
    )(input)
}

/// Parses %include directive
fn parse_include_line(input: &str) -> IResult<&str, SourceLine> {
    map(
        tuple((
            space0,
            tag_no_case(KEYWORD_INCLUDE),
            space1,
            alt((parse_string_literal, is_not(" \t"))),
            space0,
        )),
        |(_, _, _, file_path, _)| SourceLine::Include(PathBuf::from(file_path)),
    )(input)
}

/// Parses %macro directive
fn parse_macro_definition_line(input: &str) -> IResult<&str, SourceLine> {
    map(
        tuple((
            space0,
            tag_no_case(KEYWORD_MACRO),
            space1,
            parse_identifier,
            pair(tag("("), space0),
            separated_list0(pair(tag(","), space0), parse_identifier),
            pair(space0, tag(")")),
            space0,
        )),
        |(_, _, _, identifier, _, arguments, _, _)| {
            println!("{}: {:?}", identifier, arguments);

            // TODO fix this bad code
            let mut args_string: Vec<String> = vec![];
            for arg in arguments {
                args_string.push(arg.to_owned());
            }

            SourceLine::Macro(identifier, args_string)
        },
    )(input)
}

/// Parses %endm directive
fn parse_endm_line(input: &str) -> IResult<&str, SourceLine> {
    map(
        delimited(space0, tag_no_case(KEYWORD_END_MACRO), space0),
        |_| SourceLine::EndMacro,
    )(input)
}

/// Parse preprocessor identifier
fn parse_identifier(input: &str) -> IResult<&str, String> {
    map(many1(alt((alphanumeric1, is_a("_")))), |res| res.join(""))(input)
}

/// Parser that consumes any string of characters until end of input
fn any_character(input: &str) -> IResult<&str, &str> {
    Ok(("", input))
}

/// Parse double quote delimited string literal
fn parse_string_literal(input: &str) -> IResult<&str, &str> {
    delimited(tag("\""), is_not("\""), tag("\""))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_any_character() {
        let tests = [("abc123DEF456", "abc123DEF456", ""), (" ", " ", "")];
        for (input, exp_output, exp_remaining) in tests {
            let (remaining, output) = any_character(input).unwrap();
            assert_eq!(output, exp_output);
            assert_eq!(remaining, exp_remaining);
        }
    }

    #[test]
    fn parse_string_literal_succ() {
        let tests = [
            ("\"test\"", "test", ""),
            ("\"ciaoABC123\"notthis   \"", "ciaoABC123", "notthis   \""),
        ];
        for (input, exp_output, exp_remaining) in tests {
            let (remaining, output) = parse_string_literal(input).unwrap();
            assert_eq!(output, exp_output);
            assert_eq!(remaining, exp_remaining);
        }
    }

    #[test]
    #[should_panic]
    fn parse_string_literal_err() {
        parse_string_literal("nostringhere").unwrap();
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
    #[should_panic]
    fn parse_identifier_err() {
        parse_identifier(" noidentifierhere").unwrap();
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
    #[should_panic]
    fn parse_endm_line_err() {
        parse_endm_line(" there seems to be no endm here %endm ").unwrap();
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
    #[should_panic]
    fn parse_include_line_err1() {
        parse_include_line(" %endm ").unwrap();
    }

    #[test]
    #[should_panic]
    fn parse_include_line_err2() {
        parse_include_line(" %include ").unwrap();
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
    #[should_panic]
    fn parse_define_line_err1() {
        parse_define_line(" some other thing ").unwrap();
    }

    #[test]
    #[should_panic]
    fn parse_define_line_err2() {
        parse_define_line("% define").unwrap();
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
    #[should_panic]
    fn parse_macro_definition_err() {
        parse_macro_definition_line(" ").unwrap();
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
}

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
use std::{collections::HashMap, vec};

use super::file::{FileInclusionCoordinator, FileInclusionError};

use super::logging::{
    print_preprocessor_error, print_preprocessor_warning, PreprocessorError, PreprocessorErrorKind,
    PreprocessorWarning, PreprocessorWarningKind,
};

/////////////// RESERVED KEYWORDS ///////////////
const PREPRO_CHAR: &str = "%";
const KEYWORD_DEFINE: &str = "%define";
const KEYWORD_INCLUDE: &str = "%include";
/////////////////////////////////////////////////

#[derive(Debug)]
struct PreprocessorParseError<'a> {
    _input: &'a str,
    kind: PreprocessorErrorKind,
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

#[derive(Debug, PartialEq, Clone)]
pub enum SoruceLineKind {
    Define(String, String),
    Include(PathBuf),
    Code(String),
}
#[derive(Debug, PartialEq, Clone)]
pub struct LineLocation {
    pub line_n: usize,
    pub file_name: PathBuf,
    pub raw_content: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SourceLine {
    kind: SoruceLineKind,
    location: LineLocation,
}

/// Preprocess a program
pub fn preprocess(
    file_path: &PathBuf,
    fi_coord: &mut FileInclusionCoordinator,
) -> Result<Vec<String>, Vec<String>> {
    let mut error_encountered = false;

    // Parse the source file (resolving all includes)
    let source_lines = match parse_source_file(file_path, fi_coord) {
        Ok(lines) => lines,
        Err(lines) => {
            error_encountered = true;
            lines
        }
    };

    // Process %defines
    let source_lines = match process_defines(&source_lines) {
        Ok(lines) => lines,
        Err(lines) => {
            error_encountered = true;
            lines
        }
    };

    // Flatten all code lines to string
    let res = source_lines_to_string(&source_lines);

    // Return results
    match error_encountered {
        false => Ok(res),
        true => Err(res),
    }
}

/// Parses input source file to a vector of `SourceLine`s, resolving includes
pub fn parse_source_file(
    file_path: &PathBuf,
    fi_coord: &mut FileInclusionCoordinator,
) -> Result<Vec<SourceLine>, Vec<SourceLine>> {
    let mut error_encountered = false;

    // Read file
    let (abs_path, code) = match fi_coord.read_file(&file_path) {
        Ok(res) => res,
        Err(err) => {
            // Log error
            match err {
                FileInclusionError::FileNotFound(file_path) => {
                    print_preprocessor_error(PreprocessorError {
                        kind: PreprocessorErrorKind::CannotOpenSourceFile(file_path),
                        location: None,
                    });
                }
                FileInclusionError::DoubleInclusion(file_path) => {
                    print_preprocessor_error(PreprocessorError {
                        kind: PreprocessorErrorKind::DobleInclusion(file_path),
                        location: None,
                    });
                }
            }

            return Err(vec![]);
        }
    };

    // Parse source lines
    let lines = match parse_source_string(&code, &abs_path) {
        Ok(lines) => lines,
        Err(lines) => {
            error_encountered = true;
            lines
        }
    };

    // Process %includes
    let lines = match process_includes(&lines, fi_coord) {
        Ok(res) => res,
        Err(lines) => {
            error_encountered = true;
            lines
        }
    };

    // Return results
    match error_encountered {
        false => Ok(lines),
        true => Err(lines),
    }
}

/// Converts a Vec of SourceLines (containing only Code) to strings,
/// fails if vec contains other types of lines
pub fn source_lines_to_string(input: &[SourceLine]) -> Vec<String> {
    let mut strings: Vec<String> = vec![];

    // Go over all lines
    for line in input {
        if let SoruceLineKind::Code(content) = &line.kind {
            strings.push(content.clone());
        } else {
            panic!();
        }
    }

    strings
}

/*
 * Parsing and %include processing
 */

/// Parses input source code string to a vector of `SourceLine`s
fn parse_source_string(
    input: &str,
    file_name: &PathBuf,
) -> Result<Vec<SourceLine>, Vec<SourceLine>> {
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
                        kind: err.kind,
                        location: Some(LineLocation {
                            line_n: line_n + 1,
                            file_name: file_name.clone(),
                            raw_content: raw_line.to_owned(),
                        }),
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
                line_n: line_n + 1,
                file_name: file_name.clone(),
                line: raw_line.to_owned(),
                kind: PreprocessorWarningKind::GarbageAtEndOfLine(remaining.to_owned()),
            });
        }

        lines.push(SourceLine {
            kind: line,
            location: LineLocation {
                line_n: line_n + 1,
                file_name: file_name.clone(),
                raw_content: raw_line.to_owned(),
            },
        });
    }

    // Return results
    match error_encountered {
        false => Ok(lines),
        true => Err(lines),
    }
}

fn process_includes(
    input: &Vec<SourceLine>,
    fi_coord: &mut FileInclusionCoordinator,
) -> Result<Vec<SourceLine>, Vec<SourceLine>> {
    let mut output: Vec<SourceLine> = vec![];
    let mut error = false;

    for line in input {
        // If line is include, resolve it. Otherwise copy line
        if let SoruceLineKind::Include(file_path) = &line.kind {
            // Process new file
            let mut included_lines = match parse_source_file(&file_path, fi_coord) {
                Ok(lines) => lines,
                Err(lines) => {
                    error = true;
                    lines
                }
            };
            output.append(&mut included_lines)
        } else {
            output.push(line.clone())
        }
    }

    // Return results
    match error {
        false => Ok(output),
        true => Err(output),
    }
}

/*
 * Nom parsers
 */

fn parse_source_line(input: &str) -> IResult<&str, SoruceLineKind, PreprocessorParseError> {
    alt((parse_define_line, parse_include_line, parse_code_line))(input)
}

/// Parses a normal code line
fn parse_code_line(input: &str) -> IResult<&str, SoruceLineKind, PreprocessorParseError> {
    map(any_character, |res: &str| -> SoruceLineKind {
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
        any_character,
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

/*
 * Definition processing
 */

fn process_defines(input: &[SourceLine]) -> Result<Vec<SourceLine>, Vec<SourceLine>> {
    let mut res: Vec<SourceLine> = vec![];
    let mut error_encountered = false;

    // Definition table
    let mut definition_table: HashMap<String, String> = HashMap::new();

    // Construct definition table
    for line in input {
        if let SoruceLineKind::Define(identifier, value) = &line.kind {
            match definition_table.get(identifier) {
                // Check if this identifier was already defined
                Some(_) => {
                    print_preprocessor_error(PreprocessorError {
                        kind: PreprocessorErrorKind::DefineMultipleDefinition(identifier.clone()),
                        location: Some(line.location.clone()),
                    });
                    error_encountered = true;
                }
                None => {
                    definition_table.insert(identifier.clone(), value.clone());
                }
            }
        }
    }

    // Resolve definitions
    for line in input {
        if let SoruceLineKind::Code(code) = &line.kind {
            match resolve_defines(&code, &definition_table, &line.location) {
                Ok(code) => res.push(SourceLine {
                    kind: SoruceLineKind::Code(code),
                    location: line.location.clone(),
                }),
                Err(_) => error_encountered = true,
            }
        }
    }

    match error_encountered {
        false => Ok(res),
        true => Err(res),
    }
}

#[derive(Debug)]
enum DefinitionChunk {
    Code(String),
    DefinitionReference(String),
}

fn resolve_defines(
    code: &str,
    def_table: &HashMap<String, String>,
    location: &LineLocation,
) -> Result<String, ()> {
    let (_, chunks) = match parse_definitions_chunks(code) {
        Ok(chunks) => chunks,
        Err(_) => return Err(()),
    };

    let mut result: String = "".to_owned();

    for chunk in chunks {
        match chunk {
            DefinitionChunk::Code(code) => result.push_str(&code),
            DefinitionChunk::DefinitionReference(identifier) => {
                // Look for definition in table
                match def_table.get(&identifier) {
                    Some(value) => result.push_str(&value),
                    None => {
                        print_preprocessor_error(PreprocessorError {
                            kind: PreprocessorErrorKind::DefineUndefined(identifier),
                            location: Some(location.clone()),
                        });
                        return Err(());
                    }
                }
            }
        }
    }

    Ok(result)
}

fn parse_definitions_chunks(input: &str) -> IResult<&str, Vec<DefinitionChunk>> {
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

    // #[test]
    // fn parse_source_lines_succ() {
    //     let input =
    //         "   INST\n%define def=12\n    CODE this is some code ;Comment   \n%include file_path";

    //     let res = parse_source_string(input, &PathBuf::from("notimportant")).unwrap();

    //     assert_eq!(res[0], SoruceLineKind::Code("   INST".to_owned()));
    //     assert_eq!(
    //         res[1],
    //         SoruceLineKind::Define("def".to_owned(), "12".to_owned())
    //     );
    //     assert_eq!(
    //         res[2],
    //         SoruceLineKind::Code("    CODE this is some code ;Comment   ".to_owned())
    //     );
    //     assert_eq!(res[3], SoruceLineKind::Include(PathBuf::from("file_path")));
    // }

    // #[test]
    // fn parse_source_lines_err() {
    //     let input = "%macro TEST()\n   INST\n%endmacro\n%define\n    CODE this is some code ;Comment   \n%include file_path";

    //     parse_source_string(input, &PathBuf::from("notimportant")).unwrap_err();
    // }
}

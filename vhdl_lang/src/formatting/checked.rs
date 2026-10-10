// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::data::{ContentReader, Source};
use crate::syntax::VHDLParser;
use crate::syntax::{Kind, Token, TokenComments, Tokenizer};
use crate::VHDLFormatter;
use std::fmt::{Display, Formatter};

/// The reason why a source was not formatted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatError {
    SyntaxErrors,
    /// The formatter would drop regions between `-- vhdl_ls off` and `-- vhdl_ls on`.
    IgnoredRegion,
    /// The formatter would drop tool directives.
    ToolDirective,
    /// The formatted code has different tokens or comments than the original code.
    ChangedCode,
}

impl Display for FormatError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let reason = match self {
            FormatError::SyntaxErrors => "the file has syntax errors",
            FormatError::IgnoredRegion => "the file contains a region ignored with `vhdl_ls off`",
            FormatError::ToolDirective => "the file contains a tool directive",
            FormatError::ChangedCode => "formatting would change the code",
        };
        write!(f, "Cannot format: {reason}")
    }
}

/// Formats a source and checks that the result contains the same tokens and comments.
pub fn format_source(parser: &VHDLParser, source: &Source) -> Result<String, FormatError> {
    check_formattable(parser, source)?;
    let mut diagnostics = Vec::new();
    let design_file = parser.parse_design_source(source, &mut diagnostics);
    if !diagnostics.is_empty() {
        return Err(FormatError::SyntaxErrors);
    }
    let mut formatted = VHDLFormatter::format_design_file(&design_file);
    let contents = source.contents();
    let ends_with_newline = contents
        .get_line(contents.num_lines().saturating_sub(1))
        .is_some_and(|line| line.ends_with('\n'));
    if ends_with_newline && !formatted.ends_with('\n') {
        formatted.push('\n');
    }
    let formatted_file = parser.parse_design_source(
        &Source::inline(source.file_name(), &formatted),
        &mut diagnostics,
    );
    let tokens = design_file
        .design_units
        .iter()
        .flat_map(|(tokens, _)| tokens);
    let formatted_tokens = formatted_file
        .design_units
        .iter()
        .flat_map(|(tokens, _)| tokens);
    if !diagnostics.is_empty()
        || tokens.clone().count() != formatted_tokens.clone().count()
        || !tokens.zip(formatted_tokens).all(|(a, b)| same_token(a, b))
    {
        return Err(FormatError::ChangedCode);
    }
    Ok(formatted)
}

fn check_formattable(parser: &VHDLParser, source: &Source) -> Result<(), FormatError> {
    let contents = source.contents();
    let mut tokenizer = Tokenizer::new(&parser.symbols, source, ContentReader::new(&contents));
    while let Some(token) = tokenizer.pop().map_err(|_| FormatError::SyntaxErrors)? {
        if token.kind == Kind::GraveAccent {
            return Err(FormatError::ToolDirective);
        }
    }
    // The tokenizer drops ignored regions, so they can only be found in the text.
    let has_ignored_region = (0..contents.num_lines())
        .filter_map(|line| contents.get_line(line))
        .any(|line| line.contains("vhdl_ls off"));
    if has_ignored_region {
        return Err(FormatError::IgnoredRegion);
    }
    Ok(())
}

fn same_token(a: &Token, b: &Token) -> bool {
    a.equal_format(b) && same_comments(a.comments.as_deref(), b.comments.as_deref())
}

fn same_comments(a: Option<&TokenComments>, b: Option<&TokenComments>) -> bool {
    let comments = |comments: Option<&TokenComments>| {
        comments
            .into_iter()
            .flat_map(|comments| comments.leading.iter().chain(&comments.trailing))
            .map(|comment| (comment.value.trim().to_owned(), comment.multi_line))
            .collect::<Vec<_>>()
    };
    comments(a) == comments(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VHDLStandard;
    use std::path::Path;

    fn format(code: &str) -> Result<String, FormatError> {
        format_source(
            &VHDLParser::new(VHDLStandard::default()),
            &Source::inline(Path::new("test.vhd"), code),
        )
    }

    #[test]
    fn formats_valid_code() {
        assert_eq!(
            format("entity   foo is\nend   entity;\n").as_deref(),
            Ok("entity foo is\nend entity;\n")
        );
    }

    #[test]
    fn keeps_comments() {
        let code = "-- header\nentity foo is -- trailing\nend entity;\n";
        assert_eq!(format(code).as_deref(), Ok(code));
    }

    #[test]
    fn rejects_syntax_errors() {
        assert_eq!(format("entity foo is"), Err(FormatError::SyntaxErrors));
    }

    #[test]
    fn rejects_ignored_regions() {
        assert_eq!(
            format("-- vhdl_ls off\nfoo bar\n-- vhdl_ls on\nentity foo is\nend entity;"),
            Err(FormatError::IgnoredRegion)
        );
    }

    #[test]
    fn rejects_tool_directives() {
        assert_eq!(
            format("`protect begin\nentity foo is\nend entity;"),
            Err(FormatError::ToolDirective)
        );
    }
}

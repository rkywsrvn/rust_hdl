// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.

use super::tokenizer::Kind::*;
use super::tokenizer::*;
use crate::data::error_codes::ErrorCode;
use crate::data::DiagnosticHandler;
use crate::{Diagnostic, SrcPos, VHDLStandard};

/// LRM 24.2 Conditional analysis
///
/// Tracks the `` `if `` / `` `elsif `` / `` `else `` / `` `end if `` directives of a file.
/// Tokens are only analyzed while all enclosing conditions are true.
#[derive(Default)]
pub(crate) struct ConditionalAnalysis {
    branches: Vec<Branch>,
}

struct Branch {
    pos: SrcPos,
    /// True if the enclosing branch is active
    parent_active: bool,
    /// True if a previous condition of this `if` was true
    taken: bool,
    active: bool,
}

impl ConditionalAnalysis {
    pub fn is_active(&self) -> bool {
        self.branches.last().is_none_or(|branch| branch.active)
    }

    /// Handles a tool directive that starts with `` ` ``.
    pub fn directive(
        &mut self,
        grave_accent: &Token,
        tokenizer: &mut Tokenizer<'_>,
        standard: VHDLStandard,
        diagnostics: &mut dyn DiagnosticHandler,
    ) {
        let start_pos = grave_accent.pos.clone();
        let token = match tokenizer.pop() {
            Ok(Some(token)) => token,
            Ok(None) => {
                diagnostics.push(Diagnostic::syntax_error(start_pos, "Expecting identifier"));
                return;
            }
            Err(err) => {
                if self.is_active() {
                    diagnostics.push(err);
                }
                return;
            }
        };
        let pos = start_pos.combine(&token.pos);
        match token.kind {
            If => {
                let parent_active = self.is_active();
                let condition = self.condition(tokenizer, &pos, standard, diagnostics);
                let active = parent_active && condition;
                self.branches.push(Branch {
                    pos,
                    parent_active,
                    taken: active,
                    active,
                });
            }
            Elsif => {
                let condition = self.condition(tokenizer, &pos, standard, diagnostics);
                match self.branches.last_mut() {
                    Some(branch) => {
                        branch.active = branch.parent_active && !branch.taken && condition;
                        branch.taken |= branch.active;
                    }
                    None => diagnostics.push(Diagnostic::syntax_error(pos, "`elsif without `if")),
                }
            }
            Else => match self.branches.last_mut() {
                Some(branch) => {
                    branch.active = branch.parent_active && !branch.taken;
                    branch.taken = true;
                }
                None => diagnostics.push(Diagnostic::syntax_error(pos, "`else without `if")),
            },
            End => {
                if !matches!(tokenizer.pop(), Ok(Some(Token { kind: If, .. }))) {
                    diagnostics.push(Diagnostic::syntax_error(pos.clone(), "Expected `end if"));
                }
                if self.branches.pop().is_none() {
                    diagnostics.push(Diagnostic::syntax_error(pos, "`end if without `if"));
                }
            }
            Identifier => {
                let message = tokenizer.text_until_newline();
                if !self.is_active() {
                    return;
                }
                let code = match &token.value {
                    Value::Identifier(symbol) if is_named(symbol, "warning") => {
                        Some(ErrorCode::WarningDirective)
                    }
                    Value::Identifier(symbol) if is_named(symbol, "error") => {
                        Some(ErrorCode::ErrorDirective)
                    }
                    _ => None,
                };
                match (code, message) {
                    (Some(code), Ok(message)) => {
                        let text = match &message.value {
                            Value::Text(text) => text.to_string(),
                            _ => String::new(),
                        };
                        diagnostics.push(Diagnostic::new(
                            start_pos.combine(&message.pos),
                            text.trim().trim_matches('"'),
                            code,
                        ));
                    }
                    (_, Err(err)) => diagnostics.push(err),
                    (None, Ok(_)) => {}
                }
            }
            _ => {
                let _ = tokenizer.text_until_newline();
                if self.is_active() {
                    diagnostics.push(Diagnostic::syntax_error(token, "Expecting identifier"));
                }
            }
        }
    }

    /// Reports `` `if `` directives that are not closed at the end of the file.
    pub fn finish(self, diagnostics: &mut dyn DiagnosticHandler) {
        for branch in self.branches {
            diagnostics.push(Diagnostic::syntax_error(branch.pos, "Missing `end if"));
        }
    }

    /// Reads a condition until `then` and evaluates it.
    fn condition(
        &self,
        tokenizer: &mut Tokenizer<'_>,
        pos: &SrcPos,
        standard: VHDLStandard,
        diagnostics: &mut dyn DiagnosticHandler,
    ) -> bool {
        let mut tokens = Vec::new();
        loop {
            match tokenizer.pop() {
                Ok(Some(token)) if token.kind == Then => break,
                Ok(Some(token)) => tokens.push(token),
                Ok(None) => {
                    diagnostics.push(Diagnostic::syntax_error(pos.clone(), "Expected then"));
                    return false;
                }
                Err(err) => {
                    diagnostics.push(err);
                    return false;
                }
            }
        }
        let mut parser = ConditionParser {
            tokens: &tokens,
            idx: 0,
            standard,
        };
        match parser.expression() {
            Ok(value) if parser.idx == tokens.len() => value,
            Ok(_) => {
                diagnostics.push(Diagnostic::syntax_error(
                    &tokens[parser.idx],
                    "Expected then",
                ));
                false
            }
            Err(err) => {
                diagnostics.push(err.unwrap_or_else(|| {
                    Diagnostic::syntax_error(pos.clone(), "Incomplete condition")
                }));
                false
            }
        }
    }
}

fn is_named(symbol: &crate::data::Symbol, name: &str) -> bool {
    symbol.name_utf8().eq_ignore_ascii_case(name)
}

/// LRM 24.2: The value of a conditional analysis identifier.
fn identifier_value(name: &str, standard: VHDLStandard) -> Option<String> {
    Some(match name.to_ascii_uppercase().as_str() {
        "VHDL_VERSION" => match standard {
            VHDLStandard::VHDL1993 => "1993",
            VHDLStandard::VHDL2008 => "2008",
            VHDLStandard::VHDL2019 => "2019",
        }
        .to_owned(),
        "TOOL_TYPE" => "SIMULATION".to_owned(),
        "TOOL_VENDOR" => "VHDL-LS".to_owned(),
        "TOOL_NAME" => "vhdl_ls".to_owned(),
        "TOOL_EDITION" => String::new(),
        "TOOL_VERSION" => env!("CARGO_PKG_VERSION").to_owned(),
        _ => return None,
    })
}

/// An error, or `None` if the condition ended early
type ConditionResult<T> = Result<T, Option<Diagnostic>>;

struct ConditionParser<'t> {
    tokens: &'t [Token],
    idx: usize,
    standard: VHDLStandard,
}

impl<'t> ConditionParser<'t> {
    fn peek(&self) -> Option<&'t Token> {
        self.tokens.get(self.idx)
    }

    fn next(&mut self) -> ConditionResult<&'t Token> {
        let token = self.tokens.get(self.idx).ok_or(None)?;
        self.idx += 1;
        Ok(token)
    }

    /// conditional_analysis_expression ::= conditional_and_expression { or conditional_and_expression }
    /// The other logical operators are evaluated from left to right.
    fn expression(&mut self) -> ConditionResult<bool> {
        let mut value = self.relation()?;
        while let Some(kind) = self.peek().map(|token| token.kind) {
            let combine: fn(bool, bool) -> bool = match kind {
                And => |a, b| a && b,
                Or => |a, b| a || b,
                Xor => |a, b| a != b,
                Xnor => |a, b| a == b,
                Nand => |a, b| !(a && b),
                Nor => |a, b| !(a || b),
                _ => break,
            };
            self.idx += 1;
            value = combine(value, self.relation()?);
        }
        Ok(value)
    }

    fn relation(&mut self) -> ConditionResult<bool> {
        let token = self.next()?;
        match token.kind {
            Not => Ok(!self.relation()?),
            LeftPar => {
                let value = self.expression()?;
                match self.next()? {
                    Token { kind: RightPar, .. } => Ok(value),
                    token => Err(Some(Diagnostic::syntax_error(token, "Expected ')'"))),
                }
            }
            Identifier => {
                let Value::Identifier(symbol) = &token.value else {
                    unreachable!("identifier token without identifier value");
                };
                let Some(value) = identifier_value(&symbol.name_utf8(), self.standard) else {
                    return Err(Some(Diagnostic::syntax_error(
                        token,
                        format!("Unknown conditional analysis identifier '{symbol}'"),
                    )));
                };
                let operator = self.next()?;
                let literal = self.next()?;
                let Value::String(literal) = &literal.value else {
                    return Err(Some(Diagnostic::syntax_error(literal, "Expected string")));
                };
                let literal = literal.to_string();
                let ordering = value.as_str().cmp(literal.as_str());
                match operator.kind {
                    EQ => Ok(ordering.is_eq()),
                    NE => Ok(ordering.is_ne()),
                    LT => Ok(ordering.is_lt()),
                    LTE => Ok(ordering.is_le()),
                    GT => Ok(ordering.is_gt()),
                    GTE => Ok(ordering.is_ge()),
                    _ => Err(Some(Diagnostic::syntax_error(
                        operator,
                        "Expected relational operator",
                    ))),
                }
            }
            _ => Err(Some(Diagnostic::syntax_error(
                token,
                "Expected conditional analysis identifier",
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{AnyDesignUnit, HasIdent};
    use crate::data::error_codes::ErrorCode;
    use crate::syntax::VHDLParser;
    use crate::{Diagnostic, Source, VHDLStandard};
    use std::path::Path;

    /// Returns the names of the parsed primary units and the diagnostics.
    fn parse(code: &str, standard: VHDLStandard) -> (Vec<String>, Vec<Diagnostic>) {
        let mut diagnostics = Vec::new();
        let file = VHDLParser::new(standard).parse_design_source(
            &Source::inline(Path::new("file.vhd"), code),
            &mut diagnostics,
        );
        let names = file
            .design_units
            .iter()
            .filter_map(|(_, unit)| match unit {
                AnyDesignUnit::Primary(unit) => Some(unit.ident().item.to_string()),
                AnyDesignUnit::Secondary(_) => None,
            })
            .collect();
        (names, diagnostics)
    }

    const VERSIONS: &str = "
`if VHDL_VERSION >= \"2019\" then
entity v2019 is end entity;
`elsif VHDL_VERSION >= \"2008\" then
entity v2008 is end entity;
`else
entity v1993 is end entity;
`end if
entity common is end entity;
";

    #[test]
    fn analyzes_the_branch_that_matches() {
        for (standard, expected) in [
            (VHDLStandard::VHDL2019, "v2019"),
            (VHDLStandard::VHDL2008, "v2008"),
            (VHDLStandard::VHDL1993, "v1993"),
        ] {
            let (names, diagnostics) = parse(VERSIONS, standard);
            assert_eq!(names, vec![expected, "common"]);
            assert_eq!(diagnostics, vec![]);
        }
    }

    #[test]
    fn nested_conditions() {
        let (names, diagnostics) = parse(
            "
`if TOOL_NAME = \"other\" then
`if VHDL_VERSION = \"2008\" then
entity a is end entity;
`end if
`else
`if not (TOOL_TYPE = \"SYNTHESIS\" or TOOL_TYPE = \"FORMAL\") then
entity b is end entity;
`end if
`end if
",
            VHDLStandard::VHDL2008,
        );
        assert_eq!(names, vec!["b"]);
        assert_eq!(diagnostics, vec![]);
    }

    #[test]
    fn inactive_branches_are_not_parsed() {
        let (names, diagnostics) = parse(
            "
`if VHDL_VERSION = \"1993\" then
entity a is is is;
`end if
entity b is end entity;
",
            VHDLStandard::VHDL2008,
        );
        assert_eq!(names, vec!["b"]);
        assert_eq!(diagnostics, vec![]);
    }

    #[test]
    fn warning_and_error_directives_in_active_branches() {
        let (_, diagnostics) = parse(
            "
`if VHDL_VERSION < \"2008\" then
`error \"needs VHDL-2008\"
`else
`warning \"not tested\"
`end if
",
            VHDLStandard::VHDL2008,
        );
        assert_eq!(
            diagnostics
                .iter()
                .map(|diag| (diag.message.as_str(), diag.code))
                .collect::<Vec<_>>(),
            vec![("not tested", ErrorCode::WarningDirective)]
        );
    }

    #[test]
    fn reports_malformed_directives() {
        let (_, diagnostics) = parse(
            "
`if UNKNOWN = \"x\" then
`end if
`if VHDL_VERSION = \"2008\" then
",
            VHDLStandard::VHDL2008,
        );
        assert_eq!(
            diagnostics
                .iter()
                .map(|diag| diag.message.as_str())
                .collect::<Vec<_>>(),
            vec![
                "Unknown conditional analysis identifier 'UNKNOWN'",
                "Missing `end if"
            ]
        );
    }
}

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.

use super::common::ParseResult;
use super::expression::parse_expression;
use super::names::parse_name;
use super::recover::expect_semicolon;
use super::tokens::Kind::*;
use crate::ast::{DisconnectionSpecification, GuardedSignalList};
use vhdl_lang::syntax::parser::ParsingContext;

/// LRM 7.4 Disconnection specification
pub fn parse_disconnection_specification(
    ctx: &mut ParsingContext<'_>,
) -> ParseResult<DisconnectionSpecification> {
    ctx.stream.expect_kind(Disconnect)?;
    let signals = if ctx.stream.skip_if_kind(Others) {
        GuardedSignalList::Others
    } else if ctx.stream.skip_if_kind(All) {
        GuardedSignalList::All
    } else {
        let mut names = vec![parse_name(ctx)?];
        while ctx.stream.skip_if_kind(Comma) {
            names.push(parse_name(ctx)?);
        }
        GuardedSignalList::Names(names)
    };
    ctx.stream.expect_kind(Colon)?;
    let type_mark = parse_name(ctx)?;
    ctx.stream.expect_kind(After)?;
    let time = parse_expression(ctx)?;
    expect_semicolon(ctx);
    Ok(DisconnectionSpecification {
        signals,
        type_mark,
        time,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::test::Code;

    #[test]
    fn parses_signal_names() {
        let code = Code::new("disconnect a, b : bit after 10 ns;");
        assert_eq!(
            code.with_stream_no_diagnostics(parse_disconnection_specification),
            DisconnectionSpecification {
                signals: GuardedSignalList::Names(vec![code.s1("a").name(), code.s1("b").name()]),
                type_mark: code.s1("bit").name(),
                time: code.s1("10 ns").expr(),
            }
        );
    }

    #[test]
    fn parses_others_and_all() {
        let code = Code::new("disconnect others : bit after 1 ns;");
        assert_eq!(
            code.with_stream_no_diagnostics(parse_disconnection_specification)
                .signals,
            GuardedSignalList::Others
        );
        let code = Code::new("disconnect all : bit after 1 ns;");
        assert_eq!(
            code.with_stream_no_diagnostics(parse_disconnection_specification)
                .signals,
            GuardedSignalList::All
        );
    }
}

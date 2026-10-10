// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.

//! PSL (IEEE 1850) is not analyzed. PSL items are skipped so that they do not
//! produce syntax errors.

use super::common::ParseResult;
use super::tokens::Kind::{self, *};
use crate::syntax::parser::ParsingContext;

/// Returns true if the next tokens start a PSL directive or declaration.
/// A label in front of a directive is part of the item.
pub(crate) fn is_psl_item(ctx: &ParsingContext<'_>) -> bool {
    let first = ctx.stream.peek_kind();
    let item = if first == Some(Identifier) && ctx.stream.nth_kind_is(1, Colon) {
        ctx.stream.nth_kind(2)
    } else {
        first
    };
    match item {
        Some(
            Assume | AssumeGuarantee | Cover | Restrict | RestrictGuarantee | Fairness | Strong
            | Property | Sequence,
        ) => true,
        Some(Default) => ctx.stream.nth_kind_is(1, Identifier),
        _ => false,
    }
}

/// Skips a PSL directive or declaration up to and including the final `;`.
pub(crate) fn skip_psl_item(ctx: &mut ParsingContext<'_>) -> ParseResult<()> {
    skip_until_depth_zero(ctx, SemiColon)
}

/// Skips a PSL verification unit (`vunit`, `vmode` or `vprop`) up to and including its `}`.
pub(crate) fn skip_verification_unit(ctx: &mut ParsingContext<'_>) -> ParseResult<()> {
    while ctx.stream.peek_expect()?.kind != LeftCurly {
        ctx.stream.skip();
    }
    skip_until_depth_zero(ctx, RightCurly)
}

fn skip_until_depth_zero(ctx: &mut ParsingContext<'_>, end: Kind) -> ParseResult<()> {
    let mut depth = 0usize;
    loop {
        let kind = ctx.stream.peek_expect()?.kind;
        ctx.stream.skip();
        match kind {
            LeftPar | LeftSquare | LeftCurly => depth += 1,
            RightPar | RightSquare | RightCurly => depth = depth.saturating_sub(1),
            _ => {}
        }
        if kind == end && depth == 0 {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{AnyDesignUnit, AnySecondaryUnit, ConcurrentStatement};
    use crate::syntax::VHDLParser;
    use crate::{Source, VHDLStandard};
    use std::path::Path;

    #[test]
    fn skips_psl_items() {
        let code = "
vunit checks (ent) {
    assert always a -> next b;
}

entity ent is
end entity;

architecture arch of ent is
    signal clk, a, b : bit;
    default clock is rising_edge(clk);
    sequence s is {a; b};
    property p is always (a -> next[2] (b));
begin
    assert always (a -> next b);
    lbl: cover {a; b[*2]};
    assume always not (a and b);
    restrict {a; b};
    fairness a;
    vhdl: assert a = '1' report \"vhdl\" severity note;
end architecture;
";
        let mut diagnostics = Vec::new();
        let file = VHDLParser::new(VHDLStandard::VHDL2008).parse_design_source(
            &Source::inline(Path::new("file.vhd"), code),
            &mut diagnostics,
        );
        assert_eq!(diagnostics, vec![]);
        let AnyDesignUnit::Secondary(AnySecondaryUnit::Architecture(arch)) =
            &file.design_units[1].1
        else {
            panic!("expected an architecture");
        };
        assert_eq!(arch.decl.len(), 1);
        assert_eq!(arch.statements.len(), 1);
        assert!(matches!(
            arch.statements[0].statement.item,
            ConcurrentStatement::Assert(_)
        ));
    }
}

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.

use super::attributes::parse_entity_class;
use super::common::ParseResult;
use super::names::{parse_name, parse_selected_name};
use super::recover::expect_semicolon;
use super::tokens::Kind::*;
use crate::ast::{
    Declaration, GroupDeclaration, GroupTemplateDeclaration, GroupTemplateEntry, WithDecl,
};
use vhdl_lang::syntax::parser::ParsingContext;

/// LRM 6.9 Group template declarations and LRM 6.10 Group declarations
pub fn parse_group_declaration(ctx: &mut ParsingContext<'_>) -> ParseResult<Declaration> {
    ctx.stream.expect_kind(Group)?;
    let ident = WithDecl::new(ctx.stream.expect_ident()?);
    expect_token!(ctx.stream, token,
        Is => {
            ctx.stream.expect_kind(LeftPar)?;
            let mut entries = Vec::new();
            loop {
                let class = parse_entity_class(ctx)?;
                let is_open = ctx.stream.skip_if_kind(BOX);
                entries.push(GroupTemplateEntry { class, is_open });
                if !ctx.stream.skip_if_kind(Comma) {
                    break;
                }
            }
            ctx.stream.expect_kind(RightPar)?;
            expect_semicolon(ctx);
            Ok(Declaration::GroupTemplate(GroupTemplateDeclaration { ident, entries }))
        },
        Colon => {
            let template = parse_selected_name(ctx)?;
            ctx.stream.expect_kind(LeftPar)?;
            let mut constituents = vec![parse_name(ctx)?];
            while ctx.stream.skip_if_kind(Comma) {
                constituents.push(parse_name(ctx)?);
            }
            ctx.stream.expect_kind(RightPar)?;
            expect_semicolon(ctx);
            Ok(Declaration::Group(GroupDeclaration { ident, template, constituents }))
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::EntityClass;
    use crate::syntax::test::Code;

    #[test]
    fn parses_group_template() {
        let code = Code::new("group pins is (signal, signal <>);");
        assert_eq!(
            code.with_stream_no_diagnostics(parse_group_declaration),
            Declaration::GroupTemplate(GroupTemplateDeclaration {
                ident: WithDecl::new(code.s1("pins").ident()),
                entries: vec![
                    GroupTemplateEntry {
                        class: EntityClass::Signal,
                        is_open: false
                    },
                    GroupTemplateEntry {
                        class: EntityClass::Signal,
                        is_open: true
                    },
                ],
            })
        );
    }

    #[test]
    fn parses_group() {
        let code = Code::new("group clocks : pins (clk, rst);");
        assert_eq!(
            code.with_stream_no_diagnostics(parse_group_declaration),
            Declaration::Group(GroupDeclaration {
                ident: WithDecl::new(code.s1("clocks").ident()),
                template: code.s1("pins").name(),
                constituents: vec![code.s1("clk").name(), code.s1("rst").name()],
            })
        );
    }
}

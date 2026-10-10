// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::analysis::DesignRoot;
use crate::ast::search::{DeclarationItem, FoundDeclaration, NotFinished, SearchState, Searcher};
use crate::ast::Designator;
use crate::completion::tokenizer::tokenize_input;
use crate::named_entity::{DesignEnt, NamedEntities, OverloadedEnt, Region, Visibility};
use crate::syntax::{Kind, Value};
use crate::{Design, HasTokenSpan, Position, Source, TokenAccess};

/// The subprograms that a call at the cursor may refer to.
#[derive(Debug)]
pub struct SignatureHelp<'a> {
    pub signatures: Vec<OverloadedEnt<'a>>,
    pub active_signature: usize,
    pub argument: Argument,
}

/// The argument of a call that the cursor is in.
#[derive(Debug, PartialEq, Eq)]
pub enum Argument {
    Positional(usize),
    Named(Designator),
}

impl SignatureHelp<'_> {
    /// The index of the formal that the argument at the cursor is associated with.
    pub fn active_parameter(&self, signature: OverloadedEnt<'_>) -> Option<usize> {
        match &self.argument {
            Argument::Positional(index) => (*index < signature.formals().len()).then_some(*index),
            Argument::Named(designator) => signature
                .formals()
                .iter()
                .position(|formal| formal.designator() == designator),
        }
    }
}

/// Lists the subprograms that the call at the cursor may refer to.
pub fn signature_help<'a>(
    root: &'a DesignRoot,
    source: &Source,
    cursor: Position,
) -> Option<SignatureHelp<'a>> {
    let tokens = tokenize_input(root.symbols(), source, cursor);
    let mut depth = 0;
    let mut commas = 0;
    let mut argument_start = tokens.len();
    let mut open_par = None;
    for (i, token) in tokens.iter().enumerate().rev() {
        match token.kind {
            Kind::RightPar => depth += 1,
            Kind::LeftPar if depth == 0 => {
                open_par = Some(i);
                break;
            }
            Kind::LeftPar => depth -= 1,
            Kind::Comma if depth == 0 => {
                if commas == 0 {
                    argument_start = i + 1;
                }
                commas += 1;
            }
            Kind::SemiColon => return None,
            _ => {}
        }
    }
    let open_par = open_par?;
    if commas == 0 {
        argument_start = open_par + 1;
    }
    let name = tokens.get(open_par.checked_sub(1)?)?;
    let Value::Identifier(symbol) = &name.value else {
        return None;
    };
    let designator = Designator::Identifier(symbol.clone());

    let mut searcher = VisibleSubprograms {
        root,
        cursor,
        designator,
        found: Vec::new(),
    };
    let _ = root.search_source(source, &mut searcher);
    let signatures = searcher.found;
    if signatures.is_empty() {
        return None;
    }

    let argument = match &tokens[argument_start..] {
        [formal, arrow, ..]
            if formal.kind == Kind::Identifier && arrow.kind == Kind::RightArrow =>
        {
            match &formal.value {
                Value::Identifier(symbol) => {
                    Argument::Named(Designator::Identifier(symbol.clone()))
                }
                _ => Argument::Positional(commas),
            }
        }
        _ => Argument::Positional(commas),
    };
    let mut help = SignatureHelp {
        signatures,
        active_signature: 0,
        argument,
    };
    let resolved = root
        .item_at_cursor(source, name.pos.start())
        .and_then(|(_, ent)| help.signatures.iter().position(|s| s.inner() == ent));
    help.active_signature = resolved
        .or_else(|| {
            help.signatures
                .iter()
                .position(|signature| help.active_parameter(*signature).is_some())
        })
        .unwrap_or(0);
    Some(help)
}

/// Collects the subprograms with a given designator that are visible at the cursor.
struct VisibleSubprograms<'a> {
    root: &'a DesignRoot,
    cursor: Position,
    designator: Designator,
    found: Vec<OverloadedEnt<'a>>,
}

impl<'a> VisibleSubprograms<'a> {
    fn add(&mut self, entities: Option<&NamedEntities<'a>>) {
        let entities = match entities {
            Some(NamedEntities::Overloaded(overloaded)) => overloaded.entities().collect(),
            Some(NamedEntities::Single(ent)) => OverloadedEnt::from_any(ent).into_iter().collect(),
            None => Vec::new(),
        };
        for ent in entities {
            if !self.found.contains(&ent) {
                self.found.push(ent);
            }
        }
    }

    fn add_from_region(&mut self, region: &'a Region<'a>) {
        let designator = self.designator.clone();
        self.add(region.lookup_immediate(&designator));
    }

    fn add_from_visibility(&mut self, visibility: &'a Visibility<'a>) {
        for ent in visibility.visible() {
            if ent.designator() == &self.designator {
                if let Some(ent) = OverloadedEnt::from_any(ent) {
                    if !self.found.contains(&ent) {
                        self.found.push(ent);
                    }
                }
            }
        }
        for visible in visibility.all_in_region() {
            self.add_from_region(visible.region());
        }
    }

    fn add_from_design(&mut self, design: &'a Design<'a>) {
        match design {
            Design::Entity(visibility, region)
            | Design::UninstPackage(visibility, region)
            | Design::Architecture(visibility, region, _)
            | Design::Package(visibility, region)
            | Design::PackageBody(visibility, region) => {
                self.add_from_region(region);
                self.add_from_visibility(visibility);
            }
            Design::PackageInstance(region)
            | Design::InterfacePackageInstance(_, region)
            | Design::Context(region) => self.add_from_region(region),
            Design::Configuration => {}
        }
    }
}

impl Searcher for VisibleSubprograms<'_> {
    fn search_decl(&mut self, ctx: &dyn TokenAccess, decl: FoundDeclaration<'_>) -> SearchState {
        let id = match &decl.ast {
            DeclarationItem::Entity(unit) if unit.get_pos(ctx).contains(self.cursor) => {
                unit.ident.decl.get()
            }
            DeclarationItem::Architecture(unit) if unit.get_pos(ctx).contains(self.cursor) => {
                unit.ident.decl.get()
            }
            DeclarationItem::Package(unit) if unit.get_pos(ctx).contains(self.cursor) => {
                unit.ident.decl.get()
            }
            DeclarationItem::PackageBody(unit) if unit.get_pos(ctx).contains(self.cursor) => {
                unit.ident.decl.get()
            }
            DeclarationItem::Subprogram(subprogram)
                if subprogram.get_pos(ctx).contains(self.cursor) =>
            {
                for id in subprogram
                    .declarations
                    .iter()
                    .flat_map(|decl| decl.item.declarations())
                {
                    let ent = self.root.get_ent(id);
                    if ent.designator() == &self.designator {
                        if let Some(ent) = OverloadedEnt::from_any(ent) {
                            if !self.found.contains(&ent) {
                                self.found.push(ent);
                            }
                        }
                    }
                }
                None
            }
            _ => None,
        };
        if let Some(design) = id.and_then(|id| DesignEnt::from_any(self.root.get_ent(id))) {
            self.add_from_design(design.kind());
        }
        NotFinished
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::tests::LibraryBuilder;
    use crate::syntax::test::Code;

    fn help_at<'a>(
        root: &'a DesignRoot,
        code: &Code,
        cursor_after: &str,
    ) -> Option<SignatureHelp<'a>> {
        signature_help(root, code.source(), code.s1(cursor_after).end())
    }

    fn describe(help: &SignatureHelp<'_>) -> Vec<String> {
        help.signatures
            .iter()
            .map(|signature| signature.describe())
            .collect()
    }

    const PKG: &str = "\
package pkg is
    function add(a : integer; b : integer) return integer;
    function add(a : bit; b : bit; c : bit) return bit;
    procedure proc(x : integer);
end package;
";

    #[test]
    fn lists_overloads_and_counts_arguments() {
        let mut builder = LibraryBuilder::new();
        builder.code("libname", PKG);
        let code = builder.code(
            "libname",
            "\
use work.pkg.all;

entity ent is
end entity;

architecture arch of ent is
    constant c : integer := add(1,
begin
end architecture;",
        );
        let (root, _) = builder.get_analyzed_root();
        let help = help_at(&root, &code, "add(1,").unwrap();
        assert_eq!(describe(&help).len(), 2);
        assert_eq!(help.argument, Argument::Positional(1));
        assert!(help.signatures[help.active_signature].formals().len() >= 2);
    }

    #[test]
    fn selects_signature_that_has_enough_parameters() {
        let mut builder = LibraryBuilder::new();
        builder.code("libname", PKG);
        let code = builder.code(
            "libname",
            "\
use work.pkg.all;

entity ent is
end entity;

architecture arch of ent is
    constant c : bit := add('0', '1',
begin
end architecture;",
        );
        let (root, _) = builder.get_analyzed_root();
        let help = help_at(&root, &code, "'1',").unwrap();
        assert_eq!(help.argument, Argument::Positional(2));
        assert_eq!(help.signatures[help.active_signature].formals().len(), 3);
    }

    #[test]
    fn named_association() {
        let mut builder = LibraryBuilder::new();
        builder.code("libname", PKG);
        let code = builder.code(
            "libname",
            "\
use work.pkg.all;

entity ent is
end entity;

architecture arch of ent is
begin
    process
    begin
        proc(x =>
    end process;
end architecture;",
        );
        let (root, _) = builder.get_analyzed_root();
        let help = help_at(&root, &code, "proc(x =>").unwrap();
        assert_eq!(describe(&help).len(), 1);
        let signature = help.signatures[0];
        assert_eq!(help.active_parameter(signature), Some(0));
    }

    #[test]
    fn nested_calls_use_the_innermost_call() {
        let mut builder = LibraryBuilder::new();
        builder.code("libname", PKG);
        let code = builder.code(
            "libname",
            "\
use work.pkg.all;

entity ent is
end entity;

architecture arch of ent is
    constant c : integer := add(add(1, 2),
begin
end architecture;",
        );
        let (root, _) = builder.get_analyzed_root();
        let help = help_at(&root, &code, "add(1, 2),").unwrap();
        assert_eq!(help.argument, Argument::Positional(1));
    }

    #[test]
    fn no_help_outside_of_calls() {
        let mut builder = LibraryBuilder::new();
        let code = builder.code(
            "libname",
            "\
entity ent is
end entity;",
        );
        let (root, _) = builder.get_analyzed_root();
        assert!(help_at(&root, &code, "entity ent").is_none());
    }
}

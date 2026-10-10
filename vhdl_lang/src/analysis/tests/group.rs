// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.

use super::*;
use crate::data::error_codes::ErrorCode;

#[test]
fn group_with_attribute() {
    let mut builder = LibraryBuilder::new();
    let code = builder.in_declarative_region(
        "
signal clk, rst, en : bit;
group pins is (signal, signal <>);
group control : pins (clk, rst, en);
attribute note : string;
attribute note of control : group is \"reset domain\";
",
    );
    let (root, diagnostics) = builder.get_analyzed_root();
    check_no_diagnostics(&diagnostics);
    assert_eq!(
        root.search_reference_pos(code.source(), code.s1("pins (").start()),
        Some(code.s1("pins").pos())
    );
    assert_eq!(
        root.search_reference_pos(code.source(), code.s1("control :").start()),
        Some(code.s1("control").pos())
    );
}

#[test]
fn group_errors() {
    let mut builder = LibraryBuilder::new();
    let code = builder.in_declarative_region(
        "
signal clk : bit;
constant c : bit := '0';
group pair is (signal, signal);
group open_first is (signal <>, constant);
group too_few : pair (clk);
group wrong_class : pair (clk, c);
group not_a_template : clk (clk);
",
    );
    check_diagnostics(
        builder.analyze(),
        vec![
            Diagnostic::new(
                code.s1("group open_first is (signal <>, constant);"),
                "Only the last entry may have '<>', not signal",
                ErrorCode::SyntaxError,
            ),
            Diagnostic::new(
                code.s1("too_few : pair").s1("pair"),
                "Group template has 2 entries, but the group has 1 constituents",
                ErrorCode::Unassociated,
            ),
            Diagnostic::new(
                code.s1("clk, c)").s1("c)").s1("c"),
                "constant 'c' is not of class signal",
                ErrorCode::MismatchedEntityClass,
            ),
            Diagnostic::new(
                code.s1("not_a_template : clk").s1("clk"),
                "signal 'clk' is not a group template",
                ErrorCode::MismatchedKinds,
            ),
        ],
    );
}

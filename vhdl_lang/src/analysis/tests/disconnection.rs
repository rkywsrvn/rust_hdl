// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.

use super::*;
use crate::data::error_codes::ErrorCode;

#[test]
fn disconnection_specification() {
    let mut builder = LibraryBuilder::new();
    let code = builder.in_declarative_region(
        "
signal s1, s2 : bit;
disconnect s1, s2 : bit after 10 ns;
disconnect others : bit after 1 ns;
",
    );
    let (root, diagnostics) = builder.get_analyzed_root();
    check_no_diagnostics(&diagnostics);
    assert_eq!(
        root.search_reference_pos(code.source(), code.s("s2", 2).start()),
        Some(code.s1("s2").pos())
    );
}

#[test]
fn disconnection_specification_errors() {
    let mut builder = LibraryBuilder::new();
    let code = builder.in_declarative_region(
        "
signal s : bit;
constant c : bit := '0';
disconnect s : boolean after 10 ns;
disconnect c : bit after 10 ns;
disconnect s : bit after 10;
",
    );
    check_diagnostics(
        builder.analyze(),
        vec![
            Diagnostic::new(
                code.s1("s : boolean").s1("s"),
                "signal 's' of type 'BIT' does not match type 'BOOLEAN'",
                ErrorCode::TypeMismatch,
            ),
            Diagnostic::new(
                code.s1("c : bit after").s1("c"),
                "constant 'c' is not a signal",
                ErrorCode::MismatchedKinds,
            ),
            Diagnostic::new(
                code.s1("after 10;").s1("10"),
                "integer literal does not match physical type 'TIME'",
                ErrorCode::TypeMismatch,
            ),
        ],
    );
}

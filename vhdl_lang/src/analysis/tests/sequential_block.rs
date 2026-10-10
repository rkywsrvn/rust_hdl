// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.

use super::*;
use crate::data::error_codes::ErrorCode;
use crate::VHDLStandard::VHDL2019;

#[test]
fn declarations_are_visible_inside_the_block_only() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.in_declarative_region(
        "
function f return natural is
begin
    for i in 0 to 3 loop
        blk: block is
            variable v : natural := i;
        begin
            exit when v = 2;
            return v;
        end block blk;
    end loop;
    return v;
end function;
",
    );
    check_diagnostics(
        builder.analyze(),
        vec![Diagnostic::new(
            code.s("v", 5),
            "No declaration of 'v'",
            ErrorCode::Unresolved,
        )],
    );
}

#[test]
fn signals_are_not_allowed_in_sequential_blocks() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.in_declarative_region(
        "
procedure p is
begin
    block is
        signal s : bit;
    begin
    end block;
end procedure;
",
    );
    check_diagnostics(
        builder.analyze(),
        vec![Diagnostic::new(
            code.s1("signal s : bit;"),
            "signal declaration not allowed here",
            ErrorCode::DeclarationNotAllowed,
        )],
    );
}

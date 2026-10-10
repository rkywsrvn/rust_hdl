// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.

use super::*;
use crate::data::error_codes::ErrorCode;
use crate::VHDLStandard::VHDL2019;

#[test]
fn generic_types_have_the_operations_of_their_class() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    builder.code(
        "libname",
        "
package pkg is
    generic (
        type int_t is range <>;
        type real_t is range <> . <>;
        type phys_t is units <>;
        type disc_t is (<>);
        type scalar_t is <>
    );

    function double(x : int_t) return int_t;
    function half(x : real_t) return real_t;
    function twice(x : phys_t) return phys_t;
    function is_first(x : disc_t) return boolean;
    function smaller(a, b : scalar_t) return boolean;
end package;

package body pkg is
    function double(x : int_t) return int_t is
    begin
        return 2 * x + 1;
    end function;

    function half(x : real_t) return real_t is
    begin
        return x / 2.0;
    end function;

    function twice(x : phys_t) return phys_t is
    begin
        return x * 2;
    end function;

    function is_first(x : disc_t) return boolean is
    begin
        for i in disc_t loop
            return x = i;
        end loop;
        return false;
    end function;

    function smaller(a, b : scalar_t) return boolean is
    begin
        return a < b;
    end function;
end package body;

package inst is new work.pkg
    generic map (
        int_t => integer,
        real_t => real,
        phys_t => time,
        disc_t => character,
        scalar_t => real
    );
",
    );
    check_no_diagnostics(&builder.analyze());
}

#[test]
fn private_generic_types_have_no_arithmetic() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.code(
        "libname",
        "
package pkg is
    generic (type t is private);
    function double(x : t) return t;
end package;

package body pkg is
    function double(x : t) return t is
    begin
        return x + x;
    end function;
end package body;
",
    );
    check_diagnostics(
        builder.analyze(),
        vec![Diagnostic::new(
            code.s1("+"),
            "Found no match for operator \"+\"",
            ErrorCode::Unresolved,
        )],
    );
}

#[test]
fn actual_must_belong_to_the_class_of_the_generic_type() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.code(
        "libname",
        "
package pkg is
    generic (type int_t is range <>; type disc_t is (<>));
end package;

package inst is new work.pkg generic map (int_t => real, disc_t => real);
",
    );
    check_diagnostics(
        builder.analyze(),
        vec![
            Diagnostic::new(
                code.s1("int_t => real").s1("real"),
                "real type 'REAL' is not an integer type",
                ErrorCode::TypeMismatch,
            ),
            Diagnostic::new(
                code.s1("disc_t => real").s1("real"),
                "real type 'REAL' is not a discrete type",
                ErrorCode::TypeMismatch,
            ),
        ],
    );
}

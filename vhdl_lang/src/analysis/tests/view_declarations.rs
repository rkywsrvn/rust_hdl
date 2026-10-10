use crate::analysis::tests::{check_diagnostics, check_no_diagnostics, LibraryBuilder};
use crate::data::ErrorCode;
use crate::Diagnostic;
use crate::VHDLStandard::VHDL2019;
use pretty_assertions::assert_eq;

#[test]
pub fn view_mode_declaration_must_have_declared_subtype() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.in_declarative_region(
        "\
view my_view of undeclared is
end view;
    ",
    );
    check_diagnostics(
        builder.analyze(),
        vec![Diagnostic::new(
            code.s1("undeclared"),
            "No declaration of 'undeclared'",
            ErrorCode::Unresolved,
        )],
    )
}

#[test]
pub fn view_mode_declaration_must_have_record_as_type() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.in_declarative_region(
        "\
type foo is (A, B);

view my_view of foo is
end view;
    ",
    );
    check_diagnostics(
        builder.analyze(),
        vec![Diagnostic::new(
            code.s("foo", 2),
            "The type of a view must be a record type, not type 'foo'",
            ErrorCode::TypeMismatch,
        )
        .related(code.s1("foo"), "type 'foo' declared here")],
    );

    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    builder.in_declarative_region(
        "\
type foo is record
  x: bit;
end record;

view my_view of foo is
  x : in;
end view;
    ",
    );
    check_no_diagnostics(&builder.analyze());
}

#[test]
pub fn element_in_view_that_is_not_in_record() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.in_declarative_region(
        "\
type foo is record
    bar: bit;
end record;

view my_view of foo is
    baz: in;
    bar: out;
end view;
    ",
    );
    let (_, diag) = builder.get_analyzed_root();
    check_diagnostics(
        diag,
        vec![Diagnostic::new(
            code.s1("baz"),
            "Not a part of record type 'foo'",
            ErrorCode::Unresolved,
        )],
    );
}

#[test]
pub fn view_reference_set_to_the_original_element() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.in_declarative_region(
        "\
type foo is record
    bar: bit;
end record;

view my_view of foo is
    bar: in;
end view;
    ",
    );
    let (root, diag) = builder.get_analyzed_root();
    check_no_diagnostics(&diag);
    let record_element = root
        .search_reference(code.source(), code.s1("bar: bit").s1("bar").start())
        .unwrap();
    let view_element = root
        .search_reference(code.source(), code.s1("bar: in").s1("bar").start())
        .unwrap();
    assert_eq!(record_element, view_element)
}

#[test]
pub fn diagnostic_when_elements_are_missing() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.in_declarative_region(
        "\
type foo is record
    bar: bit;
    baz: bit;
end record;

view my_view of foo is
    bar: in;
end view;
    ",
    );
    let diag = builder.analyze();
    check_diagnostics(
        diag,
        vec![Diagnostic::new(
            code.s1("my_view"),
            "Missing association of element 'baz'",
            ErrorCode::Unassociated,
        )],
    );

    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.in_declarative_region(
        "\
type foo is record
    bar: bit;
    baz: bit;
    foobar: bit;
end record;

view my_view of foo is
    bar: in;
end view;
    ",
    );
    let diag = builder.analyze();
    check_diagnostics(
        diag,
        vec![Diagnostic::new(
            code.s1("my_view"),
            "Missing association of elements 'baz' and 'foobar'",
            ErrorCode::Unassociated,
        )],
    );
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.in_declarative_region(
        "\
type foo is record
    bar: bit;
    baz: bit;
    foobar: bit;
    foobaz: bit;
end record;

view my_view of foo is
    bar: in;
end view;
    ",
    );
    let diag = builder.analyze();
    check_diagnostics(
        diag,
        vec![Diagnostic::new(
            code.s1("my_view"),
            "Missing association of elements 'baz', 'foobar' and 'foobaz'",
            ErrorCode::Unassociated,
        )],
    );

    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.in_declarative_region(
        "\
type foo is record
    a, b, c, d, e, f, g, h: bit;
end record;

view my_view of foo is
    a: in;
end view;
    ",
    );
    let diag = builder.analyze();
    check_diagnostics(
        diag,
        vec![Diagnostic::new(
            code.s1("my_view"),
            "Missing association of elements 'b', 'c', 'd' and 4 more",
            ErrorCode::Unassociated,
        )],
    );
}

#[test]
pub fn view_interface_declarations_must_be_views() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.code(
        "libname",
        "\
entity my_ent is
port (
    foo: view not_declared
);
end entity;
    ",
    );
    let diag = builder.analyze();
    check_diagnostics(
        diag,
        vec![Diagnostic::new(
            code.s1("not_declared"),
            "No declaration of 'not_declared'",
            ErrorCode::Unresolved,
        )],
    );

    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.code(
        "libname",
        "\
    entity my_ent is
    port (
        foo: view bit
    );
    end entity;
        ",
    );
    let diag = builder.analyze();
    check_diagnostics(
        diag,
        vec![Diagnostic::mismatched_kinds(
            code.s1("bit"),
            "type 'BIT' is not a view",
        )],
    );

    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    builder.code(
        "libname",
        "\
package x is
    type bar is record
        x: bit;
    end bar;
     
    view foo of bar is
        x: in;
    end view;
end x;

use work.x.all;

entity my_ent is
port (
    foo: view foo
);
end entity;
    ",
    );
    let diag = builder.analyze();
    check_no_diagnostics(&diag);
}

#[test]
pub fn view_interface_declaration_subtype_must_match_declared_subtype() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.code(
        "libname",
        "\
package x is
    type bar is record
        x: bit;
    end bar;
     
    view foo of bar is
        x: in;
    end view;
end x;

use work.x.all;

entity my_ent is
port (
    foo: view foo of bit
);
end entity;
    ",
    );
    let diag = builder.analyze();
    check_diagnostics(
        diag,
        vec![Diagnostic::new(
            code.s1("of bit").s1("bit"),
            "Specified subtype must match the subtype declared for the view",
            ErrorCode::TypeMismatch,
        )],
    );

    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.code(
        "libname",
        "\
package x is
    type bar is record
        x: bit;
    end bar;
     
    view foo of bar is
        x: in;
    end view;
end x;

use work.x.all;

entity my_ent is
port (
    x: view foo of bar
);
end entity;
    ",
    );
    let (root, diag) = builder.get_analyzed_root();
    check_no_diagnostics(&diag);
    let in_view = root
        .search_reference(
            code.source(),
            code.s1("x: view foo of bar").s1("bar").start(),
        )
        .unwrap();

    let declared = root
        .search_reference(
            code.source(),
            code.s1("type bar is record").s1("bar").start(),
        )
        .unwrap();
    assert_eq!(in_view, declared)
}

#[test]
fn view_reference() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.code(
        "libname",
        "\
package x is
    type bar is record
        x: bit;
    end bar;
     
    view foo of bar is
        x: in;
    end view;
end x;

use work.x;

entity my_ent is
port (
    x: view x.foo
);
end entity;
    ",
    );
    let (root, diag) = builder.get_analyzed_root();
    check_no_diagnostics(&diag);
    let in_view = root
        .search_reference(code.source(), code.s1("x: view x.foo").s1("foo").start())
        .unwrap();

    let declared = root
        .search_reference(
            code.source(),
            code.s1("view foo of bar is").s1("foo").start(),
        )
        .unwrap();
    assert_eq!(in_view, declared)
}

#[test]
fn converse_attribute() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    builder.code(
        "libname",
        "\
package my_pkg is
    type bar is record
        x: bit;
    end bar;
     
    view foo of bar is
        x: in;
    end view;

    alias foo_conv is foo'converse;
end my_pkg;

use work.my_pkg;

entity my_ent is
port (
    x: view my_pkg.foo;
    y: view my_pkg.foo_conv
);
end entity;
    ",
    );
    let diag = builder.analyze();
    check_no_diagnostics(&diag);
}

#[test]
fn view_declaration_in_cannot_be_assigned_to() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.code(
        "libname",
        "\
package my_pkg is
    type bar is record
        x: bit;
    end bar;
     
    view foo of bar is
        x: in;
    end view;
end my_pkg;

use work.my_pkg;

entity my_ent is
port ( y: view my_pkg.foo );
end entity;

architecture arch of my_ent is
begin
    y.x <= '1';
end arch;
    ",
    );
    let diag = builder.analyze();
    check_diagnostics(
        diag,
        vec![Diagnostic::new(
            code.s1("y.x"),
            "'y.x' has mode in and may not be the target of an assignment",
            ErrorCode::MismatchedKinds,
        )],
    );
}

// GitHub issue #324
#[test]
fn view_in_generic_package() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    builder.code(
        "libname",
        "\
package test_pkg is
    generic ( width : integer );
    type test_t is record
        a : bit;
    end record;

    view vone of test_t is
        a : in;
    end view;
    alias vtwo is vone'converse;
end package;

package w8_pkg is new work.test_pkg generic map (width => 8);

use work.w8_pkg.all;

entity test_sub_entity is
    port (
        my_if : view vone
    );
end entity;

use work.w8_pkg.all;

entity test_top_entity is
end entity;

architecture rtl of test_top_entity is
    signal my_sig : test_t;
begin
    my_if : entity work.test_sub_entity
        port map (
            my_if => my_sig
        );
end architecture;
    ",
    );
    check_no_diagnostics(&builder.analyze());
}

#[test]
fn arrays_of_views_with_matching_type() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    builder.code(
        "libname",
        "\
package test_pkg is
    type test_t is record
        a : bit;
    end record;

    view vone of test_t is
        a : in;
    end view;

    type test_array is array (natural range <>) of test_t;
end package;

use work.test_pkg.all;

entity test_sub_entity is
    port (
        my_if : view vone;
        my_array_if: view (vone) of test_array(0 to 1)
    );
end entity;
    ",
    );
    check_no_diagnostics(&builder.analyze());
}

#[test]
fn arrays_of_views_with_non_matching_type() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.code(
        "libname",
        "\
package test_pkg is
    type test_t is record
        a : bit;
    end record;

    view vone of test_t is
        a : in;
    end view;

    type test_array is array (natural range <>) of bit;
end package;

use work.test_pkg.all;

entity test_sub_entity is
    port (
        my_if : view vone;
        my_array_if: view (vone) of test_array(0 to 1)
    );
end entity;
    ",
    );
    check_diagnostics(
        builder.analyze(),
        vec![Diagnostic::new(
            code.s1("test_array(0 to 1)").s1("test_array"),
            "Array element type 'BIT' must match record type 'test_t' declared for the view",
            ErrorCode::TypeMismatch,
        )],
    )
}

#[test]
fn arrays_of_views_that_are_not_arrays() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.code(
        "libname",
        "\
package test_pkg is
    type test_t is record
        a : bit;
    end record;

    view vone of test_t is
        a : in;
    end view;
end package;

use work.test_pkg.all;

entity test_sub_entity is
    port (
        my_if : view vone;
        my_array_if: view (vone) of test_t
    );
end entity;
    ",
    );
    check_diagnostics(
        builder.analyze(),
        vec![Diagnostic::new(
            code.s1("view (vone) of test_t").s1("test_t"),
            "Subtype must be an array",
            ErrorCode::TypeMismatch,
        )],
    )
}

#[test]
fn undefined_name_for_nested_view() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.code(
        "libname",
        "\
package test is
    type my_rec_t is record
        a : bit;
     end record;
     view invalid_view of my_rec_t is
         a : view undefined_view;
     end view;
end package;
    ",
    );
    check_diagnostics(
        builder.analyze(),
        vec![Diagnostic::new(
            code.s1("undefined_view"),
            "No declaration of 'undefined_view'",
            ErrorCode::Unresolved,
        )],
    )
}

#[test]
fn view_that_is_not_a_view() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.code(
        "libname",
        "\
package test is
    constant some_constant : bit := '1';
    type my_rec_t is record
        a : bit;
     end record;
     view invalid_view of my_rec_t is
         a : view some_constant;
     end view;
end package;
    ",
    );
    check_diagnostics(
        builder.analyze(),
        vec![Diagnostic::new(
            code.s1("a : view some_constant;").s1("some_constant"),
            "Expected view",
            ErrorCode::MismatchedKinds,
        )],
    )
}

#[test]
pub fn view_may_be_declared_for_a_subtype() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    builder.code(
        "libname",
        "\
package pkg is
    type generic_t is record
        a: bit;
        b: bit_vector;
    end record;

    subtype concrete_t is generic_t(b(7 downto 0));

    view vone of concrete_t is
        a: in;
        b: out;
    end view;
end package pkg;
    ",
    );
    check_no_diagnostics(&builder.analyze());
}

#[test]
pub fn view_interface_declaration_may_use_a_subtype_of_the_view_type() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    builder.code(
        "libname",
        "\
package pkg is
    type generic_t is record
        a: bit;
        b: bit_vector;
    end record;

    subtype concrete_t is generic_t(b(7 downto 0));

    view vone of generic_t is
        a: in;
        b: out;
    end view;
end package pkg;

use work.pkg.all;

entity sub is
    port (
        x: view vone of concrete_t;
        y: view vone of generic_t(b(7 downto 0))
    );
end entity sub;
    ",
    );
    check_no_diagnostics(&builder.analyze());
}

#[test]
pub fn array_view_interface_declaration_may_use_a_subtype_of_the_element_type() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    builder.code(
        "libname",
        "\
package pkg is
    type generic_t is record
        a: bit;
        b: bit_vector;
    end record;

    subtype concrete_t is generic_t(b(7 downto 0));

    type concrete_arr_t is array (natural range <>) of concrete_t;
    subtype concrete_arr_4_t is concrete_arr_t(0 to 3);

    view vone of generic_t is
        a: in;
        b: out;
    end view;
end package pkg;

use work.pkg.all;

entity sub is
    port (
        x: view (vone) of concrete_arr_t(0 to 3);
        y: view (vone) of concrete_arr_4_t
    );
end entity sub;
    ",
    );
    check_no_diagnostics(&builder.analyze());
}

const VIEW_PKG: &str = "\
package pkg is
    type inner_t is record
        a : bit;
        b : bit;
    end record;

    view inner_v of inner_t is
        a : in;
        b : out;
    end view;

    type inner_arr is array (natural range <>) of inner_t;

    type outer_t is record
        x : bit;
        r : inner_t;
        arr : inner_arr(0 to 1);
    end record;

    view outer_v of outer_t is
        x : out;
        r : view inner_v;
        arr : view (inner_v);
    end view;
end package;

use work.pkg.all;
";

#[test]
fn view_element_with_mode_out_can_be_assigned_to() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    builder.code(
        "libname",
        &format!(
            "{VIEW_PKG}
entity ent is
    port (p : view outer_v);
end entity;

architecture arch of ent is
begin
    p.x <= '1';
    p.r.b <= '1';
    p.arr(0).b <= '1';
end architecture;"
        ),
    );
    check_no_diagnostics(&builder.analyze());
}

#[test]
fn nested_view_element_with_mode_in_cannot_be_assigned_to() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.code(
        "libname",
        &format!(
            "{VIEW_PKG}
entity ent is
    port (p : view outer_v);
end entity;

architecture arch of ent is
begin
    p.r.a <= '1';
    p.arr(1).a <= '1';
end architecture;"
        ),
    );
    check_diagnostics(
        builder.analyze(),
        vec![
            Diagnostic::new(
                code.s1("p.r.a"),
                "'p.r.a' has mode in and may not be the target of an assignment",
                ErrorCode::MismatchedKinds,
            ),
            Diagnostic::new(
                code.s1("p.arr(1).a"),
                "'p.arr(1).a' has mode in and may not be the target of an assignment",
                ErrorCode::MismatchedKinds,
            ),
        ],
    );
}

#[test]
fn view_with_in_elements_cannot_be_assigned_to_as_a_whole() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.code(
        "libname",
        &format!(
            "{VIEW_PKG}
entity ent is
    port (p : view outer_v);
end entity;

architecture arch of ent is
begin
    p.r <= ('0', '0');
end architecture;"
        ),
    );
    check_diagnostics(
        builder.analyze(),
        vec![Diagnostic::new(
            code.s1("p.r"),
            "'p.r' has elements of mode in and may not be the target of an assignment",
            ErrorCode::MismatchedKinds,
        )],
    );
}

#[test]
fn converse_swaps_modes() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.code(
        "libname",
        &format!(
            "{VIEW_PKG}
entity ent is
    port (p : view outer_v'converse);
end entity;

architecture arch of ent is
begin
    p.r.a <= '1';
    p.arr(0).a <= '1';
    p.x <= '1';
    p.r.b <= '1';
end architecture;"
        ),
    );
    check_diagnostics(
        builder.analyze(),
        vec![
            Diagnostic::new(
                code.s1("p.x"),
                "'p.x' has mode in and may not be the target of an assignment",
                ErrorCode::MismatchedKinds,
            ),
            Diagnostic::new(
                code.s1("p.r.b"),
                "'p.r.b' has mode in and may not be the target of an assignment",
                ErrorCode::MismatchedKinds,
            ),
        ],
    );
}

#[test]
fn converse_of_converse_is_the_original_view() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.code(
        "libname",
        &format!(
            "{VIEW_PKG}
package conv_pkg is
    alias outer_conv is outer_v'converse;
end package;

use work.conv_pkg.all;

entity ent is
    port (p : view outer_conv'converse);
end entity;

architecture arch of ent is
begin
    p.r.a <= '1';
end architecture;"
        ),
    );
    check_diagnostics(
        builder.analyze(),
        vec![Diagnostic::new(
            code.s1("p.r.a"),
            "'p.r.a' has mode in and may not be the target of an assignment",
            ErrorCode::MismatchedKinds,
        )],
    );
}

#[test]
fn element_may_only_be_part_of_view_once() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.in_declarative_region(
        "\
type rec_t is record
    a : bit;
end record;

view rec_v of rec_t is
    a : in;
    a : out;
end view;",
    );
    check_diagnostics(
        builder.analyze(),
        vec![Diagnostic::new(
            code.s1("a : out").s1("a"),
            "record element 'a' is already part of the view",
            ErrorCode::AlreadyAssociated,
        )],
    );
}

#[test]
fn nested_view_must_match_element_type() {
    let mut builder = LibraryBuilder::with_standard(VHDL2019);
    let code = builder.in_declarative_region(
        "\
type inner_t is record
    a : bit;
end record;

view inner_v of inner_t is
    a : in;
end view;

type outer_t is record
    r : bit;
    s : inner_t;
end record;

view outer_v of outer_t is
    r : view inner_v;
    s : view (inner_v);
end view;",
    );
    check_diagnostics(
        builder.analyze(),
        vec![
            Diagnostic::new(
                code.s1("r : view inner_v").s1("inner_v"),
                "record element 'r' is type 'BIT', but the view requires record type 'inner_t'",
                ErrorCode::TypeMismatch,
            ),
            Diagnostic::new(
                code.s1("s : view (inner_v)").s1("inner_v"),
                "record element 's' is record type 'inner_t', but the view requires an array of record type 'inner_t'",
                ErrorCode::TypeMismatch,
            ),
        ],
    );
}

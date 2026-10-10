; Adapted from the Neovim queries of https://github.com/jpt13653903/tree-sitter-vhdl
; Copyright (c) 2024 John-Philip Taylor, MIT License.

[
  (line_comment)
  (block_comment)
] @comment

(identifier) @variable

[
  "access"
  "after"
  "alias"
  "architecture"
  "array"
  "assert"
  "attribute"
  "begin"
  "block"
  "body"
  "buffer"
  "bus"
  "case"
  "component"
  "configuration"
  "constant"
  "context"
  "disconnect"
  "downto"
  "else"
  "elsif"
  "end"
  "entity"
  "exit"
  "file"
  "for"
  "force"
  "function"
  "generate"
  "generic"
  "group"
  "guarded"
  "if"
  "impure"
  "in"
  "inertial"
  "inout"
  "is"
  "label"
  "library"
  "linkage"
  "literal"
  "loop"
  "map"
  "new"
  "next"
  "of"
  "on"
  "out"
  "package"
  "parameter"
  "port"
  "postponed"
  "private"
  "procedure"
  "process"
  "property"
  "protected"
  "pure"
  "range"
  "record"
  "register"
  "reject"
  "release"
  "report"
  "return"
  "select"
  "sequence"
  "severity"
  "shared"
  "signal"
  "subtype"
  "then"
  "to"
  "transport"
  "type"
  "unaffected"
  "units"
  "until"
  "use"
  "variable"
  "view"
  "vunit"
  "wait"
  "when"
  "while"
  "with"
] @keyword

[
  (unary_operator)
  (logical_operator)
  (shift_operator)
  "mod"
  "not"
  "rem"
] @keyword.operator

[
  (ALL)
  (OTHERS)
  (DEFAULT)
  (OPEN)
  "<>"
  "null"
] @constant.builtin

[
  (condition_conversion)
  (relational_operator)
  (sign)
  (adding_operator)
  (exponentiate)
  (variable_assignment)
  (signal_assignment)
  "*"
  "/"
  "|"
  "=>"
] @operator

[
  "'"
  ","
  "."
  ":"
  ";"
] @punctuation.delimiter

[
  "("
  ")"
  "["
  "]"
  "<<"
  ">>"
] @punctuation.bracket

"@" @punctuation.special

[
  (decimal_integer)
  (decimal_float)
  (based_literal)
  (bit_string_literal)
] @number

[
  (string_literal)
  (string_literal_std_logic)
] @string

(character_literal) @string.special

[
  (library_constant)
  (library_constant_std_logic)
  (library_constant_character)
  (library_constant_debug)
] @constant.builtin

(library_constant_boolean) @boolean

[
  (unit)
  (library_constant_unit)
] @type.builtin

(library_function) @function.builtin

(library_type) @type.builtin

(library_namespace) @namespace

[
  (attribute_function)
  (attribute_impure_function)
  (attribute_mode_view)
  (attribute_pure_function)
  (attribute_range)
  (attribute_signal)
  (attribute_subtype)
  (attribute_type)
  (attribute_value)
  (library_attribute)
] @attribute

(label) @label

(selection
  (identifier) @property)

(_
  library: (_) @namespace)

(_
  package: (_) @namespace)

(_
  [
    entity: (_)
    component: (_)
    configuration: (_)
    architecture: (_)
    view: (_)
    type: (_)
  ] @type)

(_
  [
    entity: (name
      .
      (_) @type)
    component: (name
      .
      (_) @type)
    configuration: (name
      .
      (_) @type)
    view: (name
      .
      (identifier) @type)
    type: (name
      .
      (identifier) @type)
  ])

(_
  [
    function: (_)
    procedure: (_)
  ] @function)

(_
  attribute: (_) @attribute)

(_
  constant: (_) @constant)

(_
  generic: (_) @variable.parameter)

[
  (user_directive)
  (protect_directive)
  (warning_directive)
  (error_directive)
  (directive_body)
  (directive_constant_builtin)
  (directive_error)
  (directive_protect)
  (directive_warning)
] @preproc

(if_conditional_analysis
  [
    "if"
    "then"
  ] @preproc)

(elsif_conditional_analysis
  "elsif" @preproc)

(else_conditional_analysis
  "else" @preproc)

(end_conditional_analysis
  [
    "end"
    "if"
  ] @preproc)

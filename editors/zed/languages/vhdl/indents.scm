[
  (entity_declaration
    (end_entity) @end)
  (package_declaration
    (end_package) @end)
  (package_definition
    (end_package_body) @end)
  (configuration_declaration
    (end_configuration) @end)
  (context_declaration
    (end_context) @end)
  (component_declaration
    (end_component) @end)
  (block_configuration
    (end_for) @end)
  (component_configuration
    (end_for) @end)
  (record_type_definition
    (end_record) @end)
  (physical_type_definition
    (end_units) @end)
  (protected_type_declaration
    (protected_type_declaration_end) @end)
  (protected_type_body
    (protected_type_body_end) @end)
  (mode_view_declaration
    (end_view) @end)
  (case_statement
    (end_case) @end)
  (loop_statement
    (end_loop) @end)
  (for_generate_statement
    (end_generate) @end)
  (case_generate_statement
    (end_generate) @end)
] @indent

; The declarative part ends at "begin" and the statement part starts there.
[
  (architecture_definition
    (concurrent_block
      "begin" @end))
  (block_statement
    (concurrent_block
      "begin" @end))
  (process_statement
    (sequential_block
      "begin" @end))
  (subprogram_definition
    (sequential_block
      "begin" @end))
] @indent

; Starts after "then" so that it does not merge with the range of the first branch.
(if_statement_block
  (if_statement
    "then" @start)
  (end_if) @end) @indent

; Conditions that span several lines.
[
  (if_statement
    "if" @start
    "then" @end)
  (elsif_statement
    "elsif" @start
    "then" @end)
] @indent

; Starts after the keyword so that it does not merge with the subprogram body.
[
  (function_specification
    "function" @start)
  (procedure_specification
    "procedure" @start)
] @indent

[
  (alias_declaration)
  (attribute_specification)
  (constant_declaration)
  (file_declaration)
  (signal_declaration)
  (subprogram_declaration)
  (subtype_declaration)
  (variable_declaration)
  (assertion_statement)
  (concurrent_assertion_statement)
  (concurrent_conditional_signal_assignment)
  (concurrent_procedure_call_statement)
  (concurrent_selected_signal_assignment)
  (concurrent_simple_signal_assignment)
  (exit_statement)
  (next_statement)
  (procedure_call_statement)
  (report_statement)
  (return_statement)
  (selected_force_assignment)
  (selected_variable_assignment)
  (selected_waveform_assignment)
  (simple_force_assignment)
  (simple_release_assignment)
  (simple_variable_assignment)
  (simple_waveform_assignment)
  (wait_statement)
  (component_instantiation_statement)
  (concurrent_block)
  (sequential_block)
  (if_statement)
  (elsif_statement)
  (else_statement)
  (if_generate)
  (elsif_generate)
  (else_generate)
  (case_statement_alternative)
  (case_generate_alternative)
] @indent

(_
  "("
  ")" @end) @indent

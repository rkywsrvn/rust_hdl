(context_declaration
  "context" @context
  (identifier) @name) @item

(entity_declaration
  "entity" @context
  entity: (_) @name) @item

(architecture_definition
  "architecture" @context
  architecture: (_) @name
  "of" @context
  entity: (_) @context) @item

(configuration_declaration
  "configuration" @context
  configuration: (_) @name
  "of" @context
  entity: (_) @context) @item

(package_declaration
  "package" @context
  package: (_) @name) @item

(package_definition
  "package" @context
  "body" @context
  package: (_) @name) @item

(package_instantiation_declaration
  "package" @context
  package: (_) @name) @item

(component_declaration
  "component" @context
  component: (_) @name) @item

(type_declaration
  "type" @context
  type: (_) @name) @item

(subtype_declaration
  "subtype" @context
  type: (_) @name) @item

(_
  [
    (function_specification
      "function" @context
      function: (_) @name)
    (procedure_specification
      "procedure" @context
      procedure: (_) @name)
  ]) @item

(process_statement
  (label_declaration
    (label) @name)
  "process" @context) @item

(process_statement
  .
  "process" @name) @item

(block_statement
  (label_declaration
    (label) @name)
  "block" @context) @item

(component_instantiation_statement
  (label_declaration
    (label) @name)) @item

[
  (for_generate_statement
    (label_declaration
      (label) @name))
  (if_generate_statement
    (label_declaration
      (label) @name))
  (case_generate_statement
    (label_declaration
      (label) @name))
] @item

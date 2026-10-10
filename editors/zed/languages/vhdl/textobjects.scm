(line_comment)+ @comment.around

(block_comment) @comment.around

[
  (subprogram_definition
    (sequential_block
      "begin"
      (_)* @function.inside))
  (process_statement
    (sequential_block
      "begin"
      (_)* @function.inside))
] @function.around

(subprogram_declaration) @function.around

[
  (architecture_definition
    (concurrent_block
      "begin"
      (_)* @class.inside))
  (block_statement
    (concurrent_block
      "begin"
      (_)* @class.inside))
] @class.around

[
  (entity_declaration
    (entity_head
      "is"
      (_)* @class.inside))
  (package_declaration
    (package_declaration_body
      "is"
      (_)* @class.inside))
  (package_definition
    (package_definition_body
      "is"
      (_)* @class.inside))
  (component_declaration
    (component_body
      "is"
      (_)* @class.inside))
] @class.around

(configuration_declaration) @class.around

(function_declaration name: (identifier) @function)
(generator_function_declaration name: (identifier) @function)
(function_expression name: (identifier) @binding)
(variable_declarator name: (identifier) @binding)
(variable_declarator name: (array_pattern (identifier) @binding))
(variable_declarator name: (object_pattern (shorthand_property_identifier_pattern) @binding))
(pair_pattern value: (identifier) @binding)
(assignment_pattern left: (identifier) @binding)
(import_clause (identifier) @module)
(import_specifier name: (identifier) @module !alias)
(import_specifier alias: (identifier) @module)
(namespace_import (identifier) @module)
(method_definition name: (property_identifier) @function)
(method_definition name: (private_property_identifier) @function)
(pair key: (property_identifier) @field)
(catch_clause parameter: (identifier) @parameter)
(statement_block) @scope
(function_declaration) @scope
(generator_function_declaration) @scope
(function_expression) @scope
(generator_function) @scope
(arrow_function) @scope
(method_definition) @scope
(catch_clause) @scope
(for_statement) @scope
(for_in_statement) @scope

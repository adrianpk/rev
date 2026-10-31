(function_item name: (identifier) @function)
(struct_item name: (type_identifier) @type)
(enum_item name: (type_identifier) @type)
(trait_item name: (type_identifier) @type)
(type_item name: (type_identifier) @type)
(mod_item name: (identifier) @module)
(const_item name: (identifier) @constant)
(static_item name: (identifier) @constant)
(let_declaration pattern: (identifier) @binding)
(let_declaration pattern: (tuple_pattern (identifier) @binding))
(let_declaration pattern: (tuple_struct_pattern type: (_) (identifier) @binding))
(let_declaration pattern: (struct_pattern (field_pattern pattern: (identifier) @binding)))
(let_declaration pattern: (struct_pattern (field_pattern name: (shorthand_field_identifier) @binding)))
(parameter pattern: (identifier) @parameter)
(parameter pattern: (tuple_pattern (identifier) @parameter))
(closure_parameters (identifier) @parameter)
(for_expression pattern: (identifier) @binding)
(field_declaration name: (field_identifier) @field)
(enum_variant name: (identifier) @constant)
(let_declaration pattern: (reference_pattern (identifier) @binding))
(let_declaration pattern: (mut_pattern (identifier) @binding))
(parameter pattern: (reference_pattern (identifier) @parameter))
(parameter pattern: (mut_pattern (identifier) @parameter))
(closure_parameters (parameter pattern: (identifier) @parameter))
(let_condition pattern: (identifier) @binding)
(let_condition pattern: (tuple_struct_pattern type: (_) (identifier) @binding))
(match_pattern (tuple_struct_pattern type: (_) (identifier) @binding))

; Explicit lexical scopes for the captured declaration nodes.
(block) @scope
(function_item) @scope
(closure_expression) @scope
(for_expression) @scope
(if_expression) @scope
(while_expression) @scope
(match_arm) @scope

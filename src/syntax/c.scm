(function_declarator declarator: (identifier) @function)
(type_definition declarator: (type_identifier) @type)
(struct_specifier name: (type_identifier) @type)
(union_specifier name: (type_identifier) @type)
(enum_specifier name: (type_identifier) @type)
(enumerator name: (identifier) @constant)
(declaration declarator: (identifier) @binding)
(declaration declarator: (init_declarator declarator: (identifier) @binding))
(declaration declarator: (pointer_declarator declarator: (identifier) @binding))
(declaration declarator: (init_declarator declarator: (pointer_declarator declarator: (identifier) @binding)))
(declaration declarator: (array_declarator declarator: (identifier) @binding))
(parameter_declaration declarator: (identifier) @parameter)
(parameter_declaration declarator: (pointer_declarator declarator: (identifier) @parameter))
(parameter_declaration declarator: (array_declarator declarator: (identifier) @parameter))
(field_declaration declarator: (field_identifier) @field)
(field_declaration declarator: (pointer_declarator declarator: (field_identifier) @field))
(field_declaration declarator: (array_declarator declarator: (field_identifier) @field))
(declaration declarator: (function_declarator declarator: (parenthesized_declarator (pointer_declarator declarator: (identifier) @binding))))

; Explicit lexical scopes for the captured declaration nodes.
(compound_statement) @scope
(function_definition) @scope
(function_declarator) @scope
(for_statement) @scope
(if_statement) @scope
(while_statement) @scope

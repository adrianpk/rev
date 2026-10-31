(function_declaration name: (identifier) @function)
(method_declaration name: (field_identifier) @function)
(type_spec name: (type_identifier) @type)
(type_alias name: (type_identifier) @type)
(field_declaration name: (field_identifier) @field)
(const_spec name: (identifier) @constant)
(var_spec name: (identifier) @binding)
(short_var_declaration left: (expression_list (identifier) @binding))
(range_clause left: (expression_list (identifier) @binding))
(parameter_declaration name: (identifier) @parameter)
(variadic_parameter_declaration name: (identifier) @parameter)
(receive_statement left: (expression_list (identifier) @binding))
(type_parameter_declaration name: (identifier) @type)

; Explicit lexical scopes for the captured declaration nodes.
(block) @scope
(function_declaration) @scope
(method_declaration) @scope
(func_literal) @scope
(for_statement) @scope
(if_statement) @scope
(expression_case) @scope
(type_case) @scope
(communication_case) @scope

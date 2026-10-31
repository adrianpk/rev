; Pair key leaves include dotted and inline-table keys, never value strings.
(pair (bare_key) @toml-key)
(pair (quoted_key) @toml-key)
(dotted_key (bare_key) @toml-key)
(dotted_key (quoted_key) @toml-key)
; Header paths remain distinct from ordinary pair keys.
(table [(bare_key) (quoted_key) (dotted_key)] @table)
(table_array_element [(bare_key) (quoted_key) (dotted_key)] @table)

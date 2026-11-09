use super::*;

pub(in crate::macros::library) fn record(name: &str) -> Record {
    let source = Definition {
        retained_bytes: 64,
        id: 1,
        revision: 1,
        counter: Counter::default(),
        initial_format: "%d".into(),
        entry: Entry {
            profile: InteractionProfile::NonModal,
            mode: None,
        },
        steps: vec![Step::Text("λ\n".into())].into(),
    };
    Record::capture(
        MacroName::new(name).unwrap(),
        &Named {
            definition: Arc::new(source),
            source: (1, 1),
            description: None,
            saved: None,
        },
        None,
    )
    .unwrap()
}

// Every admitted typed native variant survives inert TOML projection, including
// raw prefix form, relative completion preconditions and role-qualified search.
#[test]
fn typed_roundtrip() {
    let prefixes = [
        PrefixArgument::default(),
        PrefixArgument::default().extend().unwrap(),
        PrefixArgument::default().digit(4).unwrap(),
        PrefixArgument::default().negate(),
    ];
    let mut source = record("alpha").definition(1).unwrap();
    let mut steps = vec![
        Step::Key(parse_key("F3").unwrap()),
        Step::Key(parse_key("ESC").unwrap()),
        Step::Text("λ\n\"quoted\"".into()),
        Step::Redisplay,
        Step::Query(QueryMode::Confirm),
        Step::Query(QueryMode::Edit),
        Step::Completion(LiteralCompletion {
            cursor: 2,
            edits: vec![CompletionEdit {
                start: -1,
                end: 1,
                expected: "éa".into(),
                text: "λ".into(),
            }],
        }),
    ];
    for (index, prefix) in prefixes.into_iter().enumerate() {
        steps.push(Step::Command(CommandStep {
            name: "isearch-forward".into(),
            argument: prefix,
            keys: Some(vec![Step::Key(parse_key("C-s").unwrap())]),
            arguments: vec![
                Argument {
                    role: Role::SearchRepeat(Direction::Backward),
                    value: "match".into(),
                },
                Argument {
                    role: Role::Search,
                    value: format!("find-{index}"),
                },
            ],
        }));
        steps.push(Step::Counter(counter::Operation::Insert(prefix)));
    }
    for operation in [
        counter::Operation::Set(i64::MIN),
        counter::Operation::Add(i64::MAX),
        counter::Operation::Reset,
        counter::Operation::Restore,
        counter::Operation::Format("%#08x".into()),
    ] {
        steps.push(Step::Counter(operation));
    }
    source.steps = steps.into();
    source.counter.format = "%x".into();
    let named = Named {
        definition: Arc::new(source),
        source: (1, 1),
        description: Some("A definition".into()),
        saved: None,
    };
    let expected = Record::capture(
        MacroName::new("alpha").unwrap(),
        &named,
        Some(&Sequence::parse("C-x C-k 8").unwrap()),
    )
    .unwrap();
    let bytes = expected.export().unwrap();
    assert!(bytes.contains("kind = \"search\""));
    let loaded = Catalog::decode(Some(bytes.as_bytes())).unwrap();
    assert!(loaded.macros[0] == expected);
    let definition = loaded.macros[0].definition(2).unwrap();
    assert_eq!(definition.initial_format, "%d");
    assert_eq!(definition.counter.format, "%x");
    for (index, prefix) in prefixes.into_iter().enumerate() {
        let Step::Command(command) = &definition.steps[7 + index * 2] else {
            panic!("command")
        };
        assert_eq!(command.argument.form(), prefix.form());
    }
    for state in [
        EditingState::Normal,
        EditingState::Insert,
        EditingState::Replace,
        EditingState::VisualCharacter,
        EditingState::VisualLine,
        EditingState::VisualBlock,
    ] {
        let mut modal = (*named.definition).clone();
        modal.entry = Entry {
            profile: InteractionProfile::Modal,
            mode: Some(state),
        };
        let record = Record::capture(
            MacroName::new("modal").unwrap(),
            &Named {
                definition: Arc::new(modal),
                source: (1, 1),
                description: None,
                saved: None,
            },
            None,
        )
        .unwrap();
        assert_eq!(
            Catalog::decode(Some(record.export().unwrap().as_bytes()))
                .unwrap()
                .macros[0]
                .definition(3)
                .unwrap()
                .entry
                .mode,
            Some(state)
        );
    }
}

// Complete structural validation precedes per-name admission. Bad schema,
// variants, fields, limits and preconditions reject the entire aggregate.
#[test]
fn structural_bounds() {
    let valid = record("alpha");
    for variant in [
        "schema",
        "zero",
        "id",
        "duplicate-id",
        "duplicate-name",
        "description",
        "steps",
        "format",
        "profile",
        "unknown-field",
        "unknown-kind",
        "prefix",
        "command-name",
        "completion",
        "encoded",
        "names",
    ] {
        let mut catalog = Catalog {
            schema_version: 1,
            revision: 1,
            macros: vec![valid.clone()],
        };
        match variant {
            "schema" => catalog.schema_version = 3,
            "zero" => catalog.revision = 0,
            "id" => catalog.macros[0].id = "A".repeat(32),
            "duplicate-id" => {
                let mut second = valid.clone();
                second.name = "bravo".into();
                catalog.macros.push(second);
            }
            "duplicate-name" => {
                let mut second = valid.clone();
                second.id = "0".repeat(32);
                catalog.macros.push(second);
            }
            "description" => catalog.macros[0].description = Some("λ".repeat(257)),
            "steps" => catalog.macros[0].steps = vec![Input::Redisplay; 4097],
            "format" => catalog.macros[0].counter.format = "%129d".into(),
            "profile" => catalog.macros[0].entry_state = Mode::Normal,
            "command-name" => {
                catalog.macros[0].steps = vec![Input::Command {
                    name: "123".into(),
                    argument: RawArgument::Absent,
                    keys: None,
                    arguments: Vec::new(),
                }]
            }
            "prefix" => {
                catalog.macros[0].steps = vec![Input::Counter {
                    action: CounterInput::Insert {
                        argument: RawArgument::PlainPrefix { depth: 0 },
                    },
                }]
            }
            "completion" => {
                catalog.macros[0].steps = vec![Input::Completion {
                    cursor: 0,
                    edits: vec![Edit {
                        start: i64::MIN,
                        end: i64::MAX,
                        expected: "".into(),
                        text: "".into(),
                    }],
                }]
            }
            "encoded" => {
                catalog.macros[0].steps = vec![Input::Text {
                    text: "A".repeat(DEFINITION_BYTES),
                }]
            }
            "names" => catalog.macros = vec![valid.clone(); 257],
            "unknown-field" | "unknown-kind" => {
                let text = valid.export().unwrap();
                let text = if variant == "unknown-field" {
                    format!("unexpected=1\n{text}")
                } else {
                    text.replace("kind = \"text\"", "kind = \"execute\"")
                };
                assert!(Catalog::decode(Some(text.as_bytes())).is_err(), "{variant}");
                continue;
            }
            _ => unreachable!(),
        }
        assert!(catalog.encode().is_err(), "{variant}");
    }
    let mut too_many = valid.clone();
    too_many.steps = vec![Input::Redisplay; 4097];
    let invalid = format!(
        "schema_version=1\nrevision=1\n[[macros]]\n{}",
        too_many.table().unwrap()
    );
    assert!(Catalog::decode(Some(invalid.as_bytes())).is_err());
    let mut collision = valid;
    collision.name = "forward-char".into();
    // A compiled-name conflict is structurally valid and quarantined later.
    assert!(
        Catalog {
            schema_version: 1,
            revision: 1,
            macros: vec![collision]
        }
        .encode()
        .is_ok()
    );
    let bytes = vec![b' '; LIBRARY_BYTES + 1];
    assert!(Catalog::decode(Some(&bytes)).is_err());
}

// Schema 2 adds an inert qualified variant without relaxing native names or
// schema-1 admission. JSON literals preserve null, booleans and nested values.
#[test]
fn plugin_schema() {
    let mut native = record("legacy");
    let old = native.export().unwrap();
    let mut source = native.definition(1).unwrap();
    source.steps = vec![Step::Plugin(PluginCall {
        command: rev_plugin_sdk::identity::CommandId::new("field.notes.collect").unwrap(),
        arguments: serde_json::json!({"data":[null, true, {"text":"λ"}]}),
        prefix: Some(-7),
    })]
    .into();
    let named = Named {
        definition: Arc::new(source),
        source: (1, 1),
        description: None,
        saved: None,
    };
    let plugin = Record::capture(MacroName::new("plugin-call").unwrap(), &named, None).unwrap();
    let bytes = plugin.export().unwrap();
    let loaded = Catalog::decode(Some(bytes.as_bytes())).unwrap();
    assert_eq!(loaded.schema_version, 2);
    let Step::Plugin(call) = &loaded.macros[0].definition(1).unwrap().steps[0] else {
        panic!("plugin step")
    };
    assert_eq!(
        call.arguments,
        serde_json::json!({"data":[null, true, {"text":"λ"}]})
    );
    assert_eq!(call.prefix, Some(-7));
    assert!(
        Catalog::decode(Some(
            bytes
                .replace("schema_version = 2", "schema_version = 1")
                .as_bytes()
        ))
        .is_err()
    );
    assert_eq!(
        Catalog::decode(Some(old.as_bytes()))
            .unwrap()
            .schema_version,
        1
    );
    for arguments in ["[]", "null", "{broken"] {
        native.steps = vec![Input::PluginCall {
            command: "field.notes.collect".into(),
            arguments_json: arguments.into(),
            prefix: None,
        }];
        assert!(native.definition(1).is_err());
    }
}

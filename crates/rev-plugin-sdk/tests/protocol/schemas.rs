use super::*;
use std::collections::BTreeSet;

// Literal cross-language objects freeze every method's fields, direction and
// result shape. Public schemas remain distinct from advertised implementation.
#[test]
fn method_schemas() {
    let handle = json!({"session":"session-a","id":"buffer-1"});
    let stamp = json!({"buffer":handle,"revision":"1"});
    let origin = json!({"package":"field.notes","operation":"op","depth":1});
    let range = json!({"start":0,"end":3});
    let page = json!({"cursor":null,"limit":10});
    let descriptor = json!({"stamp":stamp,"name":"notes","path":null,"kind":"text","read_only":false,"modified":true,"expected_disk":{"kind":"absent"},"owner":"field.notes","scalar_len":3});
    let context = json!({"invocation":"invoke-1","generation":"7","stamp":stamp,"pane":null,"point":1,"selection":range,"prefix":null,"origin":origin});
    let declaration = json!({"name":"collect","description":"Collect task lines.","parameters":[],"prefix":"none","recording":"reject"});
    let cases = vec![
        (
            "initialize",
            serde_json::to_value(offer(vec![])).unwrap(),
            serde_json::to_value(
                offer(vec![])
                    .acknowledge(&package(), List::default())
                    .unwrap(),
            )
            .unwrap(),
        ),
        (
            "buffers.list",
            json!({"page":page,"kind":"text"}),
            json!({"buffers":[descriptor],"next":null}),
        ),
        (
            "buffers.open",
            json!({"path":"notes.txt"}),
            descriptor.clone(),
        ),
        (
            "buffers.create",
            json!({"name":"result","text":"a🦀b","read_only":false}),
            descriptor.clone(),
        ),
        ("buffers.describe", json!({"buffer":handle}), descriptor),
        (
            "buffers.read",
            json!({"buffer":handle,"range":range,"expected_revision":"1"}),
            json!({"stamp":stamp,"range":range,"text":"a🦀b"}),
        ),
        (
            "buffers.show",
            json!({"buffer":handle,"pane":null,"placement":"split_vertical"}),
            json!({"pane":{"session":"session-a","id":"pane-1"}}),
        ),
        (
            "buffers.save",
            json!({"buffer":handle,"path":"notes.txt","expected_revision":"1","expected_disk":{"kind":"absent"}}),
            json!({"stamp":stamp,"expected_disk":{"kind":"observed","token":"disk-1"}}),
        ),
        (
            "buffers.close",
            json!({"buffer":handle,"expected_revision":"1"}),
            json!({"closed":true}),
        ),
        (
            "edits.apply",
            json!({"operation":"op","targets":[{"stamp":stamp,"replacements":[{"range":range,"text":"new"}]}],"origin":origin}),
            json!({"operation":"op","stamps":[stamp]}),
        ),
        (
            "commands.register",
            json!({"commands":[{"declaration":declaration,"handler":"collect"}]}),
            json!({"registrations":[{"id":"reg-1","command":"field.notes.collect"}]}),
        ),
        (
            "commands.unregister",
            json!({"registrations":["reg-1"]}),
            json!({}),
        ),
        (
            "commands.list",
            json!({"page":page,"query":"collect"}),
            json!({"commands":[{"id":"field.notes.collect","description":"Collect task lines.","parameters":[],"prefix":"none","recording":"reject","available":true,"noninteractive":true}],"next":null}),
        ),
        (
            "commands.invoke",
            json!({"command":"field.notes.collect","arguments":{},"context":context}),
            json!({"value":null}),
        ),
        (
            "command.invoke",
            json!({"command":"field.notes.collect","handler":"collect","arguments":{},"context":context}),
            json!({"value":null}),
        ),
        (
            "command.cancel",
            json!({"invocation":"invoke-1","generation":"7"}),
            json!({}),
        ),
        (
            "keymaps.bind",
            json!({"command":"field.notes.collect","keys":"C-c n","context":"editor","scope":{"kind":"buffer","buffer":handle}}),
            json!({"registration":"binding-1","status":"conflicting","effective":null,"underlying":[],"conflicts":[{"command":"field.notes.collect","provenance":"plugin","package":"field.notes"}],"diagnostic":"Conflicting plugin defaults."}),
        ),
        (
            "keymaps.unbind",
            json!({"registrations":["binding-1"]}),
            json!({}),
        ),
        (
            "events.subscribe",
            json!({"kinds":["changed"],"buffers":[handle]}),
            json!({"subscription":"sub-1","sequence":"0"}),
        ),
        (
            "events.unsubscribe",
            json!({"subscription":"sub-1"}),
            json!({}),
        ),
        (
            "messages.show",
            json!({"text":"Collected task lines.","severity":"info"}),
            json!({}),
        ),
        (
            "state.list",
            json!({"concern":"tasks","page":page}),
            json!({"records":[{"record":"one","version":"1"}],"next":null}),
        ),
        (
            "state.get",
            json!({"concern":"tasks","record":"one"}),
            json!({"status":"present","version":"1","value":{"done":false}}),
        ),
        (
            "state.put",
            json!({"concern":"tasks","record":"one","expected_version":null,"value":null}),
            json!({"version":"1"}),
        ),
        (
            "state.delete",
            json!({"concern":"tasks","record":"one","expected_version":"1"}),
            json!({}),
        ),
    ];
    let expected: BTreeSet<_> = METHODS.iter().map(|(name, _, _)| *name).collect();
    assert_eq!(
        cases
            .iter()
            .map(|(name, _, _)| *name)
            .collect::<BTreeSet<_>>(),
        expected
    );
    for (method, params, result) in cases {
        validate_params(method, params.clone(), &Limits::default())
            .unwrap_or_else(|error| panic!("{method} params: {error}"));
        validate_result(method, result, &Limits::default())
            .unwrap_or_else(|error| panic!("{method} result: {error}"));
        let mut unknown = params;
        unknown["unknown"] = json!(true);
        assert!(
            validate_params(method, unknown, &Limits::default()).is_err(),
            "{method} unknown field"
        );
    }
}

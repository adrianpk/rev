use std::path::{Component, Path};

use super::PartialLsp;
use super::validate::{Failure, Target};

pub(super) fn validate(value: &PartialLsp, complete: bool) -> Result<(), Failure> {
    limits(value)?;
    let projected = value.project();
    if projected.servers.len() > 32 {
        return Err(failure(&["lsp", "servers"], "at most 32 server entries"));
    }
    for (name, server) in &projected.servers {
        validate_server(name, server, value.servers.get(name), complete)?;
    }
    for (language, preference) in &projected.preferred {
        let key = ["lsp", "preferred", language];
        if !super::validate::language_id(language) {
            return Err(failure(
                &key,
                "expected lowercase language ID starting with a letter",
            ));
        }
        match preference {
            super::Preference::Native(false) => (),
            super::Preference::Native(true) => {
                return Err(failure(&key, "native-only must be false"));
            }
            super::Preference::Server(name) => {
                server_name(name)?;
                if complete
                    && !projected
                        .servers
                        .get(name)
                        .is_some_and(|s| s.languages.contains(language))
                {
                    return Err(failure(&key, "expected an existing compatible server"));
                }
            }
        }
    }
    Ok(())
}

fn validate_server(
    name: &str,
    server: &super::Server,
    declared: Option<&super::PartialServer>,
    complete: bool,
) -> Result<(), Failure> {
    server_name(name)?;
    required_fields(name, server, declared, complete)?;
    strings(name, "args", &server.args, true)?;
    markers(name, &server.root_markers)?;
    json_data(
        name,
        "initialization_options",
        &server.initialization_options,
    )?;
    json_data(name, "settings", &server.settings)?;
    environment(name, &server.env)?;
    Ok(())
}

fn required_fields(
    name: &str,
    server: &super::Server,
    declared: Option<&super::PartialServer>,
    complete: bool,
) -> Result<(), Failure> {
    let required = complete || declared.is_none();
    if required || declared.is_some_and(|v| v.command.is_some()) {
        strings(
            name,
            "command",
            std::slice::from_ref(&server.command),
            false,
        )?;
    }
    if required || declared.is_some_and(|v| v.languages.is_some()) {
        languages(name, &server.languages)?;
    }
    Ok(())
}

fn failure(key: &[&str], message: &str) -> Failure {
    Failure::at(key, message, Target::Value)
}

fn limits(value: &PartialLsp) -> Result<(), Failure> {
    for (name, value, max) in [
        ("request_timeout_ms", value.request_timeout_ms, 600_000),
        (
            "initialize_timeout_ms",
            value.initialize_timeout_ms,
            600_000,
        ),
        ("shutdown_timeout_ms", value.shutdown_timeout_ms, 600_000),
        ("max_message_bytes", value.max_message_bytes, 8_388_608),
        ("max_servers", value.max_servers, 8),
        ("max_pending_requests", value.max_pending_requests, 16),
    ] {
        if value.is_some_and(|value| !(1..=max).contains(&value)) {
            return Err(failure(
                &["lsp", name],
                &format!("expected integer 1..={max}"),
            ));
        }
    }
    Ok(())
}

fn server_name(name: &str) -> Result<(), Failure> {
    if name.is_empty() || name.len() > 128 || name.contains('\0') {
        return Err(failure(&["lsp", "servers", name], "invalid server name"));
    }
    Ok(())
}

fn strings(name: &str, leaf: &str, values: &[String], empty: bool) -> Result<(), Failure> {
    if values
        .iter()
        .any(|v| v.contains('\0') || (!empty && v.trim().is_empty()))
    {
        return Err(failure(
            &["lsp", "servers", name, leaf],
            "expected non-NUL launch strings",
        ));
    }
    Ok(())
}

fn languages(name: &str, values: &[String]) -> Result<(), Failure> {
    strings(name, "languages", values, false)?;
    if values.is_empty() || !values.iter().all(|id| super::validate::language_id(id)) {
        return Err(failure(
            &["lsp", "servers", name, "languages"],
            "expected at least one valid lowercase language ID",
        ));
    }
    Ok(())
}

fn environment(
    name: &str,
    env: &std::collections::BTreeMap<String, String>,
) -> Result<(), Failure> {
    let invalid = env.len() > 64
        || env.iter().any(|(key, value)| {
            key.is_empty()
                || key.len() > 128
                || key.contains(['\0', '='])
                || value.len() > 8192
                || value.contains('\0')
        })
        || env
            .iter()
            .map(|(key, value)| key.len() + value.len())
            .sum::<usize>()
            > 65536;
    if invalid {
        return Err(failure(
            &["lsp", "servers", name, "env"],
            "invalid environment: at most 64 entries, 128-byte names, 8 KiB values and 64 KiB total; names must be nonempty without NUL or =",
        ));
    }
    Ok(())
}

fn markers(name: &str, values: &[String]) -> Result<(), Failure> {
    let valid = |value: &String| {
        !value.contains('\0')
            && !value.is_empty()
            && Path::new(value)
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
    };
    if !values.iter().all(valid) {
        return Err(failure(
            &["lsp", "servers", name, "root_markers"],
            "expected relative marker paths without traversal",
        ));
    }
    Ok(())
}

fn json_data(name: &str, leaf: &str, value: &toml::Value) -> Result<(), Failure> {
    if let toml::Value::Float(value) = value
        && !value.is_finite()
    {
        return Err(failure(
            &["lsp", "servers", name, leaf],
            "JSON data must be finite",
        ));
    }
    match value {
        toml::Value::Table(table) => table.values().try_for_each(|v| json_data(name, leaf, v)),
        toml::Value::Array(array) => array.iter().try_for_each(|v| json_data(name, leaf, v)),
        _ => Ok(()),
    }
}

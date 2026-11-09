//! First-command example for the experimental Rust SDK. This process issues no
//! host requests and completes callbacks immediately, so it has no local RPC
//! deadlines to schedule. Its blocking stdin belongs only to this package.
#![forbid(unsafe_code)]

use rev_plugin_sdk::{
    client::{Client, Event},
    identity::{CommandName, OpaqueId, PackageId},
    methods::{
        CommandCancel, CommandInvoke, CommandsInvokeResult, Empty, Handler, Label, List, Method,
        Package,
    },
};
use std::io::{Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let package = Package {
        id: PackageId::new("field.hello")?,
        version: Label::new("0.1.0")?,
    };
    let handlers = List::new(vec![Handler {
        command: package.id.command(&CommandName::new("ping")?),
        handler: OpaqueId::new("ping")?,
    }])?;
    let mut client = Client::new(package, handlers)?;
    let mut input = std::io::stdin().lock();
    let mut output = std::io::stdout().lock();
    let mut bytes = [0; 8192];
    loop {
        // Flush each queued frame before waiting for another host message.
        while !client.output().is_empty() {
            let written = output.write(client.output())?;
            if written == 0 {
                return Err(std::io::Error::from(std::io::ErrorKind::WriteZero).into());
            }
            client.consume_output(written)?;
        }
        output.flush()?;
        let count = input.read(&mut bytes)?;
        if count == 0 {
            client.finish()?;
            return Ok(());
        }
        let mut offset = 0;
        while offset < count {
            let (used, event) = client.receive(&bytes[offset..count])?;
            offset += used;
            if let Some(Event::Request(callback)) = event {
                match callback.request().method.as_str() {
                    CommandInvoke::NAME => {
                        eprintln!("Hello command accepted.");
                        client.respond::<CommandInvoke>(
                            &callback,
                            Ok(CommandsInvokeResult {
                                value: Default::default(),
                            }),
                        )?;
                    }
                    CommandCancel::NAME => {
                        client.respond::<CommandCancel>(&callback, Ok(Empty {}))?
                    }
                    _ => unreachable!("SDK admits only negotiated host callbacks"),
                }
            }
        }
    }
}

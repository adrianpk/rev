// The fixture encoder follows the TZif byte layout, independently of the native
// reader. No host zone files, operator environment or reference source is copied.
pub fn tzif(
    version: u8,
    transitions: &[(i64, u8)],
    types: &[(i32, u8, u8)],
    names: &[u8],
    footer: &str,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    if version != 0 {
        bytes.extend(header(version, &[0, 0, 0, 0, 1, 1]));
        bytes.extend([0; 7]);
    }
    bytes.extend(header(
        version,
        &[
            0,
            0,
            0,
            transitions.len() as u32,
            types.len() as u32,
            names.len() as u32,
        ],
    ));
    for (seconds, _) in transitions {
        if version == 0 {
            bytes.extend((*seconds as i32).to_be_bytes())
        } else {
            bytes.extend(seconds.to_be_bytes())
        }
    }
    bytes.extend(transitions.iter().map(|t| t.1));
    for (offset, dst, name) in types {
        bytes.extend(offset.to_be_bytes());
        bytes.extend([*dst, *name]);
    }
    bytes.extend(names);
    if version != 0 {
        bytes.extend(format!("\n{footer}\n").bytes())
    }
    bytes
}
fn header(version: u8, counts: &[u32; 6]) -> Vec<u8> {
    let mut bytes = b"TZif".to_vec();
    bytes.push(version);
    bytes.extend([0; 15]);
    for count in counts {
        bytes.extend(count.to_be_bytes())
    }
    bytes
}

//! Bounded binary ingress using the ordinary regular-file identity owner.
use crate::{
    file::{self, Location, Observation},
    org::Cancellation,
};
use std::{fs::File, io::Read};
pub(in crate::org) fn read(
    location: &Location,
    limit: usize,
    cancel: &Cancellation,
) -> Result<Vec<u8>, String> {
    super::super::preview::check(cancel)?;
    let (mut input, opened) = open(location, limit)?;
    let bytes = bounded(&mut input, limit, cancel)?;
    let after = Observation::from_metadata(input.metadata().map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if opened != after
        || file::resolve(location.path())
            .map_err(|e| e.to_string())?
            .observation()
            != location.observation()
    {
        return Err("Preview file changed while reading".into());
    }
    Ok(bytes)
}
fn bounded(input: &mut File, limit: usize, cancel: &Cancellation) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut chunk = [0; 8192];
    loop {
        super::super::preview::check(cancel)?;
        let count = input.read(&mut chunk).map_err(|e| e.to_string())?;
        if count == 0 {
            return Ok(bytes);
        }
        if bytes.len() + count > limit {
            return Err("Preview byte limit reached".into());
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
}

fn open(location: &Location, limit: usize) -> Result<(File, Observation), String> {
    let expected = location.observation().ok_or("Preview file is missing")?;
    let input = File::open(location.path()).map_err(|e| e.to_string())?;
    let opened = Observation::from_metadata(input.metadata().map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if &opened != expected || opened.length > limit as u64 {
        return Err("Preview file changed or exceeds byte limit".into());
    }
    Ok((input, opened))
}

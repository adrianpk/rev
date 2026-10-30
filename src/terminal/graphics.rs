//! Kitty PNG transfers with exact owned identifiers and reversible placements.
use crate::{display::preview::Placement, window::Area};
use std::{
    io::{self, Write},
    sync::Arc,
};
pub(super) struct Images {
    slots: Vec<Slot>,
    next: u32,
    pending: bool,
}
struct Slot {
    id: u32,
    placement: Placement,
    seen: bool,
}
impl Default for Images {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            next: std::process::id().wrapping_mul(65537).max(1),
            pending: false,
        }
    }
}
/// Direct Kitty terminals admit PNG; multiplexers need protocol forwarding that
/// this frontend does not negotiate. There is no terminal query or idle probe.
pub fn supported() -> bool {
    std::env::var("TERM").is_ok_and(|s| s == "xterm-kitty")
        && std::env::var_os("TMUX").is_none()
        && std::env::var_os("STY").is_none()
}
impl Images {
    pub(super) fn present(
        &mut self,
        output: &mut impl Write,
        placements: &[Placement],
    ) -> io::Result<()> {
        self.pending = !self.slots.is_empty() || !placements.is_empty();
        for slot in &mut self.slots {
            slot.seen = false;
        }
        for placement in placements {
            self.place(output, placement)?;
        }
        self.retire(output, false)?;
        self.pending = false;
        Ok(())
    }
    fn place(&mut self, output: &mut impl Write, placement: &Placement) -> io::Result<()> {
        let old = self
            .slots
            .iter()
            .position(|s| !s.seen && Arc::ptr_eq(&s.placement.image, &placement.image));
        if let Some(index) = old {
            let slot = &mut self.slots[index];
            slot.seen = true;
            if slot.placement.area != placement.area {
                erase(output, slot.id, false)?;
                slot.placement = placement.clone();
                put(output, slot.id, placement.area)?;
            }
            return Ok(());
        }
        let id = self.next;
        self.next = self
            .next
            .checked_add(1)
            .ok_or_else(|| io::Error::other("PNG identifiers exhausted"))?;
        // Own cleanup before a partial upload or placement can affect output.
        self.slots.push(Slot {
            id,
            placement: placement.clone(),
            seen: true,
        });
        upload(output, id, &placement.image.png.bytes)?;
        put(output, id, placement.area)
    }
    pub(super) fn clear(&mut self, output: &mut impl Write) -> io::Result<()> {
        if self.pending {
            output.write_all(b"\x1b\\")?;
            output.flush()?;
        }
        self.pending = !self.slots.is_empty();
        self.retire(output, true)?;
        self.pending = false;
        Ok(())
    }
    fn retire(&mut self, output: &mut impl Write, all: bool) -> io::Result<()> {
        let mut index = 0;
        let mut error = None;
        while index < self.slots.len() {
            if !all && self.slots[index].seen {
                index += 1;
                continue;
            }
            match erase(output, self.slots[index].id, true) {
                Ok(()) => {
                    self.slots.remove(index);
                }
                Err(e) => {
                    error = Some(e);
                    index += 1;
                }
            }
        }
        output.flush()?;
        error.map_or(Ok(()), Err)
    }
}
fn erase(output: &mut impl Write, id: u32, data: bool) -> io::Result<()> {
    let kind = if data { 'I' } else { 'i' };
    write!(output, "\x1b_Ga=d,d={kind},i={id},q=2;\x1b\\")?;
    output.flush()
}
fn put(output: &mut impl Write, id: u32, area: Area) -> io::Result<()> {
    write!(
        output,
        "\x1b[{};{}H\x1b_Ga=p,i={id},c={},r={},C=1,z=-1,q=2;\x1b\\",
        area.row() + 1,
        area.column() + 1,
        area.width(),
        area.height()
    )
}
fn upload(output: &mut impl Write, id: u32, bytes: &[u8]) -> io::Result<()> {
    let count = bytes.len().div_ceil(3072);
    for (index, chunk) in bytes.chunks(3072).enumerate() {
        let more = u8::from(index + 1 < count);
        if index == 0 {
            write!(output, "\x1b_Ga=t,t=d,f=100,i={id},q=2,m={more};")?;
        } else {
            write!(output, "\x1b_Gm={more};")?;
        }
        output.write_all(&base64(chunk))?;
        output.write_all(b"\x1b\\")?;
    }
    Ok(())
}
fn base64(bytes: &[u8]) -> Vec<u8> {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = Vec::with_capacity(bytes.len().div_ceil(3) * 4);
    for word in bytes.chunks(3) {
        let a = word[0];
        let b = *word.get(1).unwrap_or(&0);
        let c = *word.get(2).unwrap_or(&0);
        output.extend_from_slice(&[
            TABLE[usize::from(a >> 2)],
            TABLE[usize::from((a & 3) << 4 | b >> 4)],
            if word.len() > 1 {
                TABLE[usize::from((b & 15) << 2 | c >> 6)]
            } else {
                b'='
            },
            if word.len() > 2 {
                TABLE[usize::from(c & 63)]
            } else {
                b'='
            },
        ]);
    }
    output
}

#[cfg(test)]
mod tests;

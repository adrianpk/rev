use super::{Level, Visibility};
use crate::command::OrgCommand;

impl Visibility {
    pub(super) fn dispatch(
        &mut self,
        command: OrgCommand,
        byte: usize,
    ) -> Result<(), &'static str> {
        match command {
            OrgCommand::GlobalCycle => self.global((self.global + 1) % 3),
            OrgCommand::Overview => self.global(1),
            OrgCommand::Contents => self.global(2),
            OrgCommand::ShowAll => self.global(0),
            _ => return self.local(command, byte),
        }
        Ok(())
    }
    fn global(&mut self, mode: u8) {
        self.global = mode;
        let level = [Level::Full, Level::Closed, Level::Children][usize::from(mode)];
        self.levels.fill(level);
        for (index, value) in self.levels.iter_mut().enumerate() {
            if self.model.archived(index) {
                *value = Level::Closed;
            }
        }
        self.drawers.fill(mode != 0);
    }
    fn local(&mut self, command: OrgCommand, byte: usize) -> Result<(), &'static str> {
        if matches!(
            command,
            OrgCommand::HideDrawer | OrgCommand::ShowDrawer | OrgCommand::ToggleDrawer
        ) {
            return self.drawer(command, byte);
        }
        if command == OrgCommand::Cycle {
            return self.cycle(byte);
        }
        let index = self.model.heading(byte).ok_or("No Org heading here")?;
        let level = match command {
            OrgCommand::HideSubtree => Level::Closed,
            OrgCommand::ShowChildren => Level::Children,
            _ => Level::Full,
        };
        self.subtree(index, level);
        Ok(())
    }
    fn cycle(&mut self, byte: usize) -> Result<(), &'static str> {
        if self.model.drawer(byte).is_some() {
            return self.drawer(OrgCommand::ToggleDrawer, byte);
        }
        let index = self
            .model
            .heading(byte)
            .ok_or("No Org heading or drawer here")?;
        if byte >= self.model.headings[index].body {
            return Err("TAB has no Org visibility target here");
        }
        let level = if self.model.archived(index) {
            Level::Closed
        } else {
            self.levels[index].next()
        };
        self.subtree(index, level);
        Ok(())
    }
    fn drawer(&mut self, command: OrgCommand, byte: usize) -> Result<(), &'static str> {
        let index = self.model.drawer(byte).ok_or("No Org drawer here")?;
        self.drawers[index] = match command {
            OrgCommand::HideDrawer => true,
            OrgCommand::ShowDrawer => false,
            _ => !self.drawers[index],
        };
        Ok(())
    }
    fn subtree(&mut self, index: usize, level: Level) {
        let h = &self.model.headings[index];
        let end = self
            .model
            .headings
            .partition_point(|child| child.start < h.end);
        let descendant = if level == Level::Children {
            Level::Closed
        } else {
            level
        };
        self.levels[index..end].fill(descendant);
        self.levels[index] = level;
        if level == Level::Full {
            for (drawer, hidden) in self.model.drawers.iter().zip(&mut self.drawers) {
                if h.start <= drawer.start && drawer.start < h.end {
                    *hidden = false;
                }
            }
        }
    }
}

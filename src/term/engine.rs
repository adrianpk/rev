use super::{
    Geometry,
    filter::Filter,
    limits::{COMBINING_SCALARS, CONTROL_BYTES, EXTRA_BYTES},
};
use alacritty_terminal::{
    Term,
    event::{Event, EventListener},
    grid::Dimensions,
    term::{
        self,
        cell::{Cell, CellExtra, Flags},
    },
    vte::ansi::{self, *},
};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    rc::Rc,
    sync::{Arc, Weak},
};
use unicode_width::UnicodeWidthChar;

#[derive(Default)]
pub(super) struct Effects {
    pub replies: Vec<u8>,
    pub title: Option<String>,
    pub limited: bool,
}
#[derive(Clone, Default)]
pub struct Listener(Rc<RefCell<Effects>>);
impl EventListener for Listener {
    fn send_event(&self, event: Event) {
        let mut effects = self.0.borrow_mut();
        if let Event::PtyWrite(text) = event {
            if effects.replies.len() + text.len() <= CONTROL_BYTES {
                effects.replies.extend(text.bytes());
            } else {
                effects.limited = true;
            }
        }
        // Child clipboard, host title, graphics and other host effects have no authority.
    }
}
struct Allocation {
    weak: Weak<CellExtra>,
    bytes: usize,
}
#[derive(Default)]
struct Extras {
    entries: BTreeMap<usize, Allocation>,
    bytes: usize,
}
impl Extras {
    fn admit(&mut self) -> bool {
        // Reserve before an operation that can allocate. One extra can retain
        // 64 scalars and at most one admitted 4 KiB hyperlink and ID pair.
        if self.bytes + 8192 > EXTRA_BYTES {
            self.prune();
        }
        self.bytes + 8192 <= EXTRA_BYTES
    }
    fn record(&mut self, cell: &Cell) {
        let Some(extra) = &cell.extra else { return };
        let key = Arc::as_ptr(extra) as usize;
        let combining = cell
            .zerowidth()
            .map_or(0, |chars| chars.len().max(4).next_power_of_two() * 4);
        let link = cell
            .hyperlink()
            .map_or(0, |link| link.uri().len() + link.id().len() + 64);
        let bytes = size_of::<CellExtra>() + 2 * size_of::<usize>() + combining + link;
        let allocation = Allocation {
            weak: Arc::downgrade(extra),
            bytes,
        };
        if let Some(old) = self.entries.insert(key, allocation) {
            self.bytes -= old.bytes;
        }
        self.bytes += bytes;
    }
    fn prune(&mut self) {
        self.entries
            .retain(|_, entry| entry.weak.strong_count() != 0);
        self.bytes = self.entries.values().map(|entry| entry.bytes).sum();
    }
}

pub(super) struct Engine {
    pub term: Term<Listener>,
    processor: Processor,
    filter: Filter,
    effects: Listener,
    extras: Extras,
    pub(super) prompts: super::prompts::Prompts,
}
impl Engine {
    pub fn new(size: Geometry, history: usize) -> Self {
        let effects = Listener::default();
        let config = term::Config {
            scrolling_history: history,
            osc52: term::Osc52::Disabled,
            ..Default::default()
        };
        Self {
            term: Term::new(config, &size, effects.clone()),
            processor: Processor::new(),
            filter: Filter::default(),
            effects,
            extras: Extras::default(),
            prompts: super::prompts::Prompts::new(history),
        }
    }
    pub fn advance(&mut self, bytes: &[u8]) -> bool {
        let mut handler = Admitted {
            term: &mut self.term,
            extras: &mut self.extras,
            effects: &self.effects,
            prompts: &mut self.prompts,
        };
        for byte in bytes {
            if let Some(bytes) = self.filter.byte(*byte) {
                handler.prompts.observe(bytes, handler.term);
                self.processor.advance(&mut handler, bytes);
            }
        }
        // Synchronized updates never create a retained 2 MiB parser backlog or
        // a timer. The editor presents damage at its own service/render boundary.
        self.processor.stop_sync(&mut handler);
        !self.effects.0.borrow().limited
    }
    pub fn effects(&self) -> std::cell::RefMut<'_, Effects> {
        self.effects.0.borrow_mut()
    }
    pub fn extra_bytes(&mut self) -> usize {
        self.extras.prune();
        self.extras.bytes
    }
    pub fn rejected_controls(&self) -> bool {
        self.filter.rejected
    }
}
struct Admitted<'a> {
    term: &'a mut Term<Listener>,
    extras: &'a mut Extras,
    effects: &'a Listener,
    prompts: &'a mut super::prompts::Prompts,
}
impl Admitted<'_> {
    fn combining_point(&self) -> alacritty_terminal::index::Point {
        let grid = self.term.grid();
        let mut point = grid.cursor.point;
        if !grid.cursor.input_needs_wrap {
            point.column.0 = point.column.0.saturating_sub(1);
        }
        if grid[point].flags.contains(Flags::WIDE_CHAR_SPACER) {
            point.column.0 = point.column.0.saturating_sub(1);
        }
        point
    }
    fn allocation(&mut self) -> bool {
        if self.extras.admit() {
            return true;
        }
        self.effects.0.borrow_mut().limited = true;
        false
    }
}
macro_rules! forward {
    ($($name:ident($($arg:ident: $kind:ty),*);)*) => {$ (
        fn $name(&mut self, $($arg: $kind),*) { self.term.$name($($arg),*); }
    )*};
}
impl Handler for Admitted<'_> {
    fn input(&mut self, c: char) {
        let combining = c.width() == Some(0);
        let point = self.combining_point();
        if combining
            && self.term.grid()[point]
                .zerowidth()
                .is_some_and(|chars| chars.len() >= COMBINING_SCALARS)
        {
            return;
        }
        if (combining || self.term.grid().cursor.template.extra.is_some()) && !self.allocation() {
            return;
        }
        self.prompts.input(self.term, c);
        self.term.input(c);
        if combining {
            self.extras.record(&self.term.grid()[point]);
        }
    }
    fn set_title(&mut self, title: Option<String>) {
        if title
            .as_ref()
            .is_none_or(|value| value.len() <= CONTROL_BYTES)
        {
            self.effects.0.borrow_mut().title = title;
        }
    }
    fn set_hyperlink(&mut self, link: Option<ansi::Hyperlink>) {
        if !self.allocation() {
            return;
        }
        self.term.set_hyperlink(link);
        self.extras.record(&self.term.grid().cursor.template);
    }
    fn terminal_attribute(&mut self, attr: ansi::Attr) {
        if !self.allocation() {
            return;
        }
        self.term.terminal_attribute(attr);
        self.extras.record(&self.term.grid().cursor.template);
    }
    fn linefeed(&mut self) {
        self.prompts.linefeed(self.term);
        self.term.linefeed();
    }
    fn newline(&mut self) {
        self.prompts.linefeed(self.term);
        self.term.newline();
    }
    fn scroll_up(&mut self, lines: usize) {
        self.prompts
            .scroll(lines.min(self.term.grid().screen_lines()));
        self.term.scroll_up(lines);
    }
    fn set_scrolling_region(&mut self, top: usize, bottom: Option<usize>) {
        self.prompts
            .region(top, bottom, self.term.grid().screen_lines());
        self.term.set_scrolling_region(top, bottom);
    }
    fn reset_state(&mut self) {
        self.prompts.reset();
        self.term.reset_state();
    }
    fn set_private_mode(&mut self, mode: PrivateMode) {
        let before = self.term.mode().contains(term::TermMode::ALT_SCREEN);
        self.term.set_private_mode(mode);
        if before != self.term.mode().contains(term::TermMode::ALT_SCREEN) {
            self.prompts.clear();
        }
    }
    fn unset_private_mode(&mut self, mode: PrivateMode) {
        let before = self.term.mode().contains(term::TermMode::ALT_SCREEN);
        self.term.unset_private_mode(mode);
        if before != self.term.mode().contains(term::TermMode::ALT_SCREEN) {
            self.prompts.clear();
        }
    }
    fn scroll_down(&mut self, lines: usize) {
        self.prompts.clear();
        self.term.scroll_down(lines);
    }
    fn insert_blank_lines(&mut self, lines: usize) {
        self.prompts.clear();
        self.term.insert_blank_lines(lines);
    }
    fn delete_lines(&mut self, lines: usize) {
        self.prompts.clear();
        self.term.delete_lines(lines);
    }
    fn reverse_index(&mut self) {
        self.prompts.clear();
        self.term.reverse_index();
    }
    fn clear_screen(&mut self, mode: ClearMode) {
        self.prompts.clear();
        self.term.clear_screen(mode);
    }
    fn decaln(&mut self) {
        self.prompts.clear();
        self.term.decaln();
    }
    fn insert_blank(&mut self, columns: usize) {
        self.prompts.erase_row(self.term.grid().cursor.point.line);
        self.term.insert_blank(columns);
    }
    fn erase_chars(&mut self, columns: usize) {
        self.prompts.erase_row(self.term.grid().cursor.point.line);
        self.term.erase_chars(columns);
    }
    fn delete_chars(&mut self, columns: usize) {
        self.prompts.erase_row(self.term.grid().cursor.point.line);
        self.term.delete_chars(columns);
    }
    fn clear_line(&mut self, mode: LineClearMode) {
        self.prompts.erase_row(self.term.grid().cursor.point.line);
        self.term.clear_line(mode);
    }
    // Title stacks, clipboard and extended keyboard stacks stay inert. The
    // ordinary engine cursor, screen, charset, color and input modes are native.
    forward! {
        set_cursor_style(arg0: Option<CursorStyle>);
        set_cursor_shape(_shape: CursorShape);
        goto(_line: i32, _col: usize);
        goto_line(_line: i32);
        goto_col(_col: usize);
        move_up(arg0: usize);
        move_down(arg0: usize);
        identify_terminal(_intermediate: Option<char>);
        device_status(arg0: usize);
        move_forward(_col: usize);
        move_backward(_col: usize);
        move_down_and_cr(_row: usize);
        move_up_and_cr(_row: usize);
        put_tab(_count: u16);
        backspace();
        carriage_return();
        bell();
        substitute();
        set_horizontal_tabstop();
        move_backward_tabs(_count: u16);
        move_forward_tabs(_count: u16);
        save_cursor_position();
        restore_cursor_position();
        clear_tabs(_mode: TabulationClearMode);
        set_tabs(_interval: u16);
        set_mode(_mode: Mode);
        unset_mode(_mode: Mode);
        report_mode(_mode: Mode);
        report_private_mode(_mode: PrivateMode);
        set_keypad_application_mode();
        unset_keypad_application_mode();
        set_active_charset(arg0: CharsetIndex);
        configure_charset(arg0: CharsetIndex, arg1: StandardCharset);
        set_color(arg0: usize, arg1: Rgb);
        dynamic_color_sequence(arg0: String, arg1: usize, arg2: &str);
        reset_color(arg0: usize);
        text_area_size_pixels();
        text_area_size_chars();
        report_keyboard_mode();
    }
}

use super::*;
impl Calendar<'_> {
    fn emit(&mut self, value: &str) -> Result<(), String> {
        document::check(self.cancel)?;
        if value.len() > self.snapshot.export_bytes.saturating_sub(self.output.len()) {
            return Err("Calendar export output byte limit reached".into());
        }
        self.output.push_str(value);
        Ok(())
    }
    fn piece(&mut self, value: &str) -> Result<(), String> {
        for c in value.chars() {
            if self.width + c.len_utf8() > 75 {
                self.emit("\r\n ")?;
                self.width = 1;
            }
            let mut bytes = [0; 4];
            self.emit(c.encode_utf8(&mut bytes))?;
            self.width += c.len_utf8();
        }
        Ok(())
    }
    fn start(&mut self, name: &str) -> Result<(), String> {
        self.width = 0;
        self.piece(name)?;
        self.piece(":")
    }
    pub(super) fn raw(&mut self, name: &str, value: &str) -> Result<(), String> {
        self.start(name)?;
        self.piece(value)?;
        self.emit("\r\n")
    }
    pub(super) fn property(&mut self, name: &str, value: &str) -> Result<(), String> {
        self.start(name)?;
        for c in value.replace("\r\n", "\n").replace('\r', "\n").chars() {
            self.scalar(c)?;
        }
        self.emit("\r\n")
    }
    fn scalar(&mut self, c: char) -> Result<(), String> {
        if c.is_control() && !['\t', '\n'].contains(&c) {
            return Err("Calendar text contains an unsupported control scalar".into());
        }
        let mut bytes = [0; 4];
        let text = match c {
            '\\' => "\\\\",
            ';' => "\\;",
            ',' => "\\,",
            '\n' => "\\n",
            _ => c.encode_utf8(&mut bytes),
        };
        self.piece(text)
    }
}

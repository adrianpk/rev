use super::Input;

impl Input {
    pub(crate) fn suppress_repeat(&mut self) {
        self.suppress_repeat = true;
    }
}

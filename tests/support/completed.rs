pub const HIDE: &str = "\x1b[?25l";
pub const SHOW: &str = "\x1b[?25h";

// Writer frames are complete only through their Show footer. A partial UTF-8
// chunk or split footer remains queued for the next bounded stream read.
pub fn completed(output: &str, cursor: &mut usize, observe: &mut impl FnMut(&str)) {
    while let Some(start) = output[*cursor..].find(HIDE) {
        let start = *cursor + start;
        let Some(end) = output[start..].find(SHOW) else {
            *cursor = start;
            return;
        };
        let end = start + end + SHOW.len();
        observe(&output[start..end]);
        *cursor = end;
    }
}

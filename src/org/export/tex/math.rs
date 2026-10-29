use super::super::math::fragment;
use super::*;
impl Document<'_> {
    pub(super) fn math(
        &self,
        tex: &mut Tex<'_>,
        range: Range<usize>,
        depth: usize,
        context: InlineContext,
    ) -> Result<(), String> {
        if depth >= 32 {
            return Err("TeX inline nesting limit reached".into());
        }
        let value = self.text.byte_slice(range.clone()).to_string();
        let nodes = crate::org::inline::objects(self.text, range.clone(), context, tex.cancel)?;
        let mut protected = nodes
            .iter()
            .filter(|n| n.kind != InlineKind::Entity)
            .peekable();
        let mut at = 0;
        let mut cursor = 0;
        while at < value.len() {
            document::check(tex.cancel)?;
            while protected
                .peek()
                .is_some_and(|n| n.source.end <= range.start + at)
            {
                protected.next();
            }
            if let Some(n) = protected
                .peek()
                .filter(|n| n.source.start <= range.start + at)
            {
                at = n.source.end - range.start;
                continue;
            }
            if let Some((end, body, display)) = fragment(&value, at, 4096) {
                self.objects(tex, range.start + cursor..range.start + at, depth, context)?;
                tex.math_fragment(&value[at..end], body, display)?;
                at = end;
                cursor = end;
            } else {
                at += value[at..]
                    .chars()
                    .next()
                    .expect("scalar boundary")
                    .len_utf8();
            }
        }
        self.objects(tex, range.start + cursor..range.end, depth, context)
    }
}
impl Tex<'_> {
    fn math_fragment(&mut self, source: &str, body: &str, display: bool) -> Result<(), String> {
        if !admitted(body) {
            self.diagnostic("Unsupported math retained as escaped data")?;
            return self.text(source);
        }
        self.emit(if display { "\\[" } else { "\\(" })?;
        self.emit(body)?;
        self.emit(if display { "\\]" } else { "\\)" })
    }
}
fn admitted(value: &str) -> bool {
    if value.contains("^^") {
        return false;
    }
    let mut chars = value.chars().peekable();
    let mut depth = 0;
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                let name: String =
                    std::iter::from_fn(|| chars.next_if(|c| c.is_ascii_alphabetic())).collect();
                if !command(&name) {
                    return false;
                }
            }
            c => {
                if !balance(c, &mut depth) {
                    return false;
                }
            }
        }
    }
    depth == 0
}
fn balance(c: char, depth: &mut usize) -> bool {
    match c {
        '{' => {
            *depth += 1;
            *depth <= 32
        }
        '}' => {
            let previous = *depth;
            *depth = depth.saturating_sub(1);
            previous > 0
        }
        '$' | '%' | '#' | '&' | '\0' => false,
        _ => true,
    }
}
fn command(value: &str) -> bool {
    [
        "alpha", "beta", "gamma", "delta", "epsilon", "theta", "lambda", "mu", "pi", "rho",
        "sigma", "tau", "phi", "chi", "psi", "omega", "Gamma", "Delta", "Theta", "Lambda", "Pi",
        "Sigma", "Phi", "Psi", "Omega", "frac", "sqrt", "sum", "prod", "int", "infty", "cdot",
        "times", "pm", "leq", "geq", "neq", "approx", "sin", "cos", "tan", "log", "ln", "exp",
        "text", "mathrm", "mathbf", "mathit", "left", "right",
    ]
    .contains(&value)
}

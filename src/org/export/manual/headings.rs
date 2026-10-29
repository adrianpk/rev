use super::*;
impl Manual<'_> {
    pub(super) fn preamble(&mut self) -> Result<(), String> {
        let title = self
            .snapshot
            .metadata
            .title
            .as_deref()
            .filter(|_| {
                self.snapshot.options.title
                    && !matches!(self.snapshot.request.scope, Scope::Region(_))
            })
            .unwrap_or("Org manual");
        if self.texinfo() {
            self.emit("\\input texinfo\n@documentencoding UTF-8\n@setfilename org-manual.info\n@settitle ")?;
            self.line(title)?;
            self.emit("\n@node Top\n@top ")?;
            self.line(title)?;
            self.emit("\n\n")?;
        } else {
            self.man_header(title)?;
        }
        self.frontmatter()
    }
    fn man_header(&mut self, title: &str) -> Result<(), String> {
        self.emit(".TH \"")?;
        self.line(title)?;
        self.emit("\" \"1\" \"")?;
        let date = self
            .snapshot
            .metadata
            .date
            .as_deref()
            .filter(|_| !matches!(self.snapshot.request.scope, Scope::Region(_)))
            .unwrap_or("");
        self.line(date)?;
        self.emit("\" \"\" \"\"\n")
    }
    fn frontmatter(&mut self) -> Result<(), String> {
        if matches!(self.snapshot.request.scope, Scope::Region(_)) {
            return Ok(());
        }
        let m = &self.snapshot.metadata;
        for (value, enabled) in [
            (&m.author, self.snapshot.options.author),
            (&m.email, self.snapshot.options.author),
            (&m.date, true),
            (&m.language, true),
        ] {
            let Some(value) = value.as_ref().filter(|_| enabled) else {
                continue;
            };
            if !self.texinfo() {
                self.emit(".PP\n")?;
            }
            self.text(value)?;
            self.emit("\n\n")?;
        }
        if self.texinfo() && self.snapshot.options.toc {
            self.emit("@contents\n\n")?;
        }
        Ok(())
    }
    fn number(&mut self, level: usize) -> String {
        self.numbers.resize(level, 0);
        self.numbers[level - 1] += 1;
        self.numbers
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(".")
    }
}
impl Document<'_> {
    pub(super) fn heading(
        &self,
        out: &mut Manual<'_>,
        id: usize,
        depth: usize,
    ) -> Result<(), String> {
        let level = self.levels[id];
        out.emitted.insert(format!("org-heading-{id}"));
        let range = super::super::render::headings::title_range(
            self.text,
            &self.nodes[id],
            &out.snapshot.options,
            &out.snapshot.todo,
        );
        let title = self.text.byte_slice(range).to_string();
        let number = out.number(level);
        if out.texinfo() {
            self.tex_heading(out, id, &title)?;
        } else {
            self.man_heading(out, level, (&number, &title))?;
        }
        self.children(out, id, depth)
    }
    fn tex_heading(&self, out: &mut Manual<'_>, id: usize, title: &str) -> Result<(), String> {
        let commands = if out.snapshot.options.numbered {
            ["chapter", "section", "subsection", "subsubsection"]
        } else {
            [
                "unnumbered",
                "unnumberedsec",
                "unnumberedsubsec",
                "unnumberedsubsubsec",
            ]
        };
        let command = commands
            .get(self.levels[id] - 1)
            .ok_or("Texinfo admits four relative heading levels")?;
        out.emit(&format!("@node org-heading-{id}\n@{command} "))?;
        out.line(title)?;
        out.emit("\n\n")
    }
    fn man_heading(
        &self,
        out: &mut Manual<'_>,
        level: usize,
        title: (&str, &str),
    ) -> Result<(), String> {
        out.emit(if level == 1 { ".SH \"" } else { ".SS \"" })?;
        if out.snapshot.options.numbered {
            out.emit(&format!("{} ", title.0))?;
        } else {
            out.emit(&"  ".repeat(level.saturating_sub(2)))?;
        }
        out.line(title.1)?;
        out.emit("\"\n")
    }
}

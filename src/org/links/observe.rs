use super::*;
impl Document {
    pub(crate) fn observe(
        text: Rope,
        policy: &crate::config::Org,
        cancel: &Cancellation,
    ) -> Result<Self, String> {
        let raw = structure::observe(text.clone(), Limits::default(), cancel);
        if raw.coverage != Coverage::Complete
            || raw.nodes.iter().any(|n| n.kind == StructureKind::Malformed)
        {
            return Err("Link source has incomplete or malformed structure".into());
        }
        let properties = Properties::observe_nodes(raw.nodes.iter(), true, &text, cancel);
        let workflow = workflow(&raw.nodes, &text, &policy.todo)?;
        let mut doc = Self {
            text,
            links: Vec::new(),
            anchors: Vec::new(),
            headings: Vec::new(),
            properties,
        };
        for node in &raw.nodes {
            doc.node(node, &workflow, cancel)?;
        }
        doc.links.sort_by_key(|l| l.bytes.start);
        doc.links.dedup_by(|a, b| a.bytes == b.bytes);
        doc.anchors.sort_by_key(|a| a.bytes.start);
        doc.anchors
            .dedup_by(|a, b| a.bytes == b.bytes && a.name == b.name);
        let bytes = doc
            .links
            .iter()
            .map(|l| l.target.len() + l.description.len())
            .sum::<usize>()
            + doc.anchors.iter().map(|a| a.name.len()).sum::<usize>()
            + doc.headings.iter().map(|h| h.title.len()).sum::<usize>();
        if doc.links.len() + doc.anchors.len() + doc.headings.len() > 4096 || bytes > 1048576 {
            return Err("Link observations exceed 4096 records or 1 MiB labels".into());
        }
        Ok(doc)
    }
    fn node(
        &mut self,
        node: &structure::Raw,
        workflow: &crate::org::task::Workflow,
        cancel: &Cancellation,
    ) -> Result<(), String> {
        if cancel.is_cancelled() {
            return Err("Link observation cancelled".into());
        }
        self.metadata(node, workflow)?;
        if node.context != Some(InlineContext::Standard) {
            return Ok(());
        }
        let objects = crate::org::inline::objects(
            &self.text,
            node.content.clone(),
            InlineContext::Standard,
            cancel,
        )?;
        for object in objects {
            self.object(object)?;
        }
        if self.links.len() + self.anchors.len() + self.headings.len() > 4096 {
            return Err("Link record limit reached".into());
        }
        Ok(())
    }
    fn metadata(
        &mut self,
        node: &structure::Raw,
        workflow: &crate::org::task::Workflow,
    ) -> Result<(), String> {
        if matches!(node.kind, StructureKind::Heading { .. }) {
            self.heading(node, workflow)?;
        }
        if matches!(
            node.kind,
            StructureKind::AffiliatedKeyword | StructureKind::Keyword
        ) {
            self.named(node)?;
        }
        Ok(())
    }
    fn object(&mut self, object: crate::org::inline::Raw) -> Result<(), String> {
        match object.kind {
            InlineKind::Link(_) => {
                if object.source.len() > 8192 || object.content.len() > 4096 {
                    return Err("Link source exceeds its 8192-byte/4096-target profile".into());
                }
                self.links.push(Link {
                    bytes: object.source,
                    target: self.text.byte_slice(object.content).to_string(),
                    description: object
                        .detail
                        .map_or_else(String::new, |r| self.text.byte_slice(r).to_string()),
                });
            }
            InlineKind::Target | InlineKind::RadioTarget => {
                if object.content.len() > 4096 {
                    return Err("Link target name exceeds 4096 bytes".into());
                }
                self.anchors.push(Anchor {
                    name: self.text.byte_slice(object.content).to_string(),
                    bytes: object.source,
                });
            }
            _ => (),
        }
        Ok(())
    }
    fn heading(
        &mut self,
        node: &structure::Raw,
        workflow: &crate::org::task::Workflow,
    ) -> Result<(), String> {
        let header = crate::org::visibility::query_header(node, &self.text)?;
        let title = if workflow.state(&header.word).is_some() {
            header.title[header.word.len()..].trim()
        } else {
            header.title.trim()
        };
        let title = if title.starts_with("[#") {
            title
                .split_once(']')
                .ok_or("Malformed link heading priority")?
                .1
                .trim()
        } else {
            title
        };
        self.headings.push(Heading {
            byte: node.source.start,
            end: node.source.end,
            title: title.to_owned(),
        });
        Ok(())
    }
    fn named(&mut self, node: &structure::Raw) -> Result<(), String> {
        let Some(name) = &node.name else {
            return Ok(());
        };
        if self
            .text
            .byte_slice(name.clone())
            .to_string()
            .eq_ignore_ascii_case("NAME")
        {
            if node.content.len() > 4096 {
                return Err("Named link target exceeds 4096 bytes".into());
            }
            self.anchors.push(Anchor {
                name: self
                    .text
                    .byte_slice(node.content.clone())
                    .to_string()
                    .trim()
                    .into(),
                bytes: node.source.clone(),
            });
        }
        Ok(())
    }
}
fn workflow(
    nodes: &[structure::Raw],
    text: &Rope,
    policy: &crate::config::Todo,
) -> Result<crate::org::task::Workflow, String> {
    let mut workflow = crate::org::task::Workflow::default();
    for node in nodes.iter().filter(|n| n.kind == StructureKind::Keyword) {
        let Some(name) = &node.name else {
            continue;
        };
        let key = text
            .byte_slice(name.clone())
            .to_string()
            .to_ascii_uppercase();
        if matches!(key.as_str(), "TODO" | "SEQ_TODO" | "TYP_TODO") {
            if node.content.len() > 4096 {
                return Err("Link workflow declaration exceeds 4096 bytes".into());
            }
            workflow.declaration(&text.byte_slice(node.content.clone()).to_string());
        }
    }
    workflow.effective(policy)
}

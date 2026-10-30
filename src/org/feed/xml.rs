//! Bounded XML observations. No DTD, external entities, network or evaluation.
mod lex;
mod names;
use crate::org::Cancellation;
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Name {
    pub space: String,
    pub local: String,
}
#[derive(Debug)]
pub(super) enum Piece {
    Text(String),
    Child(usize),
}
#[derive(Debug)]
pub(super) struct Node {
    pub name: Name,
    pub attrs: BTreeMap<Name, String>,
    pub children: Vec<usize>,
    parent: Option<usize>,
    pieces: Vec<Piece>,
    raw: String,
}
#[derive(Debug)]
pub(super) struct Document {
    pub nodes: Vec<Node>,
    pub root: usize,
}
struct Frame {
    index: usize,
    names: BTreeMap<String, String>,
}
struct Builder {
    nodes: Vec<Node>,
    stack: Vec<Frame>,
    root: Option<usize>,
    declaration: bool,
}
impl Document {
    pub fn parse(value: &str, cancel: &Cancellation) -> Result<Self, String> {
        let value = value.strip_prefix('\u{feff}').unwrap_or(value);
        let mut reader = lex::Reader::new(value, cancel);
        let mut builder = Builder {
            nodes: Vec::new(),
            stack: Vec::new(),
            root: None,
            declaration: false,
        };
        while let Some(token) = reader.next()? {
            builder.token(token)?;
        }
        if !builder.stack.is_empty() {
            return Err("Unclosed XML element".into());
        }
        Ok(Self {
            nodes: builder.nodes,
            root: builder.root.ok_or("XML has no document element")?,
        })
    }
    pub fn text(&self, index: usize, cancel: &Cancellation) -> Result<String, String> {
        let mut work = vec![&self.nodes[index].pieces[..]];
        let mut result = String::new();
        while let Some(pieces) = work.pop() {
            check(cancel)?;
            let Some((first, rest)) = pieces.split_first() else {
                continue;
            };
            work.push(rest);
            match first {
                Piece::Text(value) => result.push_str(value),
                Piece::Child(child) => {
                    if self.nodes[*child].name.space == "http://www.w3.org/1999/xhtml" {
                        result.push(' ');
                    }
                    work.push(&self.nodes[*child].pieces);
                }
            }
            if result.len() > 65536 {
                return Err("Feed field exceeds 64 KiB".into());
            }
        }
        Ok(result)
    }
}
impl Node {
    pub fn attr(&self, local: &str) -> Option<&str> {
        self.attrs
            .get(&Name {
                space: String::new(),
                local: local.into(),
            })
            .map(String::as_str)
    }
}
impl Builder {
    fn token(&mut self, token: lex::Token) -> Result<(), String> {
        match token {
            lex::Token::Open(tag) => self.open(tag),
            lex::Token::Close(name) => self.close(&name),
            lex::Token::Text(text) => self.text(text),
            lex::Token::Declaration => {
                if self.root.is_some() || self.declaration {
                    return Err("Misplaced XML declaration".into());
                }
                self.declaration = true;
                Ok(())
            }
        }
    }
    fn open(&mut self, tag: lex::Tag) -> Result<(), String> {
        if self.nodes.len() >= 100000 || self.stack.len() >= 64 {
            return Err("Feed XML node/depth limit reached".into());
        }
        let inherited = self.stack.last().map(|f| &f.names);
        let names = names::bindings(inherited, &tag.attrs)?;
        let name = names::resolve(&tag.name, &names, false)?;
        let attrs = names::attributes(&tag.attrs, &names)?;
        let index = self.nodes.len();
        self.attach(index)?;
        self.nodes.push(Node {
            name,
            attrs,
            raw: tag.name.clone(),
            parent: self.stack.last().map(|f| f.index),
            children: Vec::new(),
            pieces: Vec::new(),
        });
        self.stack.push(Frame { index, names });
        if tag.empty {
            self.close(&tag.name)?;
        }
        Ok(())
    }
    fn attach(&mut self, index: usize) -> Result<(), String> {
        if let Some(frame) = self.stack.last() {
            self.nodes[frame.index].children.push(index);
            self.nodes[frame.index].pieces.push(Piece::Child(index));
        } else if self.root.replace(index).is_some() {
            return Err("XML has multiple document elements".into());
        }
        Ok(())
    }
    fn close(&mut self, name: &str) -> Result<(), String> {
        let frame = self.stack.pop().ok_or("Unexpected XML closing tag")?;
        if self.nodes[frame.index].raw != name {
            return Err("Mismatched XML closing tag".into());
        }
        Ok(())
    }
    fn text(&mut self, text: String) -> Result<(), String> {
        if let Some(frame) = self.stack.last() {
            self.nodes[frame.index].pieces.push(Piece::Text(text));
        } else if !text.trim().is_empty() {
            return Err("Text outside XML document element".into());
        }
        Ok(())
    }
}
pub(super) fn check(cancel: &Cancellation) -> Result<(), String> {
    if cancel.is_cancelled() {
        Err("Feed observation cancelled or expired".into())
    } else {
        Ok(())
    }
}

impl Document {
    pub fn base(&self, index: usize, initial: Option<&str>) -> Result<Option<String>, String> {
        let key = Name {
            space: "http://www.w3.org/XML/1998/namespace".into(),
            local: "base".into(),
        };
        let mut parents = vec![index];
        while let Some(parent) = self.nodes[*parents.last().expect("nonempty")].parent {
            parents.push(parent);
        }
        let mut base = initial.map(str::to_owned);
        for i in parents.into_iter().rev() {
            if let Some(value) = self.nodes[i].attrs.get(&key) {
                base = Some(super::uri::resolve(base.as_deref(), value)?);
            }
        }
        Ok(base)
    }
}

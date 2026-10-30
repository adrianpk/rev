//! Explicit RSS/Atom observations. Feed text is data, never configuration/code.
mod status;
mod target;
pub use target::Plan;
mod uri;
mod xml;
use crate::{config::OrgLimits, org::Cancellation};
use std::collections::BTreeSet;
const ATOM: &str = "http://www.w3.org/2005/Atom";
const RSS: &str = "http://purl.org/rss/1.0/";
const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
/// Literal item fields, qualified by a stable source identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub id: String,
    pub title: String,
    pub link: Option<String>,
    pub body: String,
    pub published: Option<String>,
}
/// One complete bounded observation. Invalid/truncated input never yields items.
#[derive(Clone, Debug)]
pub struct Feed {
    pub entries: Vec<Entry>,
}
impl Feed {
    pub fn parse(
        value: &str,
        base: Option<&str>,
        limits: &OrgLimits,
        cancel: &Cancellation,
    ) -> Result<Self, String> {
        bounds(value, limits)?;
        let document = xml::Document::parse(value, cancel)?;
        let (container, space, item) = container(&document)?;
        let mut entries = Vec::new();
        let mut ids = BTreeSet::new();
        for index in document.nodes[container].children.iter().copied() {
            xml::check(cancel)?;
            let node = &document.nodes[index];
            if !item_node(node, space, item) {
                continue;
            }
            if entries.len() >= limits.feed_items {
                return Err("Feed item limit reached; nothing imported".into());
            }
            let entry = Item {
                document: &document,
                index,
                base,
                cancel,
            }
            .entry()?;
            if !ids.insert(entry.id.clone()) {
                return Err("Duplicate item identity in feed".into());
            }
            entries.push(entry);
        }
        Ok(Self { entries })
    }
}
fn bounds(value: &str, limits: &OrgLimits) -> Result<(), String> {
    if value.len() > limits.feed_bytes
        || !(1..=10000).contains(&limits.feed_items)
        || !(1..=16777216).contains(&limits.feed_bytes)
    {
        return Err("Feed byte/item limit reached".into());
    }
    Ok(())
}
fn container(document: &xml::Document) -> Result<(usize, &str, &str), String> {
    let root = &document.nodes[document.root];
    match (root.name.space.as_str(), root.name.local.as_str()) {
        (ATOM, "feed") => Ok((document.root, ATOM, "entry")),
        (RDF, "RDF") => Ok((document.root, RSS, "item")),
        ("", "rss") => rss_container(document),
        _ => Err("Expected native RSS or Atom feed document".into()),
    }
}
fn rss_container(document: &xml::Document) -> Result<(usize, &str, &str), String> {
    let root = &document.nodes[document.root];
    if !root
        .attr("version")
        .is_some_and(|v| ["0.91", "0.92", "2.0"].contains(&v))
    {
        return Err("Unsupported RSS version".into());
    }
    let channels: Vec<_> = root
        .children
        .iter()
        .copied()
        .filter(|i| {
            let n = &document.nodes[*i];
            n.name.local == "channel" && n.name.space.is_empty()
        })
        .collect();
    if channels.len() != 1 {
        return Err("RSS requires one channel".into());
    }
    Ok((channels[0], "", "item"))
}
struct Item<'a> {
    document: &'a xml::Document,
    index: usize,
    base: Option<&'a str>,
    cancel: &'a Cancellation,
}
impl Item<'_> {
    fn entry(&self) -> Result<Entry, String> {
        let atom = self.document.nodes[self.index].name.space == ATOM;
        let (title, body) = self.content(atom)?;
        let link = self.link(atom)?;
        let id = self.identity(atom, &link)?;
        let published = self
            .field(if atom { "published" } else { "pubDate" })?
            .or(self.field("updated")?);
        Ok(Entry {
            id,
            title,
            link,
            body,
            published,
        })
    }
    fn content(&self, atom: bool) -> Result<(String, String), String> {
        let body = self
            .field(if atom { "content" } else { "description" })?
            .or(self.field("summary")?)
            .unwrap_or_default();
        let title = self
            .field("title")?
            .unwrap_or_else(|| body.lines().next().unwrap_or("").into());
        let title = title.split_whitespace().collect::<Vec<_>>().join(" ");
        if title.is_empty() || title.len() > 4096 {
            return Err(
                "Feed item requires a title/description of at most 4096 title bytes".into(),
            );
        }
        Ok((title, body))
    }
    fn field(&self, local: &str) -> Result<Option<String>, String> {
        let space = &self.document.nodes[self.index].name.space;
        let mut values = self.document.nodes[self.index].children.iter().filter(|i| {
            let name = &self.document.nodes[**i].name;
            name.local == local && &name.space == space
        });
        let Some(index) = values.next() else {
            return Ok(None);
        };
        if values.next().is_some() {
            return Err(format!("Duplicate feed field {local}"));
        }
        self.document
            .text(*index, self.cancel)
            .map(|s| Some(s.trim().into()))
    }
    fn identity(&self, atom: bool, link: &Option<String>) -> Result<String, String> {
        let explicit = self.field(if atom { "id" } else { "guid" })?;
        let about = self.document.nodes[self.index].attrs.get(&xml::Name {
            space: RDF.into(),
            local: "about".into(),
        });
        let id = explicit
            .or_else(|| about.cloned())
            .or_else(|| if atom { None } else { link.clone() })
            .ok_or("Feed item has no stable id/guid/link")?;
        if id.is_empty() || id.len() > 4096 || id.chars().any(char::is_control) {
            return Err("Invalid feed item identity".into());
        }
        Ok(id)
    }
    fn link(&self, atom: bool) -> Result<Option<String>, String> {
        let link = if atom {
            self.atom_link()?
        } else {
            self.field("link")?.or(self.permalink()?)
        };
        let Some(link) = link else {
            return Ok(None);
        };
        let base = self.document.base(self.index, self.base)?;
        uri::resolve(base.as_deref(), &link).map(Some)
    }
    fn atom_link(&self) -> Result<Option<String>, String> {
        for index in &self.document.nodes[self.index].children {
            let node = &self.document.nodes[*index];
            if node.name.space != ATOM || node.name.local != "link" {
                continue;
            }
            if node.attr("rel").is_none_or(|r| r == "alternate") {
                return Ok(node.attr("href").map(str::to_owned));
            }
        }
        Ok(None)
    }
    fn permalink(&self) -> Result<Option<String>, String> {
        let node = &self.document.nodes[self.index];
        let guid = node
            .children
            .iter()
            .map(|i| &self.document.nodes[*i])
            .find(|n| n.name.local == "guid" && n.name.space == node.name.space);
        if guid.is_some_and(|n| n.attr("isPermaLink") == Some("false")) {
            return Ok(None);
        }
        Ok(self
            .field("guid")?
            .filter(|s| s.starts_with("http://") || s.starts_with("https://")))
    }
}

impl Feed {
    /// Prepare one destination transaction. The ordinary canonical edit/save
    /// owner must still revalidate and publish `insertion`; this never writes.
    pub fn plan(
        self,
        source: crate::org::io::DestinationSource,
        feed: &crate::config::FeedSource,
        policy: &crate::config::Org,
        cancel: &Cancellation,
    ) -> Result<Plan, String> {
        let target = target::resolve(&source, feed, policy, cancel)?;
        target::plan(source, target, feed, self, cancel)
    }
}

fn item_node(node: &xml::Node, space: &str, item: &str) -> bool {
    node.name.space == space && node.name.local == item
}

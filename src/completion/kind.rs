/// Presentation metadata shared by native declarations and LSP items.
/// It never participates in insertion text, matching or edit admission.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Kind {
    #[default]
    Text,
    Function,
    Constructor,
    Variable,
    Type,
    Module,
    Field,
    Keyword,
    Constant,
}

impl Kind {
    pub(crate) fn native(name: &str) -> Self {
        use Kind::*;
        const KINDS: &[(&[&str], Kind)] = &[
            (&["function", "method"], Function),
            (&["constructor"], Constructor),
            (&["binding", "variable", "parameter"], Variable),
            (&["type", "class", "interface", "enum", "trait"], Type),
            (&["module", "namespace", "table"], Module),
            (
                &[
                    "field",
                    "property",
                    "attribute",
                    "json-key",
                    "toml-key",
                    "property-key",
                ],
                Field,
            ),
            (&["keyword", "todo", "directive"], Keyword),
            (&["constant", "enumerator"], Constant),
        ];
        KINDS
            .iter()
            .find_map(|(names, kind)| names.contains(&name).then_some(*kind))
            .unwrap_or_default()
    }

    pub(crate) fn semantic(code: Option<u64>) -> (Self, &'static str) {
        use Kind::*;
        const KINDS: [(Kind, &str); 25] = [
            (Text, "text"),
            (Function, "method"),
            (Function, "function"),
            (Constructor, "constructor"),
            (Field, "field"),
            (Variable, "variable"),
            (Type, "class"),
            (Type, "interface"),
            (Module, "module"),
            (Field, "property"),
            (Type, "unit"),
            (Variable, "value"),
            (Type, "enum"),
            (Keyword, "keyword"),
            (Text, "snippet"),
            (Text, "color"),
            (Text, "file"),
            (Text, "reference"),
            (Module, "folder"),
            (Constant, "enum member"),
            (Constant, "constant"),
            (Type, "struct"),
            (Text, "event"),
            (Keyword, "operator"),
            (Type, "type parameter"),
        ];
        code.and_then(|code| code.checked_sub(1))
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| KINDS.get(index).copied())
            .unwrap_or((Text, ""))
    }

    pub(crate) fn indicator(self) -> char {
        match self {
            Self::Text => '·',
            Self::Function => 'ƒ',
            Self::Constructor => '↦',
            Self::Variable => 'v',
            Self::Type => 'T',
            Self::Module => 'M',
            Self::Field => '◆',
            Self::Keyword => 'k',
            Self::Constant => '#',
        }
    }
}

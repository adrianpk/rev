use super::Family;
use std::ffi::OsString;

/// A native command family has one label and one exact editable argv template.
struct Definition {
    label: &'static str,
    seed: &'static [&'static str],
}
impl Family {
    pub fn label(self) -> &'static str {
        self.creation_definition().label
    }
    pub fn seed(self) -> Vec<String> {
        self.creation_definition()
            .seed
            .iter()
            .map(|a| (*a).into())
            .collect()
    }
    fn creation_definition(self) -> Definition {
        match self {
            Self::Clone => Definition {
                label: "regular clone",
                seed: &["clone", "--", "", ""],
            },
            Self::CloneShallow => Definition {
                label: "shallow clone",
                seed: &["clone", "--depth=1", "--", "", ""],
            },
            Self::CloneBare => Definition {
                label: "bare clone",
                seed: &["clone", "--bare", "--", "", ""],
            },
            Self::CloneMirror => Definition {
                label: "mirror clone",
                seed: &["clone", "--mirror", "--", "", ""],
            },
            Self::Init => Definition {
                label: "repository init",
                seed: &["init", "--", ""],
            },
            family => family.submodule_definition(),
        }
    }
    fn submodule_definition(self) -> Definition {
        match self {
            Self::SubmoduleAdd => Definition {
                label: "submodule add",
                seed: &["submodule", "add", "--name", "", "--", "", ""],
            },
            Self::SubmoduleList => Definition {
                label: "submodule list",
                seed: &["submodule", "status", "--recursive"],
            },
            Self::SubmoduleUpdate => Definition {
                label: "submodule update",
                seed: &["submodule", "update", "--init", "--checkout", "--", ""],
            },
            Self::SubmoduleSync => Definition {
                label: "submodule sync",
                seed: &["submodule", "sync", "--", ""],
            },
            Self::SubmoduleDeinit => Definition {
                label: "submodule deinit",
                seed: &["submodule", "deinit", "--", ""],
            },
            family => family.subtree_definition(),
        }
    }
    fn subtree_definition(self) -> Definition {
        match self {
            Self::SubtreeAdd => Definition {
                label: "subtree add",
                seed: &["subtree", "add", "--prefix", "", "--", "", ""],
            },
            Self::SubtreeMerge => Definition {
                label: "subtree merge",
                seed: &["subtree", "merge", "--prefix", "", "--", ""],
            },
            Self::SubtreePull => Definition {
                label: "subtree pull",
                seed: &["subtree", "pull", "--prefix", "", "--", "", ""],
            },
            Self::SubtreePush => Definition {
                label: "subtree push",
                seed: &["subtree", "push", "--prefix", "", "--", "", ""],
            },
            Self::SubtreeSplit => Definition {
                label: "subtree split",
                seed: &["subtree", "split", "--prefix", "", "--", ""],
            },
            family => family.notes_definition(),
        }
    }
    fn notes_definition(self) -> Definition {
        match self {
            Self::NotesAdd => Definition {
                label: "notes add",
                seed: &["notes", "--ref", "", "add", "--message", "", "--", ""],
            },
            Self::NotesRemove => Definition {
                label: "notes remove",
                seed: &["notes", "--ref", "", "remove", "--", ""],
            },
            Self::NotesPrune => Definition {
                label: "notes prune",
                seed: &["notes", "--ref", "", "prune"],
            },
            family => family.sparse_definition(),
        }
    }
    fn sparse_definition(self) -> Definition {
        match self {
            Self::SparseEnable => Definition {
                label: "sparse checkout enable",
                seed: &["sparse-checkout", "init", "--cone", "--no-sparse-index"],
            },
            Self::SparseDisable => Definition {
                label: "sparse checkout disable",
                seed: &["sparse-checkout", "disable"],
            },
            Self::SparseReapply => Definition {
                label: "sparse checkout reapply",
                seed: &["sparse-checkout", "reapply"],
            },
            Self::SparseSet => Definition {
                label: "sparse checkout set",
                seed: &[
                    "sparse-checkout",
                    "set",
                    "--cone",
                    "--no-sparse-index",
                    "--skip-checks",
                    "--",
                    "",
                ],
            },
            Self::SparseAdd => Definition {
                label: "sparse checkout add",
                seed: &["sparse-checkout", "add", "--skip-checks", "--", ""],
            },
            family => family.output_definition(),
        }
    }
    fn output_definition(self) -> Definition {
        match self {
            Self::FormatPatch => Definition {
                label: "format patch output",
                seed: &[
                    "format-patch",
                    "--stdout",
                    "--binary",
                    "--full-index",
                    "--no-signature",
                    "--no-cover-letter",
                    "--no-notes",
                    "--no-ext-diff",
                    "--no-textconv",
                    "",
                    "--",
                ],
            },
            Self::SaveDiff => Definition {
                label: "saved diff output",
                seed: &[
                    "diff",
                    "--binary",
                    "--full-index",
                    "--no-ext-diff",
                    "--no-textconv",
                    "--no-color",
                    "",
                    "",
                    "--",
                ],
            },
            Self::RequestPull => Definition {
                label: "request pull output",
                seed: &["request-pull", "", "", ""],
            },
            family => family.patch_definition(),
        }
    }
    fn patch_definition(self) -> Definition {
        match self {
            Self::ApplyPatch => Definition {
                label: "plain patch application",
                seed: &[
                    "apply",
                    "--index",
                    "--binary",
                    "--whitespace=nowarn",
                    "--",
                    "",
                ],
            },
            Self::ApplyMailbox => Definition {
                label: "mailbox application",
                seed: &[
                    "am",
                    "--patch-format=mboxrd",
                    "--no-3way",
                    "--no-signoff",
                    "--no-message-id",
                    "--no-scissors",
                    "--no-keep-cr",
                    "--no-keep",
                    "--no-keep-non-patch",
                    "--utf8",
                    "--no-rerere-autoupdate",
                    "--whitespace=nowarn",
                    "--",
                    "",
                ],
            },
            family => family.direct_definition(),
        }
    }
    fn direct_definition(self) -> Definition {
        match self {
            Self::RunCommand => Definition {
                label: "direct Git command",
                seed: &[""],
            },
            Self::RunShellCommand => Definition {
                label: "explicit shell command",
                seed: &[""],
            },
            _ => unreachable!("command definition dispatched by its native family"),
        }
    }
    pub(in crate::git) fn validate(self, args: &[OsString]) -> Result<(), String> {
        if self.direct() {
            return self.validate_direct(args);
        }
        self.validate_template(args)?;
        if args
            .iter()
            .any(|a| a.is_empty() || a.as_encoded_bytes().contains(&0))
        {
            return Err("Fill every prepared argument with nonempty, non-NUL input".into());
        }
        Ok(())
    }
    fn validate_direct(self, args: &[OsString]) -> Result<(), String> {
        if args.is_empty()
            || args[0].is_empty()
            || args.iter().any(|a| a.as_encoded_bytes().contains(&0))
            || (self == Self::RunShellCommand && args.len() != 1)
        {
            return Err("Direct Git requires literal arguments; shell mode requires exactly one nonempty command string without NUL".into());
        }
        Ok(())
    }
    fn validate_template(self, args: &[OsString]) -> Result<(), String> {
        let expected = self.seed();
        let variadic = matches!(self, Self::SparseSet | Self::SparseAdd);
        if (if variadic {
            args.len() < expected.len()
        } else {
            args.len() != expected.len()
        }) || args
            .iter()
            .zip(&expected)
            .any(|(a, b)| !b.is_empty() && a != b.as_str())
        {
            return Err(
                "Keep the prepared family and its explicit option/parameter structure".into(),
            );
        }
        Ok(())
    }
}

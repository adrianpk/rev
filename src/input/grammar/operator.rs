use super::Character;
use crate::command::editing::{Action, Object, Operand};
use crate::command::navigation::Target;
use crate::command::{Binding, Command, Editing, Navigation};
use crate::input::{Input, Key, Outcome, Prefix};
use crossterm::event::KeyCode;

impl Input {
    pub(crate) fn expect_operator(&mut self, request: Editing) -> Outcome {
        self.clear_prefix();
        self.grammar.operator = Some(request);
        self.wait(Prefix::Sequence)
    }

    pub(crate) fn expect_replacement(&mut self, request: Editing) -> Outcome {
        self.clear_prefix();
        self.grammar.character = Some(Character::Editing(request));
        self.wait(Prefix::Sequence)
    }

    pub(crate) fn expect_visual_object(&mut self, request: Editing) -> Outcome {
        let strokes = self.grammar.strokes;
        self.clear_prefix();
        self.grammar.strokes = strokes;
        self.grammar.operator = Some(request);
        self.grammar.object = Some(request.action == Action::SelectAround);
        self.wait(Prefix::Sequence)
    }

    pub(super) fn editing_binding(&mut self, request: Editing) -> Result<Binding, Outcome> {
        if self.editing_state.visual().is_some() {
            if request.action.selects() {
                return Err(self.expect_visual_object(request));
            }
            if request.pending() {
                self.clear_prefix();
                return Ok(Binding::Command(Command::Edit(request)));
            }
        }
        if let Some(operator) = self.grammar.operator {
            let count = multiplied(operator.count, request.count)
                .ok_or_else(|| self.grammar_failure("Modal count multiplication exceeds u32"))?;
            if operator.action != request.action {
                return Err(self.grammar_failure("Expected an operator target"));
            }
            let mut complete = operator;
            complete.count = count;
            complete.operand = Some(Operand::Lines);
            self.clear_prefix();
            return Ok(Binding::Command(Command::Edit(complete)));
        }
        if request.pending() {
            let strokes = self.grammar.strokes;
            self.clear_prefix();
            self.grammar.strokes = strokes;
            self.grammar.operator = Some(request);
            return Err(self.wait(Prefix::Sequence));
        }
        if request.action == Action::Replace && request.character.is_none() {
            self.grammar.character = Some(Character::Editing(request));
            return Err(self.wait(Prefix::Sequence));
        }
        self.clear_prefix();
        Ok(Binding::Command(Command::Edit(request)))
    }

    pub(super) fn motion_binding(&mut self, mut request: Navigation) -> Result<Binding, Outcome> {
        if let Some(mut operator) = self.grammar.operator {
            request.count = multiplied(operator.count, request.count)
                .ok_or_else(|| self.grammar_failure("Modal count multiplication exceeds u32"))?;
            operator.count = request.count;
            if matches!(request.target, Target::Find(_)) {
                self.grammar.operator = Some(operator);
                return Err(self.expect_navigation(request));
            }
            operator.operand = Some(Operand::Motion(request));
            self.clear_prefix();
            return Ok(Binding::Command(Command::Edit(operator)));
        }
        if matches!(request.target, Target::Find(_)) {
            return Err(self.expect_navigation(request));
        }
        self.clear_prefix();
        Ok(Binding::Command(Command::Navigate(request)))
    }

    pub(super) fn object_key(&mut self, key: Key) -> Option<Outcome> {
        let operator = self.grammar.operator?;
        if let Some(around) = self.grammar.object {
            self.grammar.strokes += 1;
            if self.grammar.strokes > super::STROKES {
                return Some(self.grammar_failure("Modal input exceeds 16 strokes"));
            }
            let object = object(key, around);
            return Some(match object {
                Some(object) => self.complete_object(operator, object),
                None => self.grammar_failure("Unsupported text object"),
            });
        }
        if matches!(key.plain(), Some(KeyCode::Char('i' | 'a'))) {
            self.grammar.strokes += 1;
            if self.grammar.strokes > super::STROKES {
                return Some(self.grammar_failure("Modal input exceeds 16 strokes"));
            }
            self.grammar.object = Some(key.plain() == Some(KeyCode::Char('a')));
            return Some(self.wait(Prefix::Sequence));
        }
        None
    }

    fn complete_object(&mut self, mut request: Editing, object: Object) -> Outcome {
        let count = multiplied(request.count, self.grammar.count);
        self.clear_prefix();
        let Some(count) = count else {
            return Outcome::Message("Modal count multiplication exceeds u32");
        };
        request.count = count;
        request.operand = Some(Operand::Object(object));
        Outcome::Command(Command::Edit(request))
    }

    fn grammar_failure(&mut self, message: &'static str) -> Outcome {
        self.clear_prefix();
        self.suppress_repeat = true;
        Outcome::Message(message)
    }
}

fn multiplied(first: Option<u32>, second: Option<u32>) -> Option<Option<u32>> {
    if first.is_none() && second.is_none() {
        return Some(None);
    }
    first
        .unwrap_or(1)
        .checked_mul(second.unwrap_or(1))
        .map(Some)
}

fn object(key: Key, around: bool) -> Option<Object> {
    match key.plain()? {
        KeyCode::Char('w') => Some(Object::Word { big: false, around }),
        KeyCode::Char('W') => Some(Object::Word { big: true, around }),
        KeyCode::Char('p') => Some(Object::Paragraph { around }),
        KeyCode::Char(character) => delimited(character, around),
        _ => None,
    }
}

fn delimited(character: char, around: bool) -> Option<Object> {
    let open = match character {
        '(' | ')' => '(',
        '[' | ']' => '[',
        '{' | '}' => '{',
        '\'' | '"' | '`' => {
            return Some(Object::Quote {
                quote: character,
                around,
            });
        }
        _ => return None,
    };
    Some(Object::Pair { open, around })
}

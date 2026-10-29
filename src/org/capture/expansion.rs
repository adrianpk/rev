//! Bounded data-only expansion. Inserted source and answers are opaque text;
//! they are never scanned again as template instructions.
mod parse;
mod time;
use crate::{
    config::CaptureTemplate,
    coordinates::Byte,
    org::{Cancellation, time::CivilDateTime},
};

pub const EXPANSION_BYTES: usize = 1048576;
pub const VALUE_BYTES: usize = 262144;
#[derive(Clone, Debug)]
pub struct Context {
    pub now: CivilDateTime,
    pub initial: String,
    pub annotation: String,
    pub link: String,
    pub kill: String,
    pub user: Option<String>,
    pub clipboard: Option<String>,
}
impl Context {
    pub fn new(now: CivilDateTime) -> Self {
        Self {
            now,
            initial: String::new(),
            annotation: String::new(),
            link: String::new(),
            kill: String::new(),
            user: None,
            clipboard: None,
        }
    }
    fn validate(&self) -> Result<(), String> {
        for value in [&self.initial, &self.annotation, &self.link, &self.kill] {
            bounded(value, VALUE_BYTES)?;
        }
        for value in [&self.user, &self.clipboard].into_iter().flatten() {
            bounded(value, VALUE_BYTES)?;
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prompt {
    pub label: String,
    pub default: String,
    pub completions: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DraftText {
    pub text: String,
    pub cursor: Byte,
}
pub enum Step {
    Progress,
    Prompt(Prompt),
    Ready(DraftText),
}
#[derive(Clone)]
enum Token {
    Literal(String),
    Field(char),
    Prompt(Prompt),
    Format(String),
    Cursor,
}
#[derive(Clone, Copy)]
enum Answer {
    Text,
    User,
    Clipboard,
}
pub struct Expansion {
    tokens: Vec<Token>,
    context: Context,
    index: usize,
    output: String,
    line_start: usize,
    cursor: Option<usize>,
    pending: Option<(Prompt, Answer)>,
    prompts: usize,
    finished: bool,
}
impl Expansion {
    pub fn new(template: &CaptureTemplate, context: Context) -> Result<Self, String> {
        template.validate()?;
        context.validate()?;
        Ok(Self {
            tokens: parse::tokens(template.text())?,
            context,
            index: 0,
            output: String::new(),
            line_start: 0,
            cursor: None,
            pending: None,
            prompts: 0,
            finished: false,
        })
    }
    /// One finite quantum; native callers schedule Progress only while this
    /// expansion is active. Waiting for a human prompt creates no idle polling.
    pub fn advance(&mut self, cancel: &Cancellation) -> Result<Step, String> {
        if cancel.is_cancelled() {
            return Err("Capture expansion cancelled".into());
        }
        if self.finished {
            return Err("Capture expansion is already complete".into());
        }
        if let Some((prompt, _)) = &self.pending {
            return Ok(Step::Prompt(prompt.clone()));
        }
        for _ in 0..64 {
            if cancel.is_cancelled() {
                return Err("Capture expansion cancelled".into());
            }
            let Some(token) = self.tokens.get(self.index).cloned() else {
                self.finished = true;
                let cursor = Byte(self.cursor.unwrap_or(self.output.len()));
                return Ok(Step::Ready(DraftText {
                    text: std::mem::take(&mut self.output),
                    cursor,
                }));
            };
            if let Some(prompt) = self.token(token, cancel)? {
                return Ok(Step::Prompt(prompt));
            }
            self.index += 1;
        }
        Ok(Step::Progress)
    }
    pub fn answer(&mut self, value: &str) -> Result<(), String> {
        bounded(value, VALUE_BYTES)?;
        let Some((_, answer)) = self.pending.as_ref() else {
            return Err("No capture expansion prompt is pending".into());
        };
        match answer {
            Answer::Text => {
                self.append(value)?;
                self.index += 1;
            }
            Answer::User => self.context.user = Some(value.into()),
            Answer::Clipboard => self.context.clipboard = Some(value.into()),
        }
        self.pending = None;
        Ok(())
    }
    fn token(&mut self, token: Token, cancel: &Cancellation) -> Result<Option<Prompt>, String> {
        match token {
            Token::Literal(value) => self.append(&value)?,
            Token::Cursor => {
                self.cursor.get_or_insert(self.output.len());
            }
            Token::Format(format) => self.append(&time::format(self.context.now, &format)?)?,
            Token::Prompt(prompt) => return self.ask(prompt, Answer::Text),
            Token::Field('i') => self.initial(cancel)?,
            Token::Field(field) => return self.field(field),
        }
        Ok(None)
    }
    fn field(&mut self, field: char) -> Result<Option<Prompt>, String> {
        let value = match field {
            'a' => self.context.annotation.clone(),
            'l' => self.context.link.clone(),
            'c' => self.context.kill.clone(),
            'n' => match &self.context.user {
                Some(v) => v.clone(),
                None => return self.ask(simple("Capture user name: "), Answer::User),
            },
            'x' => match &self.context.clipboard {
                Some(v) => v.clone(),
                None => {
                    return self.ask(
                        simple("Clipboard text (explicit input): "),
                        Answer::Clipboard,
                    );
                }
            },
            _ => time::timestamp(self.context.now, field)?,
        };
        self.append(&value)?;
        Ok(None)
    }
    fn ask(&mut self, prompt: Prompt, answer: Answer) -> Result<Option<Prompt>, String> {
        if self.prompts == 64 {
            return Err("Capture expansion exceeds 64 prompts".into());
        }
        self.prompts += 1;
        self.pending = Some((prompt.clone(), answer));
        Ok(Some(prompt))
    }
    fn initial(&mut self, cancel: &Cancellation) -> Result<(), String> {
        let prefix = self.output[self.line_start..].to_owned();
        let value = self.context.initial.clone();
        for (index, line) in value.split_inclusive('\n').enumerate() {
            if cancel.is_cancelled() {
                return Err("Capture expansion cancelled".into());
            }
            if index > 0 {
                self.append(&prefix)?;
            }
            self.append(line)?;
        }
        Ok(())
    }
    fn append(&mut self, value: &str) -> Result<(), String> {
        if self.output.len().saturating_add(value.len()) > EXPANSION_BYTES {
            return Err("Capture expansion exceeds 1 MiB".into());
        }
        if let Some(last) = value.rfind('\n') {
            self.line_start = self.output.len() + last + 1;
        }
        self.output.push_str(value);
        Ok(())
    }
}
fn simple(label: &str) -> Prompt {
    Prompt {
        label: label.into(),
        default: String::new(),
        completions: Vec::new(),
    }
}
fn bounded(value: &str, maximum: usize) -> Result<(), String> {
    if value.len() > maximum || value.contains('\0') {
        return Err(format!(
            "Capture value exceeds {maximum} bytes or contains NUL"
        ));
    }
    Ok(())
}

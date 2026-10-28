use super::*;

struct Layer {
    base: Option<usize>,
    additions: Vec<usize>,
}
impl Properties {
    pub(super) fn resolve(
        &self,
        buffer: &Buffer,
        entry: usize,
        key: &str,
        inherit: &[String],
    ) -> Result<PropertyValue, String> {
        let key = key.trim_end_matches('+');
        let selected = inherit.iter().any(|name| name.eq_ignore_ascii_case(key));
        let mut chain = Vec::new();
        let mut owner = Some(entry);
        while let Some(index) = owner {
            let layer = self.layer(index, key)?;
            let stop = layer.base.is_some() || !selected;
            chain.push(layer);
            if stop {
                break;
            }
            owner = self.entries[index].parent;
        }
        let mut value = PropertyValue {
            text: String::new(),
            records: Vec::new(),
            undefined: false,
        };
        for layer in chain.into_iter().rev() {
            self.combine(buffer, layer, &mut value)?;
        }
        Ok(value)
    }
    fn layer(&self, owner: usize, key: &str) -> Result<Layer, String> {
        let entry = &self.entries[owner];
        let mut layer = Layer {
            base: None,
            additions: Vec::new(),
        };
        // File defaults precede file-drawer ownership regardless of source order.
        for directive in [true, false] {
            for index in &entry.records {
                let record = &self.records[*index];
                if record.key == key && record.directive == directive {
                    self.admit(&mut layer, *index)?;
                }
            }
        }
        Ok(layer)
    }
    fn admit(&self, layer: &mut Layer, index: usize) -> Result<(), String> {
        let record = &self.records[index];
        if record.additive {
            layer.additions.push(index);
            return Ok(());
        }
        if let Some(old) = layer.base
            && !self.records[old].directive
            && !record.directive
        {
            return Err("Duplicate local Org property; inspect source before resolving".into());
        }
        // A drawer base overrides file declarations. Repeated file base
        // declarations reset their previous additions, as ordered data defaults.
        layer.base = Some(index);
        if record.directive {
            layer.additions.clear();
        } else {
            layer.additions.retain(|i| !self.records[*i].directive);
        }
        Ok(())
    }
    fn combine(
        &self,
        buffer: &Buffer,
        layer: Layer,
        value: &mut PropertyValue,
    ) -> Result<(), String> {
        if let Some(base) = layer.base {
            value.text = text(buffer, self.records[base].value.clone())?;
            value.undefined = value.text == "nil";
            value.records = vec![base];
        }
        for index in layer.additions {
            let addition = text(buffer, self.records[index].value.clone())?;
            if !value.text.is_empty() && !addition.is_empty() {
                value.text.push(' ');
            }
            value.text.push_str(&addition);
            value.records.push(index);
        }
        value.undefined = value.text == "nil";
        Ok(())
    }
}

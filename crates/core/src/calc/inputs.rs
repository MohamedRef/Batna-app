//! Named inputs and the errors a calculation can return.

use super::*;

/// Named inputs for one calculation.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Inputs(pub BTreeMap<String, CalcValue>);

impl Inputs {
    pub fn new() -> Self {
        Self::default()
    }

    /// Content inputs for one calculator. A key may drop its unit suffix
    /// (`alt` for `alt_u_l`, `bilirubin` for `bilirubin_mg_dl`); it is renamed
    /// to the field's key. Keys that match no field stay as written, so
    /// [`unknown_inputs`] can report them.
    pub fn for_spec(spec: &CalcSpec, raw: &BTreeMap<String, CalcValue>) -> Self {
        let rename = |k: &String| {
            spec.fields
                .iter()
                .find(|f| f.key == k || base_key(f.key) == k)
                .map_or_else(|| k.clone(), |f| f.key.to_owned())
        };
        Self(raw.iter().map(|(k, v)| (rename(k), v.clone())).collect())
    }

    pub fn num(mut self, key: &str, v: f64) -> Self {
        self.0.insert(key.to_owned(), CalcValue::Number(v));
        self
    }

    pub fn flag(mut self, key: &str, v: bool) -> Self {
        self.0.insert(key.to_owned(), CalcValue::Bool(v));
        self
    }

    pub fn choice(mut self, key: &str, v: &str) -> Self {
        self.0.insert(key.to_owned(), CalcValue::Text(v.to_owned()));
        self
    }

    /// A required finite number.
    pub fn get_num(&self, key: &'static str) -> Result<f64, CalcError> {
        self.opt_num(key)?.ok_or(CalcError::Missing(key))
    }

    /// An optional finite number. Integers written as choices ("2") are
    /// accepted so form text fields can pass strings straight through.
    pub fn opt_num(&self, key: &'static str) -> Result<Option<f64>, CalcError> {
        let v = match self.0.get(key) {
            None => return Ok(None),
            Some(CalcValue::Number(n)) => *n,
            Some(CalcValue::Text(s)) if s.trim().is_empty() => return Ok(None),
            Some(CalcValue::Text(s)) => s
                .trim()
                .parse::<f64>()
                .map_err(|_| CalcError::Invalid { field: key, reason: "not a number" })?,
            Some(CalcValue::Bool(_)) => {
                return Err(CalcError::Invalid { field: key, reason: "expected a number" });
            }
        };
        if v.is_finite() {
            Ok(Some(v))
        } else {
            Err(CalcError::Invalid { field: key, reason: "not a finite number" })
        }
    }

    /// A positive number, as needed under a logarithm or a division.
    pub fn get_pos(&self, key: &'static str) -> Result<f64, CalcError> {
        let v = self.get_num(key)?;
        if v > 0.0 { Ok(v) } else { Err(CalcError::Invalid { field: key, reason: "must be above zero" }) }
    }

    /// A yes/no input; missing means no.
    pub fn get_flag(&self, key: &'static str) -> Result<bool, CalcError> {
        match self.0.get(key) {
            None => Ok(false),
            Some(CalcValue::Bool(b)) => Ok(*b),
            Some(CalcValue::Text(s)) => match s.as_str() {
                "yes" | "true" => Ok(true),
                "no" | "false" | "" => Ok(false),
                _ => Err(CalcError::Invalid { field: key, reason: "expected yes or no" }),
            },
            Some(CalcValue::Number(_)) => {
                Err(CalcError::Invalid { field: key, reason: "expected yes or no" })
            }
        }
    }

    /// A required choice, matched against each option's id and its aliases
    /// (content writers say "mild" as often as "mild_moderate"). Returns the id.
    pub fn get_option(
        &self,
        key: &'static str,
        options: &[(&'static str, &[&str])],
    ) -> Result<&'static str, CalcError> {
        let text = match self.0.get(key) {
            None => return Err(CalcError::Missing(key)),
            Some(CalcValue::Text(s)) => s.trim().to_ascii_lowercase().replace([' ', '-'], "_"),
            Some(_) => return Err(CalcError::Invalid { field: key, reason: "not one of the options" }),
        };
        options
            .iter()
            .find(|(id, aliases)| *id == text || aliases.contains(&text.as_str()))
            .map(|(id, _)| *id)
            .ok_or(CalcError::Invalid { field: key, reason: "not one of the options" })
    }

    /// A required choice that must be one of `allowed`.
    pub fn get_choice(&self, key: &'static str, allowed: &[&str]) -> Result<&str, CalcError> {
        match self.0.get(key) {
            None => Err(CalcError::Missing(key)),
            Some(CalcValue::Text(s)) if allowed.contains(&s.as_str()) => Ok(s.as_str()),
            Some(_) => Err(CalcError::Invalid { field: key, reason: "not one of the options" }),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CalcError {
    Unknown(String),
    Missing(&'static str),
    Invalid { field: &'static str, reason: &'static str },
}

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown(id) => write!(f, "unknown calculator `{id}`"),
            Self::Missing(k) => write!(f, "missing input `{k}`"),
            Self::Invalid { field, reason } => write!(f, "input `{field}`: {reason}"),
        }
    }
}

impl std::error::Error for CalcError {}

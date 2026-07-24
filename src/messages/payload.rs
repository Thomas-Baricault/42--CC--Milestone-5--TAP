#[derive(Clone)]
pub enum PayloadKind {
    String(String),
    KeyValue {
        key: String,
        value: String
    },
    Json(serde_json::Value),
}

pub trait PayloadKeyword {
    fn set_str(&mut self, s: &str) -> Result<(), std::io::Error>;
}

impl<T: std::str::FromStr<Err = std::io::Error>> PayloadKeyword for T {
    fn set_str(&mut self, s: &str) -> Result<(), std::io::Error> {
        match T::from_str(s) {
            Ok(v) => {
                *self = v;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

pub trait PayloadJson {
    fn set_json(&mut self, value: &serde_json::Value) -> Result<(), std::io::Error>;
}

impl<T: serde::de::DeserializeOwned> PayloadJson for T {
    fn set_json(&mut self, value: &serde_json::Value) -> Result<(), std::io::Error> {
        match serde_json::from_value(value.clone()) {
            Ok(v) => {
                *self = v;
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }
}

pub enum PayloadExtractor<'a> {
    String(&'a mut String),
    Keyword(&'a mut dyn PayloadKeyword),
    KeyValue {
        key: &'a mut String,
        value: &'a mut String,
    },
    Json(&'a mut dyn PayloadJson),
}

impl PayloadKind {
    pub fn new_json<T: serde::Serialize>(data: &T) -> Self {
        Self::Json(serde_json::to_value(data).unwrap())
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    pub fn is_key_value(&self) -> bool {
        matches!(self, Self::KeyValue { .. })
    }

    pub fn is_json(&self) -> bool {
        matches!(self, Self::Json(_))
    }

    pub fn string_from_str(s: &str) -> Result<Self, std::io::Error> {
        Ok(Self::String(Self::unescape(s)))
    }

    pub fn key_value_from_str(s: &str) -> Result<Self, std::io::Error> {
        let parts: Vec<&str> = s.split("=").collect();
        if parts.len() != 2 {
            return Err(crate::utils::invalid_input("invalid key value"));
        }
        Ok(Self::KeyValue {
            key: Self::unescape(parts[0]),
            value: Self::unescape(parts[1])
        })
    }

    pub fn json_from_str(s: &str) -> Result<Self, std::io::Error> {
        match serde_json::from_str(s) {
            Ok(v) => Ok(Self::Json(v)),
            Err(_) => Err(crate::utils::invalid_input("invalid json")),
        }
    }

    fn escape(s: &str) -> String {
        let mut res = String::new();
        for char in s.chars() {
            if matches!(char, '\\' | ' ' | '=' | '{' | '}' | '[' | ']') {
                res.push('\\');
            }
            res.push(char);
        }
        res
    }

    fn unescape(s: &str) -> String {
        let mut res = String::new();
        let mut escape = false;
        for char in s.chars() {
            if !escape && char == '\\' {
                escape = true;
            } else {
                escape = false;
                res.push(char);
            }
        }
        res
    }
}

impl std::fmt::Display for PayloadKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(
                f,
                "{}",
                Self::escape(s),
            ),
            Self::KeyValue {
                key,
                value,
            } => write!(
                f,
                "{}={}",
                Self::escape(key),
                Self::escape(value),
            ),
            Self::Json(json) => write!(
                f,
                "{}",
                serde_json::to_string(&json)
                    .map_err(|_| std::fmt::Error)?,
            ),
        }
    }
}

#[derive(Clone, Default)]
pub struct Payload {
    pub args: Vec<PayloadKind>,
}

impl Payload {
    pub fn new(args: &[PayloadKind]) -> Self {
        Self {
            args: args.to_vec(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    pub fn extract(&self, dest: &mut [PayloadExtractor<'_>]) -> Result<(), std::io::Error> {
        if dest.len() != self.args.len() {
            return Err(std::io::Error::other(format!(
                "invalid number of arguments: {} expected, got {}",
                self.args.len(),
                dest.len(),
            )));
        }
        for (i, (src, dest)) in self.args.iter().zip(dest.iter_mut()).enumerate() {
            match (src, dest) {
                (
                    PayloadKind::String(src),
                    PayloadExtractor::String(dest),
                ) => {
                    if dest.is_empty() {
                        **dest = src.clone();
                    } else if src != *dest {
                        return Err(std::io::Error::other(format!(
                            "invalid argument {}: '{}' expected, got '{}'",
                            i + 1,
                            dest,
                            src,
                        )));
                    }
                }
                (
                    PayloadKind::String(src),
                    PayloadExtractor::Keyword(dest),
                ) => {
                    if let Err(e) = dest.set_str(src) {
                        return Err(std::io::Error::other(format!(
                            "invalid keyword argument {}: {}",
                            i + 1,
                            e,
                        )));
                    }
                }
                (
                    PayloadKind::KeyValue {
                        key: src_key,
                        value: src_value
                    },
                    PayloadExtractor::KeyValue {
                        key: dest_key,
                        value: dest_value
                    },
                ) => {
                    if !dest_key.is_empty() && src_key != *dest_key {
                        return Err(std::io::Error::other(format!(
                            "invalid argument key {}: '{}' expected, got '{}'",
                            i + 1,
                            dest_key,
                            src_key,
                        )));
                    }
                    if !dest_value.is_empty() && src_value != *dest_value {
                        return Err(std::io::Error::other(format!(
                            "invalid argument value {}: '{}' expected, got '{}'",
                            i + 1,
                            dest_value,
                            src_value,
                        )));
                    }
                    **dest_key = src_key.clone();
                    **dest_value = src_value.clone();
                }
                (
                    PayloadKind::Json(src),
                    PayloadExtractor::Json(dest),
                ) => {
                    if let Err(e) = dest.set_json(src) {
                        return Err(std::io::Error::other(format!(
                            "invalid json argument {}: {}",
                            i + 1,
                            e,
                        )));
                    }
                }
                _ => return Err(std::io::Error::other(format!(
                    "argument {} has invalid type",
                    i + 1,
                ))),
            }
        }
        Ok(())
    }
}

impl std::str::FromStr for Payload {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut payload = Self { args: Vec::new() };
        let mut escaped = false;
        let mut i: usize = 0;
        let chars: Vec<char> = s.chars().collect();
        while i < chars.len() {
            let mut j = i;
            let mut t: i8 = 0;
            let mut level = 0;
            let mut in_string = false;
            while j < chars.len() {
                if !escaped {
                    if chars[j] == '\\' && (t != 2 || in_string) {
                        escaped = true;
                    } else if chars[j] == ' ' && (t != 2 || level == 0) {
                        break;
                    } else if chars[j] == '=' {
                        t += 1;
                    } else if matches!(chars[j], '{' | '}' | '[' | ']') {
                        t = 2;
                        if !in_string {
                            if matches!(chars[j], '{' | '[') {
                                level += 1;
                            } else {
                                level -= 1;
                            }
                        }
                    } else if t == 2 && chars[j] == '"' {
                        in_string = !in_string;
                    }
                }
                j += 1;
            }
            let arg: String = chars[i..j].iter().collect();
            payload.args.push(match match t {
                1 => PayloadKind::key_value_from_str(&arg),
                2 => PayloadKind::json_from_str(&arg),
                _ => PayloadKind::string_from_str(&arg),
            } {
                Ok(v) => v,
                Err(_) => return Err(crate::utils::invalid_input(&format!("invalid payload '{s}'"))),
            });
            i = j + 1;
        }
        Ok(payload)
    }
}

impl std::fmt::Display for Payload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut v = vec![];
        for arg in &self.args {
            v.push(arg.to_string());
        }
        write!(f, "{}", v.join(" "))
    }
}

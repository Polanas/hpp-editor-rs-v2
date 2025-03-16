use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Russian,
}

pub trait Translatable {
    fn translate_key(&self) -> &str;
}

impl<T: AsRef<str>> Translatable for T {
    fn translate_key(&self) -> &str {
        self.as_ref()
    }
}

impl Translatable for str {
    fn translate_key(&self) -> &str {
        self
    }
}

#[derive(Debug)]
pub struct UiText {
    language: Language,
    data: HashMap<String, HashMap<String, String>>,
}

impl UiText {
    pub fn new(language: Language, json: &str) -> Self {
        let json = serde_json::from_str(json).unwrap();
        Self {
            language,
            data: json,
        }
    }
    pub fn get<T: Translatable + ?Sized>(&self, translatable: &T) -> &str {
        let key = translatable.translate_key();
        match self.language {
            Language::English => self.data["en"].get(key).unwrap_or_else(|| {
                panic!("could not find a key for: en, {0}", key);
            }),
            Language::Russian => self.data["ru"].get(key).unwrap_or_else(|| {
                panic!("could not find a key for: ru, {0}", key);
            }),
        }
    }

    pub fn language(&self) -> Language {
        self.language
    }
}

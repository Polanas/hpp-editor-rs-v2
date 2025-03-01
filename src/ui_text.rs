use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
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
    pub language: Language,
    pub data: HashMap<String, HashMap<String, String>>,
}

impl UiText {
    pub fn new(language: Language, json: &str) -> Self {
        let json = serde_json::from_str(json).unwrap();
        Self {
            language,
            data: json,
        }
    }
    pub fn get<T: Translatable + ?Sized>(&self, translatable: &T) -> String {
        let key = translatable.translate_key();
        match self.language {
            Language::English => self.data["en"]
                .get(key)
                .unwrap_or_else(|| {
                    panic!("could not find a key for: en, {0}", key);
                })
                .clone(),
            Language::Russian => self.data["ru"]
                .get(key)
                .unwrap_or_else(|| {
                    panic!("could not find a key for: ru, {0}", key);
                })
                .clone(),
        }
    }
}

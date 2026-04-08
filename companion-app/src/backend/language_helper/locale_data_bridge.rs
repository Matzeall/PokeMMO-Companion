use crate::backend::{
    locales::{Locale, LocaleSubsystem, LocalizedText},
    search::{
        data_bridge::DataBridge,
        search_update::{UpdateRequest, UpdateRequestTracker},
    },
};
use std::{mem, rc::Rc, sync::Arc};

pub struct LocaleDataBridge {
    update_tracker: UpdateRequestTracker,

    locale_subsystem: Rc<LocaleSubsystem>,

    // search configuration
    cur_search_prompt: String,
    search_locale: String,
}

impl LocaleDataBridge {
    pub fn new(locale_subsystem: Rc<LocaleSubsystem>) -> Self {
        Self {
            update_tracker: UpdateRequestTracker::default(),
            locale_subsystem,
            cur_search_prompt: "".into(),
            search_locale: "".into(),
        }
    }

    pub fn get_search_prompt(&self) -> String {
        self.cur_search_prompt.clone()
    }

    pub fn set_search_prompt(&mut self, search_prompt: String) {
        if search_prompt == self.cur_search_prompt {
            return;
        }

        // update cur_search_prompt and save old prompt without cloning
        let old_prompt = mem::replace(&mut self.cur_search_prompt, search_prompt);

        // request incremental update only
        if self.cur_search_prompt.starts_with(&old_prompt) {
            self.update_tracker.request(UpdateRequest::Incremental);
            return;
        }

        self.update_tracker.request(UpdateRequest::Full);
    }

    pub fn set_search_locale(&mut self, locale_key: String) {
        self.search_locale = locale_key;
        self.update_tracker.request(UpdateRequest::Full);
    }
}

impl DataBridge for LocaleDataBridge {
    type Item = LocalizedText;

    fn get_all_potential_matches(&self) -> Vec<usize> {
        self.locale_subsystem
            .with_locale(&self.search_locale, |locale| {
                locale
                    .monsters
                    .values()
                    .cloned()
                    .chain(locale.moves.values().cloned())
                    .chain(locale.locations.values().cloned())
                    .chain(locale.items.values().cloned())
                    .collect()
            })
            .unwrap_or_default() // empty index if not found
    }

    fn filter_matches(&self, cur_matches: Vec<usize>) -> Vec<usize> {
        let mut search_list = cur_matches;
        let normalized_prompt_tokens: Vec<String> = self
            .cur_search_prompt
            .split_whitespace()
            .map(normalize_string)
            .collect();

        // custom whitespace-separated-tokens based filter function for QoL & performance
        // also uses a char table to make e == è etc. (the same for many other letters)
        // also checks starts_with against any word in multi-word names and multi-word prompts
        let filter_search_list = |locale: &Locale, search_list: &mut Vec<usize>| {
            search_list.retain(|index| match locale.localized_texts.get(*index) {
                Some(text) => {
                    let target_text_normalized = normalize_string(&text.text);
                    let target_words: Vec<&str> =
                        target_text_normalized.split_whitespace().collect();

                    // all prompt words must have some starts_with match in the target text
                    // i.e. "Punch Fir" matches "Fire Punch", but not "Ice Punch"
                    normalized_prompt_tokens
                        .iter()
                        .all(|token| target_words.iter().any(|word| word.starts_with(token)))
                }
                None => false,
            });
        };

        let found = self
            .locale_subsystem
            .with_locale(&self.search_locale, |locale| {
                filter_search_list(locale, &mut search_list);
            })
            .is_some();

        // if for ANY reason no "search-locale" can't be found, search_index is emptied
        if !found {
            search_list.clear();
        }

        search_list
    }

    fn get_items(&self, indices: &[usize]) -> Vec<Arc<LocalizedText>> {
        self.locale_subsystem
            .with_locale(&self.search_locale, |locale| {
                indices
                    .iter()
                    .filter_map(|i| locale.localized_texts.get(*i).cloned())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default() // empty vec
    }

    fn update_request(&mut self) -> &mut UpdateRequestTracker {
        &mut self.update_tracker
    }
}

/// using this makes it easier to search for certain things in unfirmiliar languages or the damn Pokè
fn normalize_char(c: char) -> String {
    match c.to_ascii_lowercase() {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => "a".to_string(),
        'è' | 'é' | 'ê' | 'ë' => "e".to_string(),
        'ì' | 'í' | 'î' | 'ï' => "i".to_string(),
        'ò' | 'ó' | 'ô' | 'ö' | 'õ' => "o".to_string(),
        'ù' | 'ú' | 'û' | 'ü' => "u".to_string(),
        'ñ' => "n".to_string(),
        'ç' => "c".to_string(),
        'ß' => "ss".to_string(),
        _ => c.to_ascii_lowercase().to_string(),
    }
}

fn normalize_string(s: &str) -> String {
    s.chars().map(normalize_char).collect()
}

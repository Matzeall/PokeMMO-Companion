use super::super::locales::TextCategory;
use super::super::search::search_index::SearchIndex;
use crate::backend::language_helper::locale_data_bridge::LocaleDataBridge;
use crate::backend::locales::LocaleSubsystem;
use crate::backend::search::data_bridge::DataBridge;
use crate::backend::search::search_update::UpdateRequest;
use std::rc::Rc;

pub struct LanguageHelperSubsystem {
    pub locale_subsystem: Rc<LocaleSubsystem>,
    locale_init_counter: usize, // used to compare against locale_subs counter -> update?
    search_index: SearchIndex<LocaleDataBridge>,
    // might or might not be valid locale keys at any given time
    from_locale: String,
    to_locale: String,
}

impl LanguageHelperSubsystem {
    pub fn new(locale_subsystem: Rc<LocaleSubsystem>) -> LanguageHelperSubsystem {
        let search_data_bridge = LocaleDataBridge::new(locale_subsystem.clone());

        Self {
            locale_subsystem,
            locale_init_counter: 0,
            search_index: SearchIndex::new(search_data_bridge),
            from_locale: String::new(),
            to_locale: String::new(),
        }
    }

    pub fn update_subsystem(&mut self) {
        // check if locale subsystem was re-initialized
        let counter_guard = self.locale_subsystem.init_counter.read().unwrap();
        if self.locale_init_counter < *counter_guard {
            println!("LanguageHelper - locale update detected -> triggering full search update");
            self.search_index
                .data_bridge
                .update_request()
                .request(UpdateRequest::Full);
            self.locale_init_counter = *counter_guard;
        }

        // pass tick to search_index, which occasionally computes new results
        self.search_index.update_search_index();
    }

    pub fn get_search_prompt(&self) -> String {
        self.search_index.data_bridge.get_search_prompt()
    }

    pub fn set_search_prompt(&mut self, search_prompt: String) {
        self.search_index
            .data_bridge
            .set_search_prompt(search_prompt);
    }

    pub fn get_translation_target_locale(&self) -> &String {
        &self.to_locale
    }

    pub fn get_translation_source_locale(&self) -> &String {
        &self.from_locale
    }

    pub fn set_translation_target_locale(&mut self, locale_key: String) {
        if self.to_locale == locale_key {
            return;
        }

        println!("LanguageHelper - set translation target locale to: {locale_key})");
        self.to_locale = locale_key;
    }

    pub fn set_translation_source_locale(&mut self, locale_key: String) {
        if self.from_locale == locale_key {
            return;
        }

        self.from_locale = locale_key.clone();
        self.search_index
            .data_bridge
            .set_search_locale(locale_key.clone());

        println!("LanguageHelper - set translation source locale to: {locale_key})");
    }

    pub fn swap_translation_locales(&mut self) {
        if self.to_locale == self.from_locale {
            return;
        }
        println!("LanguageHelper - swapping translation locales");

        let cur_source = self.from_locale.clone();
        let cur_target = self.to_locale.clone();
        self.set_translation_target_locale(cur_source);
        self.set_translation_source_locale(cur_target);
    }

    pub fn get_translation_pairs_for_search(&self) -> Vec<(TextCategory, String, String)> {
        let mut source_texts = Vec::new();
        let guard = self.locale_subsystem.data.read().unwrap();

        if let Some(data) = &*guard
            && let Some(target_locale) = data.locales.get(&self.to_locale)
        {
            for text in self.search_index.get_search_results() {
                let translation: String = target_locale.find_localized_text(&text.key);
                source_texts.push((text.category, text.text.clone(), translation));
            }
        };

        source_texts
    }
}

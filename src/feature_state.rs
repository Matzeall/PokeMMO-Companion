use std::collections::HashMap;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

// implements default Equals & Hash functions so a HashSet<Feature> can be checked for containment.
// Debug/Display is for printing or str::fmt the enum value name.
// Clone for passing it without a reference to function without trasfering ownership (by copying it)
// EnumIter is from the strum & strum_macros crate, allows iterating over all enum values.
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, EnumIter)]
pub enum Feature {
    Notes,
    Ressources,
    TypeMatrix,
    BreedingCalculator,
}

pub struct FeatureSubsystem {
    active_feature_windows: HashMap<Feature, bool>,
}

impl FeatureSubsystem {
    pub fn new() -> Self {
        // init map with all feature values + false
        let active_feature_windows = Feature::iter().map(|f| (f, false)).collect();
        Self {
            active_feature_windows,
        }
    }

    pub fn set_feature_active(&mut self, feature: Feature, enabled: bool) {
        self.active_feature_windows.insert(feature, enabled);
        println!("set {feature:#?} to {enabled}");
    }

    pub fn is_feature_active(&self, feature: Feature) -> bool {
        self.active_feature_windows
            .get(&feature)
            .cloned()
            .unwrap_or(false)
    }

    pub fn get_feature_active_mut_ref(&mut self, feature: Feature) -> &mut bool {
        self.active_feature_windows
            .get_mut(&feature)
            .expect("every feature should be contained in the map after app init")
    }
}

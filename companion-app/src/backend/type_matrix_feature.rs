use strum_macros::EnumIter;

/// TYPE_EFFECTIVENESS\[atk\]\[def\] = damage multiplier
#[rustfmt::skip]
pub const TYPE_EFFECTIVENESS: [[f32; 17]; 17] = [
    // Def:   N    F    Fl   P    G    R    B    Gh   St   Fi   Wa   Gr   El   Ps   I    D    Dk
    /*N*/ [ 1.0, 1.0, 1.0, 1.0, 1.0, 0.5, 1.0, 0.0, 0.5, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0 ],
    /*F*/ [ 2.0, 1.0, 0.5, 0.5, 1.0, 2.0, 0.5, 0.0, 2.0, 1.0, 1.0, 1.0, 1.0, 0.5, 2.0, 1.0, 2.0 ],
    /*Fl*/[ 1.0, 2.0, 1.0, 1.0, 1.0, 0.5, 2.0, 1.0, 0.5, 1.0, 1.0, 2.0, 0.5, 1.0, 1.0, 1.0, 1.0 ],
    /*P*/ [ 1.0, 1.0, 1.0, 0.5, 0.5, 0.5, 1.0, 0.5, 0.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0 ],
    /*G*/ [ 1.0, 1.0, 0.0, 2.0, 1.0, 2.0, 0.5, 1.0, 2.0, 2.0, 1.0, 0.5, 2.0, 1.0, 1.0, 1.0, 1.0 ],
    /*R*/ [ 1.0, 0.5, 2.0, 1.0, 0.5, 1.0, 2.0, 1.0, 0.5, 2.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0 ],
    /*B*/ [ 1.0, 0.5, 0.5, 0.5, 1.0, 1.0, 1.0, 0.5, 0.5, 1.0, 1.0, 2.0, 1.0, 2.0, 1.0, 1.0, 2.0 ],
    /*Gh*/[ 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 0.5 ],
    /*St*/[ 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 0.5, 0.5, 0.5, 1.0, 0.5, 1.0, 2.0, 1.0, 1.0 ],
    /*Fi*/[ 1.0, 1.0, 1.0, 1.0, 0.5, 2.0, 2.0, 1.0, 2.0, 0.5, 0.5, 2.0, 1.0, 1.0, 2.0, 0.5, 1.0 ],
    /*Wa*/[ 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 1.0, 1.0, 1.0, 2.0, 0.5, 0.5, 1.0, 1.0, 1.0, 0.5, 1.0 ],
    /*Gr*/[ 1.0, 1.0, 0.5, 0.5, 2.0, 2.0, 0.5, 1.0, 0.5, 0.5, 2.0, 0.5, 1.0, 1.0, 1.0, 0.5, 1.0 ],
    /*El*/[ 1.0, 1.0, 2.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 0.5, 0.5, 1.0, 1.0, 0.5, 1.0 ],
    /*Ps*/[ 1.0, 2.0, 1.0, 2.0, 1.0, 1.0, 1.0, 1.0, 0.5, 1.0, 1.0, 1.0, 1.0, 0.5, 1.0, 1.0, 0.0 ],
    /*I */[ 1.0, 1.0, 2.0, 1.0, 2.0, 1.0, 1.0, 1.0, 0.5, 0.5, 0.5, 2.0, 1.0, 1.0, 0.5, 2.0, 1.0 ],
    /*D */[ 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.5, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0 ],
    /*Dk*/[ 1.0, 0.5, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 0.5 ],
];

#[derive(Debug, Copy, Clone, EnumIter)]
pub enum PokemonType {
    Normal = 0,
    Fighting = 1,
    Flying = 2,
    Poison = 3,
    Ground = 4,
    Rock = 5,
    Bug = 6,
    Ghost = 7,
    Steel = 8,
    Fire = 9,
    Water = 10,
    Grass = 11,
    Electric = 12,
    Psychic = 13,
    Ice = 14,
    Dragon = 15,
    Dark = 16,
}

impl PokemonType {
    pub fn get_debug_name(&self) -> String {
        format!("{:?}", self)
    }
}

pub fn attack_effectiveness_single(
    attacking_type: impl Into<PokemonType>,
    defending_type: impl Into<PokemonType>,
) -> f32 {
    let atk_type: PokemonType = attacking_type.into();
    let def_type: PokemonType = defending_type.into();
    TYPE_EFFECTIVENESS[atk_type as usize][def_type as usize]
}

#[allow(dead_code)]
pub fn attack_effectiveness_double(
    attacking_type: impl Into<PokemonType>,
    defending_types: [Option<impl Into<PokemonType>>; 2],
) -> f32 {
    let atk_type: PokemonType = attacking_type.into();
    let [opt_def_1, opt_def_2] = defending_types;
    let def_type_1: Option<PokemonType> = opt_def_1.map(|t| t.into());
    let def_type_2: Option<PokemonType> = opt_def_2.map(|t| t.into());

    def_type_1.map_or(1., |def| attack_effectiveness_single(atk_type, def))
        * def_type_2.map_or(1., |def| attack_effectiveness_single(atk_type, def))
}

#[cfg(test)]
mod test {
    use crate::backend::type_matrix_feature::{
        PokemonType, attack_effectiveness_double, attack_effectiveness_single,
    };

    #[test]
    fn test_some_types() {
        let mut effectivness;

        effectivness = attack_effectiveness_single(PokemonType::Fighting, PokemonType::Normal);
        assert_eq!(effectivness, 2.);

        effectivness = attack_effectiveness_single(PokemonType::Water, PokemonType::Normal);
        assert_eq!(effectivness, 1.);

        effectivness = attack_effectiveness_single(PokemonType::Normal, PokemonType::Normal);
        assert_eq!(effectivness, 1.);

        effectivness = attack_effectiveness_single(PokemonType::Fire, PokemonType::Grass);
        assert_eq!(effectivness, 2.);

        effectivness = attack_effectiveness_double(
            PokemonType::Water,
            [Some(PokemonType::Ground), Some(PokemonType::Dragon)],
        );
        assert_eq!(effectivness, 1.);
    }
}

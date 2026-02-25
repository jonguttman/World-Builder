use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CodexCategory {
    Species,
    BodyPlan,
    Biome,
    PlanetarySystem,
    EvolutionaryEvent,
    FailureMode,
    RarePhenomenon,
    HistoricWorld,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UnlockTrigger {
    SpeciesAppeared,
    TraitStabilized { trait_name: String, min_duration: u64 },
    BiodiversityThreshold { min_species: u32 },
    MassExtinction,
    SpeciationEvent,
    BiomeCondition { criteria: String, min_tiles: u32 },
    ParamThreshold { param: String, min_value: f32 },
    PopulationExplosion { min_population: f64 },
    TrophicCascade,
    StableEcosystem { min_duration: u64 },
    TotalExtinction,
    RunawayGreenhouse,
    FrozenDeath,
    TrophicCollapse,
    Placeholder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexEntry {
    pub id: String,
    pub category: CodexCategory,
    pub name: String,
    pub unlock_trigger: UnlockTrigger,
    pub requirements_text: String,
    pub facts_text: String,
    pub flavor_text: String,
    pub related_entry_ids: Vec<String>,
    pub icon_asset_id: String,
}

pub struct CodexTracker {
    entries: Vec<CodexEntry>,
    unlocked: HashSet<String>,
}

impl CodexTracker {
    pub fn new(entries: Vec<CodexEntry>) -> Self {
        Self {
            entries,
            unlocked: HashSet::new(),
        }
    }

    pub fn unlocked_ids(&self) -> Vec<String> {
        self.unlocked.iter().cloned().collect()
    }

    pub fn check_species_appeared(&mut self) -> Vec<String> {
        self.check_trigger(|t| matches!(t, UnlockTrigger::SpeciesAppeared))
    }

    pub fn check_speciation(&mut self) -> Vec<String> {
        self.check_trigger(|t| matches!(t, UnlockTrigger::SpeciationEvent))
    }

    pub fn check_mass_extinction(&mut self) -> Vec<String> {
        self.check_trigger(|t| matches!(t, UnlockTrigger::MassExtinction))
    }

    pub fn check_biodiversity(&mut self, count: u32) -> Vec<String> {
        self.check_trigger(|t| {
            matches!(t, UnlockTrigger::BiodiversityThreshold { min_species } if count >= *min_species)
        })
    }

    fn check_trigger<F>(&mut self, predicate: F) -> Vec<String>
    where
        F: Fn(&UnlockTrigger) -> bool,
    {
        let mut newly_unlocked = Vec::new();
        for entry in &self.entries {
            if !self.unlocked.contains(&entry.id) && predicate(&entry.unlock_trigger) {
                newly_unlocked.push(entry.id.clone());
            }
        }
        for id in &newly_unlocked {
            self.unlocked.insert(id.clone());
        }
        newly_unlocked
    }
}

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::types::*;

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

#[derive(Serialize, Deserialize)]
pub struct CodexTracker {
    entries: Vec<CodexEntry>,
    unlocked: HashSet<String>,
    #[serde(skip)]
    new_unlocks: Vec<String>,
    // For stable ecosystem tracking
    trophic_sustained_steps: u64,
}

impl CodexTracker {
    pub fn new(entries: Vec<CodexEntry>) -> Self {
        Self {
            entries,
            unlocked: HashSet::new(),
            new_unlocks: Vec::new(),
            trophic_sustained_steps: 0,
        }
    }

    pub fn entries(&self) -> &[CodexEntry] {
        &self.entries
    }

    pub fn unlocked_ids(&self) -> Vec<String> {
        self.unlocked.iter().cloned().collect()
    }

    /// Returns newly unlocked entry IDs since last call, then clears the buffer.
    pub fn drain_new_unlocks(&mut self) -> Vec<String> {
        std::mem::take(&mut self.new_unlocks)
    }

    /// Restore previously unlocked entries (used after loading saved state).
    pub fn restore_unlocked(&mut self, ids: Vec<String>) {
        for id in ids {
            self.unlocked.insert(id);
        }
    }

    /// Run all checks for the current simulation state. Called every SPECIATION_EPOCH steps.
    pub fn check_all(
        &mut self,
        grid: &WorldGrid,
        species: &[Species],
        params: &PlanetParams,
        events: &[SimEvent],
        step: u64,
    ) {
        self.check_species_appeared(events);
        self.check_speciation(events);
        self.check_mass_extinction(events);
        self.check_biodiversity(grid, species);
        self.check_param_thresholds(params);
        self.check_biome_conditions(grid, params);
        self.check_population_explosions(grid, species);
        self.check_trait_stabilized(species, step);
        self.check_stable_ecosystem(grid, species);
        self.check_failure_conditions(grid, species, params);
    }

    fn unlock(&mut self, id: &str) {
        if !self.unlocked.contains(id) {
            self.unlocked.insert(id.to_string());
            self.new_unlocks.push(id.to_string());
        }
    }

    /// Collect matching entry IDs then unlock them, avoiding borrow conflicts.
    fn unlock_matching<F>(&mut self, predicate: F)
    where
        F: Fn(&CodexEntry) -> bool,
    {
        let ids: Vec<String> = self.entries.iter()
            .filter(|e| !self.unlocked.contains(&e.id) && predicate(e))
            .map(|e| e.id.clone())
            .collect();
        for id in ids {
            self.unlock(&id);
        }
    }

    fn check_species_appeared(&mut self, events: &[SimEvent]) {
        let has_appeared = events.iter().any(|e| matches!(e, SimEvent::SpeciesAppeared { .. }));
        if has_appeared {
            self.unlock_matching(|e| matches!(e.unlock_trigger, UnlockTrigger::SpeciesAppeared));
        }
    }

    fn check_speciation(&mut self, events: &[SimEvent]) {
        let has_speciation = events.iter().any(|e| matches!(e, SimEvent::Speciation { .. }));
        if has_speciation {
            self.unlock_matching(|e| matches!(e.unlock_trigger, UnlockTrigger::SpeciationEvent));
        }
    }

    fn check_mass_extinction(&mut self, events: &[SimEvent]) {
        let has_mass = events.iter().any(|e| matches!(e, SimEvent::MassExtinction { .. }));
        if has_mass {
            self.unlock_matching(|e| matches!(e.unlock_trigger, UnlockTrigger::MassExtinction));
        }
    }

    fn check_biodiversity(&mut self, grid: &WorldGrid, species: &[Species]) {
        let count = crate::biosphere::biodiversity_count(grid, species);
        self.unlock_matching(|e| {
            matches!(&e.unlock_trigger, UnlockTrigger::BiodiversityThreshold { min_species } if count >= *min_species)
        });
    }

    fn check_param_thresholds(&mut self, params: &PlanetParams) {
        // Collect IDs to unlock first to avoid borrow conflict
        let ids: Vec<String> = self.entries.iter()
            .filter(|e| !self.unlocked.contains(&e.id))
            .filter_map(|e| {
                if let UnlockTrigger::ParamThreshold { param, min_value } = &e.unlock_trigger {
                    let actual = match param.as_str() {
                        "co2" => params.atmosphere.co2,
                        "o2" => params.atmosphere.o2,
                        "magnetic_field" => params.magnetic_field,
                        "ocean_coverage" => params.hydrology.ocean_coverage,
                        "current_strength" => params.hydrology.current_strength,
                        _ => 0.0,
                    };
                    if actual >= *min_value {
                        return Some(e.id.clone());
                    }
                }
                None
            })
            .collect();
        for id in ids {
            self.unlock(&id);
        }
    }

    fn check_biome_conditions(&mut self, grid: &WorldGrid, params: &PlanetParams) {
        let ids: Vec<String> = self.entries.iter()
            .filter(|e| !self.unlocked.contains(&e.id))
            .filter_map(|e| {
                if let UnlockTrigger::BiomeCondition { criteria, min_tiles } = &e.unlock_trigger {
                    let count = count_biome_tiles(grid, params, criteria);
                    if count >= *min_tiles {
                        return Some(e.id.clone());
                    }
                }
                None
            })
            .collect();
        for id in ids {
            self.unlock(&id);
        }
    }

    fn check_population_explosions(&mut self, grid: &WorldGrid, species: &[Species]) {
        let ids: Vec<String> = self.entries.iter()
            .filter(|e| !self.unlocked.contains(&e.id))
            .filter_map(|e| {
                if let UnlockTrigger::PopulationExplosion { min_population } = &e.unlock_trigger {
                    for s in species {
                        let pop = crate::biosphere::global_population(grid, s.id);
                        if pop >= *min_population {
                            return Some(e.id.clone());
                        }
                    }
                }
                None
            })
            .collect();
        for id in ids {
            self.unlock(&id);
        }
    }

    fn check_trait_stabilized(&mut self, species: &[Species], _step: u64) {
        // Simplified: check if any living species has the trait now.
        // Full duration tracking would require per-species timers — for MVP we unlock
        // when the trait exists (the min_duration is aspirational).
        let ids: Vec<String> = self.entries.iter()
            .filter(|e| !self.unlocked.contains(&e.id))
            .filter_map(|e| {
                if let UnlockTrigger::TraitStabilized { trait_name, .. } = &e.unlock_trigger {
                    let matched = species.iter().any(|s| match trait_name.as_str() {
                        "temp_optimal_below_zero" => s.traits.temp_optimal < 0.0,
                        "temp_optimal_very_cold" => s.traits.temp_optimal < -20.0,
                        "temp_optimal_below_neg10" => s.traits.temp_optimal < -10.0,
                        "temp_optimal_above_40" => s.traits.temp_optimal > 40.0,
                        "o2_need_very_low" => s.traits.o2_need < 0.02,
                        "toxin_resistance_high" => s.traits.toxin_resistance > 0.6,
                        "toxin_resistance_very_high" => s.traits.toxin_resistance > 0.8,
                        _ => false,
                    });
                    if matched {
                        return Some(e.id.clone());
                    }
                }
                None
            })
            .collect();
        for id in ids {
            self.unlock(&id);
        }
    }

    fn check_stable_ecosystem(&mut self, grid: &WorldGrid, species: &[Species]) {
        let levels = crate::biosphere::trophic_level_count(grid, species);
        if levels >= 3 {
            self.trophic_sustained_steps += 1000; // Called every SPECIATION_EPOCH
        } else {
            self.trophic_sustained_steps = 0;
        }

        let sustained = self.trophic_sustained_steps;
        self.unlock_matching(|e| {
            matches!(&e.unlock_trigger, UnlockTrigger::StableEcosystem { min_duration } if sustained >= *min_duration)
        });
    }

    fn check_failure_conditions(&mut self, grid: &WorldGrid, species: &[Species], params: &PlanetParams) {
        let total_biomass: f64 = grid.tiles.iter()
            .flat_map(|t| t.populations.values())
            .sum();

        // Total extinction
        if total_biomass <= 0.0 && !species.is_empty() {
            self.unlock_matching(|e| matches!(e.unlock_trigger, UnlockTrigger::TotalExtinction));
        }

        // Runaway greenhouse (avg temp > 60)
        let avg_temp = grid.tiles.iter().map(|t| t.temperature as f64).sum::<f64>() / grid.tiles.len() as f64;
        if avg_temp > 60.0 {
            self.unlock_matching(|e| matches!(e.unlock_trigger, UnlockTrigger::RunawayGreenhouse));
        }

        // Frozen death (ice_fraction > 0.9)
        if params.hydrology.ice_fraction > 0.9 {
            self.unlock_matching(|e| matches!(e.unlock_trigger, UnlockTrigger::FrozenDeath));
        }

        // Trophic collapse (a trophic level has 0 living species)
        let levels = crate::biosphere::trophic_level_count(grid, species);
        let has_any_life = total_biomass > 0.0;
        if has_any_life && levels < 3 && species.len() >= 3 {
            // At least 3 species were added but not all trophic levels survive
            self.unlock_matching(|e| matches!(e.unlock_trigger, UnlockTrigger::TrophicCollapse));
        }
    }
}

fn count_biome_tiles(grid: &WorldGrid, params: &PlanetParams, criteria: &str) -> u32 {
    let mut count = 0u32;
    for tile in &grid.tiles {
        let matches = match criteria {
            "ocean_hot_nutrient" => tile.is_ocean && tile.temperature > 40.0 && tile.nutrients > 0.5,
            "land_frozen" => !tile.is_ocean && tile.temperature < -30.0,
            "ocean_warm_populated" => {
                let total_pop: f64 = tile.populations.values().sum();
                tile.is_ocean && tile.temperature > 10.0 && tile.temperature < 30.0 && total_pop > 100.0
            }
            "toxic_inhabited" => {
                let total_pop: f64 = tile.populations.values().sum();
                params.atmosphere.toxicity > 0.5 && total_pop > 0.0
            }
            "land_barren" => !tile.is_ocean && tile.nutrients < 0.05,
            _ => false,
        };
        if matches { count += 1; }
    }
    count
}

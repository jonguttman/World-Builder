use rand::Rng;
use rand_chacha::ChaCha8Rng;

use crate::types::*;

const EXTINCTION_THRESHOLD: f64 = 0.5;

/// How suitable a tile is for a species (0.0–1.0)
pub fn suitability(traits: &SpeciesTraits, tile: &Tile, atmo: &AtmosphereState) -> f32 {
    // Temperature suitability: gaussian-like falloff from optimal
    let temp_diff = (tile.temperature - traits.temp_optimal).abs();
    let temp_suit = if traits.temp_range > 0.0 {
        (1.0 - (temp_diff / traits.temp_range).powi(2)).max(0.0)
    } else {
        0.0
    };

    // Oxygen suitability
    let o2_suit = if traits.o2_need > 0.0 {
        (atmo.o2 / traits.o2_need).min(1.0)
    } else {
        1.0
    };

    // Toxicity resistance
    let tox_suit = if atmo.toxicity > traits.toxin_resistance {
        (1.0 - (atmo.toxicity - traits.toxin_resistance)).max(0.0)
    } else {
        1.0
    };

    // Nutrient availability for producers
    let nutrient_suit = match traits.trophic_level {
        TrophicLevel::Producer => tile.nutrients.min(1.0),
        _ => 1.0,
    };

    (temp_suit * o2_suit * tox_suit * nutrient_suit).clamp(0.0, 1.0)
}

/// Carrying capacity for a species on a tile
pub fn carrying_capacity(traits: &SpeciesTraits, tile: &Tile, atmo: &AtmosphereState) -> f64 {
    let suit = suitability(traits, tile, atmo) as f64;
    let base_capacity = match traits.trophic_level {
        TrophicLevel::Producer => 10_000.0,
        TrophicLevel::Consumer => 1_000.0,
        TrophicLevel::Predator => 100.0,
    };
    base_capacity * suit * tile.nutrients as f64
}

/// Update populations on a single tile for one tick
pub fn update_tile_populations(
    tile: &mut Tile,
    species_list: &[Species],
    atmo: &AtmosphereState,
) {
    let present_ids: Vec<u32> = tile.populations.keys().cloned().collect();

    for &sp_id in &present_ids {
        let species = match species_list.iter().find(|s| s.id == sp_id) {
            Some(s) => s,
            None => continue,
        };

        let pop = *tile.populations.get(&sp_id).unwrap_or(&0.0);
        if pop <= 0.0 {
            continue;
        }

        let suit = suitability(&species.traits, tile, atmo) as f64;
        let capacity = carrying_capacity(&species.traits, tile, atmo);

        let r = species.traits.reproduction_rate as f64;
        let growth = if capacity > 0.0 {
            r * suit * pop * (1.0 - pop / capacity)
        } else {
            -pop * 0.1
        };

        let mortality_rate = 0.02 * (1.0 - suit);
        let mortality = mortality_rate * pop;

        let predation = compute_predation(tile, species, species_list);

        let new_pop = (pop + growth - mortality - predation).max(0.0);

        if new_pop < EXTINCTION_THRESHOLD {
            tile.populations.insert(sp_id, 0.0);
        } else {
            tile.populations.insert(sp_id, new_pop);
        }
    }
}

fn compute_predation(tile: &Tile, prey_species: &Species, all_species: &[Species]) -> f64 {
    let prey_pop = *tile.populations.get(&prey_species.id).unwrap_or(&0.0);
    if prey_pop <= 0.0 {
        return 0.0;
    }

    let prey_level = &prey_species.traits.trophic_level;
    let mut total_consumed = 0.0;

    for predator in all_species {
        let eats_prey = match (&predator.traits.trophic_level, prey_level) {
            (TrophicLevel::Predator, TrophicLevel::Consumer) => true,
            (TrophicLevel::Consumer, TrophicLevel::Producer) => true,
            _ => false,
        };

        if !eats_prey {
            continue;
        }

        let pred_pop = *tile.populations.get(&predator.id).unwrap_or(&0.0);
        if pred_pop <= 0.0 {
            continue;
        }

        let a = 0.01;
        let h = 0.001;
        let consumed = a * pred_pop * prey_pop / (1.0 + h * prey_pop);
        total_consumed += consumed;
    }

    total_consumed
}

/// Update all biosphere populations across the grid
pub fn update_grid(grid: &mut WorldGrid, species: &[Species], atmo: &AtmosphereState) {
    let height = grid.height;
    let width = grid.width;

    for y in 0..height {
        for x in 0..width {
            let tile = grid.get_mut(x, y);
            update_tile_populations(tile, species, atmo);
        }
    }
}

/// Count total global population of a species across the grid
pub fn global_population(grid: &WorldGrid, species_id: u32) -> f64 {
    grid.tiles
        .iter()
        .map(|t| t.populations.get(&species_id).unwrap_or(&0.0))
        .sum()
}

/// Count how many distinct species have non-zero global population
pub fn biodiversity_count(grid: &WorldGrid, species: &[Species]) -> u32 {
    species
        .iter()
        .filter(|s| global_population(grid, s.id) > 0.0)
        .count() as u32
}

/// Nudge a value by a random amount within a range
fn nudge(val: f32, range: f32, rng: &mut ChaCha8Rng) -> f32 {
    val + rng.gen_range(-range..range)
}

/// Slightly mutate species traits
pub fn mutate_traits(original: &SpeciesTraits, rng: &mut ChaCha8Rng) -> SpeciesTraits {
    let mut t = original.clone();

    t.temp_optimal = nudge(t.temp_optimal, 3.0, rng);
    t.temp_range = nudge(t.temp_range, 2.0, rng).max(5.0);
    t.o2_need = nudge(t.o2_need, 0.02, rng).max(0.0);
    t.toxin_resistance = nudge(t.toxin_resistance, 0.05, rng).clamp(0.0, 1.0);
    t.reproduction_rate = nudge(t.reproduction_rate, 0.005, rng).max(0.001);
    t.dispersal = nudge(t.dispersal, 0.05, rng).clamp(0.0, 1.0);
    t.mutation_rate = nudge(t.mutation_rate, 0.002, rng).clamp(0.001, 1.0);
    // trophic_level stays the same
    t
}

/// Attempt speciation. Returns Some(new_species) if mutation fires.
pub fn try_speciate(
    parent: &Species,
    new_id: u32,
    rng: &mut ChaCha8Rng,
) -> Option<Species> {
    let roll: f32 = rng.gen();
    if roll < parent.traits.mutation_rate {
        let new_traits = mutate_traits(&parent.traits, rng);
        let name = format!("{}-v{}", parent.name, new_id);
        Some(Species {
            id: new_id,
            name,
            traits: new_traits,
        })
    } else {
        None
    }
}

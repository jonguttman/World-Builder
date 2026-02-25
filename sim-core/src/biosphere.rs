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

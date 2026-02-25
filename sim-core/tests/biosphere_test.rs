use planet_architect_sim::biosphere;
use planet_architect_sim::types::*;

fn make_producer() -> Species {
    Species {
        id: 0,
        name: "Proto-algae".to_string(),
        traits: SpeciesTraits {
            temp_optimal: 20.0,
            temp_range: 40.0,
            o2_need: 0.0,
            toxin_resistance: 0.1,
            trophic_level: TrophicLevel::Producer,
            reproduction_rate: 0.05,
            dispersal: 0.2,
            mutation_rate: 0.01,
        },
    }
}

fn make_habitable_tile() -> Tile {
    Tile {
        elevation: -0.5,
        is_ocean: true,
        temperature: 20.0,
        moisture: 1.0,
        nutrients: 0.5,
        radiation: 0.2,
        biome_id: 1,
        populations: std::collections::HashMap::new(),
    }
}

#[test]
fn test_suitability_optimal_conditions() {
    let species = make_producer();
    let tile = make_habitable_tile();
    let params = PlanetParams::default();
    let suit = biosphere::suitability(&species.traits, &tile, &params);
    assert!(suit >= 0.4, "Optimal conditions should give high suitability: {}", suit);
}

#[test]
fn test_suitability_wrong_temperature() {
    let species = make_producer();
    let mut tile = make_habitable_tile();
    tile.temperature = -60.0;
    let params = PlanetParams::default();
    let suit = biosphere::suitability(&species.traits, &tile, &params);
    assert!(suit < 0.1, "Wrong temp should give low suitability: {}", suit);
}

#[test]
fn test_population_grows_in_good_conditions() {
    let species = make_producer();
    let mut tile = make_habitable_tile();
    tile.populations.insert(0, 100.0);
    let params = PlanetParams::default();
    let species_list = vec![species];
    biosphere::update_tile_populations(&mut tile, &species_list, &params);
    let pop = tile.populations.get(&0).unwrap();
    assert!(*pop > 100.0, "Population should grow in good conditions: {}", pop);
}

#[test]
fn test_population_declines_in_bad_conditions() {
    let species = make_producer();
    let mut tile = make_habitable_tile();
    tile.temperature = -50.0;
    tile.populations.insert(0, 100.0);
    let params = PlanetParams::default();
    let species_list = vec![species];
    biosphere::update_tile_populations(&mut tile, &species_list, &params);
    let pop = tile.populations.get(&0).unwrap();
    assert!(*pop < 100.0, "Population should decline in bad conditions: {}", pop);
}

#[test]
fn test_population_bounded_by_carrying_capacity() {
    let species = make_producer();
    let mut tile = make_habitable_tile();
    tile.populations.insert(0, 1_000_000.0);
    let params = PlanetParams::default();
    let species_list = vec![species];
    for _ in 0..100 {
        biosphere::update_tile_populations(&mut tile, &species_list, &params);
    }
    let pop = tile.populations.get(&0).unwrap();
    assert!(*pop < 1_000_000.0, "Population should be bounded by carrying capacity: {}", pop);
}

#[test]
fn test_extinct_species_removed() {
    let species = make_producer();
    let mut tile = make_habitable_tile();
    tile.temperature = -70.0;
    tile.nutrients = 0.0;
    tile.populations.insert(0, 1.0);
    let params = PlanetParams::default();
    let species_list = vec![species];
    for _ in 0..100 {
        biosphere::update_tile_populations(&mut tile, &species_list, &params);
    }
    let pop = *tile.populations.get(&0).unwrap_or(&0.0);
    assert!(pop == 0.0, "Tiny population in lethal conditions should go extinct: {}", pop);
}

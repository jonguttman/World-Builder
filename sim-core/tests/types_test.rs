use planet_architect_sim::types::*;

#[test]
fn test_planet_params_default_creates_valid_planet() {
    let params = PlanetParams::default();
    assert!(params.gravity > 0.0);
    assert!(params.atmosphere.pressure > 0.0);
    assert!(params.hydrology.ocean_coverage >= 0.0);
    assert!(params.hydrology.ocean_coverage <= 1.0);
}

#[test]
fn test_tile_default_is_barren() {
    let tile = Tile::default();
    assert_eq!(tile.elevation, 0.0);
    assert!(!tile.is_ocean);
    assert!(tile.populations.is_empty());
}

#[test]
fn test_world_grid_dimensions() {
    let grid = WorldGrid::new(64, 32);
    assert_eq!(grid.width, 64);
    assert_eq!(grid.height, 32);
    assert_eq!(grid.tiles.len(), 64 * 32);
}

#[test]
fn test_species_traits_serialization() {
    let species = Species {
        id: 0,
        name: "Proto-microbe".to_string(),
        traits: SpeciesTraits {
            temp_optimal: 25.0,
            temp_range: 40.0,
            o2_need: 0.0,
            toxin_resistance: 0.1,
            trophic_level: TrophicLevel::Producer,
            reproduction_rate: 0.05,
            dispersal: 0.3,
            mutation_rate: 0.01,
        },
    };
    let json = serde_json::to_string(&species).unwrap();
    let deserialized: Species = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "Proto-microbe");
}

#[test]
fn test_intervention_types() {
    let intervention = Intervention {
        kind: InterventionKind::AdjustCO2 { delta: 0.1 },
        target_region: None,
        step: 1000,
    };
    assert_eq!(intervention.step, 1000);
}

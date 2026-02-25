use planet_architect_sim::sim::Simulation;
use planet_architect_sim::types::*;

#[test]
fn test_save_and_load_produces_identical_state() {
    let mut sim = Simulation::new(42, PlanetParams::default());

    let producer = Species {
        id: 0,
        name: "Algae".to_string(),
        traits: SpeciesTraits {
            temp_optimal: 15.0,
            temp_range: 50.0,
            o2_need: 0.0,
            toxin_resistance: 0.1,
            trophic_level: TrophicLevel::Producer,
            reproduction_rate: 0.05,
            dispersal: 0.3,
            mutation_rate: 0.01,
        },
    };
    sim.add_species(producer, 100.0);
    sim.step(500);

    let bytes = sim.save_state().expect("save should work");
    let loaded = Simulation::load_state(&bytes).expect("load should work");

    assert_eq!(sim.current_step(), loaded.current_step());

    let snap1 = sim.snapshot();
    let snap2 = loaded.snapshot();

    for i in 0..snap1.grid.tiles.len() {
        assert_eq!(snap1.grid.tiles[i].temperature, snap2.grid.tiles[i].temperature,
            "Tile {} temperature mismatch after load", i);
    }
}

#[test]
fn test_loaded_sim_continues_deterministically() {
    let mut sim = Simulation::new(42, PlanetParams::default());
    sim.step(100);

    let bytes = sim.save_state().unwrap();
    let mut loaded = Simulation::load_state(&bytes).unwrap();

    sim.step(100);
    loaded.step(100);

    let snap1 = sim.snapshot();
    let snap2 = loaded.snapshot();

    assert_eq!(snap1.current_step, snap2.current_step);
    for i in 0..snap1.grid.tiles.len() {
        assert_eq!(snap1.grid.tiles[i].temperature, snap2.grid.tiles[i].temperature);
    }
}

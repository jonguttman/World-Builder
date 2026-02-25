use planet_architect_sim::sim::Simulation;
use planet_architect_sim::types::PlanetParams;

#[test]
fn test_determinism_same_seed_same_result() {
    let seed = 42u64;
    let params = PlanetParams::default();

    let mut sim1 = Simulation::new(seed, params.clone());
    sim1.step(1000);
    let snap1 = sim1.snapshot();

    let mut sim2 = Simulation::new(seed, params);
    sim2.step(1000);
    let snap2 = sim2.snapshot();

    assert_eq!(snap1.current_step, snap2.current_step);
    assert_eq!(snap1.current_step, 1000);
    for i in 0..snap1.grid.tiles.len() {
        assert_eq!(snap1.grid.tiles[i].temperature, snap2.grid.tiles[i].temperature,
            "Temperature mismatch at tile {}", i);
        assert_eq!(snap1.grid.tiles[i].nutrients, snap2.grid.tiles[i].nutrients,
            "Nutrient mismatch at tile {}", i);
    }
}

#[test]
fn test_different_seed_different_result() {
    let params = PlanetParams::default();

    let mut sim1 = Simulation::new(1, params.clone());
    sim1.step(100);

    let mut sim2 = Simulation::new(2, params);
    sim2.step(100);

    let snap1 = sim1.snapshot();
    let snap2 = sim2.snapshot();
    let diffs: usize = snap1.grid.tiles.iter().zip(snap2.grid.tiles.iter())
        .filter(|(a, b)| (a.temperature - b.temperature).abs() > 0.001)
        .count();
    assert!(diffs > 0, "Different seeds should produce different worlds");
}

#[test]
fn test_step_advances_time() {
    let mut sim = Simulation::new(42, PlanetParams::default());
    assert_eq!(sim.current_step(), 0);
    sim.step(500);
    assert_eq!(sim.current_step(), 500);
    sim.step(500);
    assert_eq!(sim.current_step(), 1000);
}

use planet_architect_sim::sim::Simulation;
use planet_architect_sim::types::*;

#[test]
fn test_adjust_co2_changes_atmosphere() {
    let mut sim = Simulation::new(42, PlanetParams::default());
    let co2_before = sim.params().atmosphere.co2;

    let intervention = Intervention {
        kind: InterventionKind::AdjustCO2 { delta: 0.01 },
        target_region: None,
        step: 0,
    };

    let result = sim.apply_intervention(intervention);
    assert!(result.is_ok());

    let co2_after = sim.params().atmosphere.co2;
    assert!((co2_after - co2_before - 0.01).abs() < 0.0001,
        "CO2 should increase by delta: {} -> {}", co2_before, co2_after);
}

#[test]
fn test_adjust_o2_changes_atmosphere() {
    let mut sim = Simulation::new(42, PlanetParams::default());
    let o2_before = sim.params().atmosphere.o2;

    let intervention = Intervention {
        kind: InterventionKind::AdjustO2 { delta: 0.05 },
        target_region: None,
        step: 0,
    };

    sim.apply_intervention(intervention).unwrap();
    let o2_after = sim.params().atmosphere.o2;
    assert!((o2_after - o2_before - 0.05).abs() < 0.0001);
}

#[test]
fn test_nutrient_bloom_increases_nutrients() {
    let mut sim = Simulation::new(42, PlanetParams::default());

    let target = RegionTarget { x: 32, y: 16, radius: 3 };
    let nutrients_before = sim.grid().get(32, 16).nutrients;

    let intervention = Intervention {
        kind: InterventionKind::NutrientBloom { magnitude: 0.5 },
        target_region: Some(target),
        step: 0,
    };

    sim.apply_intervention(intervention).unwrap();
    let nutrients_after = sim.grid().get(32, 16).nutrients;
    assert!(nutrients_after > nutrients_before,
        "Nutrient bloom should increase nutrients: {} -> {}", nutrients_before, nutrients_after);
}

#[test]
fn test_intervention_values_clamped() {
    let mut sim = Simulation::new(42, PlanetParams::default());

    let intervention = Intervention {
        kind: InterventionKind::AdjustCO2 { delta: 100.0 },
        target_region: None,
        step: 0,
    };

    sim.apply_intervention(intervention).unwrap();
    assert!(sim.params().atmosphere.co2 <= 1.0, "CO2 should be clamped");
}

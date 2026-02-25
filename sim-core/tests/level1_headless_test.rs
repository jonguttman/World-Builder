use planet_architect_sim::sim::Simulation;
use planet_architect_sim::level::*;
use planet_architect_sim::biosphere;
use planet_architect_sim::types::*;

#[test]
fn test_level1_can_be_loaded_and_started() {
    let json = include_str!("../levels/level_01_first_breath.json");
    let spec: LevelSpec = serde_json::from_str(json).unwrap();

    let params = spec.starting_params.unwrap_or_default();
    let sim = Simulation::new(spec.starting_seed, params);

    assert_eq!(sim.current_step(), 0);
}

#[test]
fn test_level1_determinism_across_runs() {
    let json = include_str!("../levels/level_01_first_breath.json");
    let spec: LevelSpec = serde_json::from_str(json).unwrap();
    let params = spec.starting_params.unwrap_or_default();

    let mut sim1 = Simulation::new(spec.starting_seed, params.clone());
    let mut sim2 = Simulation::new(spec.starting_seed, params);

    let microbe = Species {
        id: 0,
        name: "Thermophile".to_string(),
        traits: SpeciesTraits {
            temp_optimal: 5.0,
            temp_range: 60.0,
            o2_need: 0.0,
            toxin_resistance: 0.3,
            trophic_level: TrophicLevel::Producer,
            reproduction_rate: 0.04,
            dispersal: 0.2,
            mutation_rate: 0.005,
        },
    };

    sim1.add_species(microbe.clone(), 50.0);
    sim2.add_species(microbe, 50.0);

    sim1.step(5000);
    sim2.step(5000);

    let pop1 = biosphere::global_population(sim1.grid(), 0);
    let pop2 = biosphere::global_population(sim2.grid(), 0);

    assert_eq!(pop1, pop2, "Same seed + same species should give identical populations");
    assert_eq!(sim1.current_step(), sim2.current_step());
}

#[test]
fn test_level1_microbes_can_survive() {
    let json = include_str!("../levels/level_01_first_breath.json");
    let spec: LevelSpec = serde_json::from_str(json).unwrap();
    let mut params = spec.starting_params.unwrap_or_default();

    // Player interventions to make planet habitable
    params.atmosphere.co2 = 0.05;
    params.hydrology.ice_fraction = 0.2;
    params.core_heat = 0.4;

    let mut sim = Simulation::new(spec.starting_seed, params);

    // Warm up nutrients first
    sim.step(500);

    let microbe = Species {
        id: 0,
        name: "Extremophile".to_string(),
        traits: SpeciesTraits {
            temp_optimal: 0.0,
            temp_range: 60.0,
            o2_need: 0.0,
            toxin_resistance: 0.3,
            trophic_level: TrophicLevel::Producer,
            reproduction_rate: 0.04,
            dispersal: 0.3,
            mutation_rate: 0.005,
        },
    };
    sim.add_species(microbe, 100.0);

    sim.step(10_000);

    let pop = biosphere::global_population(sim.grid(), 0);
    assert!(pop > 0.0, "Microbes should survive with player intervention: pop={}", pop);

    let snap = sim.snapshot();
    assert!(snap.biodiversity_count >= 1);
}

#[test]
fn test_level1_objective_evaluator_integration() {
    let json = include_str!("../levels/level_01_first_breath.json");
    let spec: LevelSpec = serde_json::from_str(json).unwrap();

    let mut eval = ObjectiveEvaluator::new(spec.objective.clone());

    let status = eval.evaluate(0.0, 0, 0);
    assert_eq!(status, ObjectiveStatus::InProgress);

    for step in 1..=20 {
        eval.evaluate(10000.0, 3, step);
    }
    assert!(eval.sustained_steps() >= 20);
}

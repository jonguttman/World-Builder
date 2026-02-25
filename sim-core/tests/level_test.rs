use planet_architect_sim::level::*;

#[test]
fn test_load_level_spec() {
    let json = r#"{
        "id": "level_01",
        "name": "First Breath",
        "pack": "FREE",
        "description": "Establish microbial life for 10M years",
        "starting_seed": 42,
        "starting_params": null,
        "allowed_interventions": ["AdjustCO2", "AdjustO2", "NutrientBloom"],
        "energy_budget": 100.0,
        "objective": {
            "type": "MicrobialStability",
            "min_biomass": 1000.0,
            "required_duration_steps": 10000000
        },
        "fail_conditions": ["Extinction"]
    }"#;

    let spec: LevelSpec = serde_json::from_str(json).unwrap();
    assert_eq!(spec.id, "level_01");
    assert_eq!(spec.name, "First Breath");
    assert_eq!(spec.pack, Pack::Free);
}

#[test]
fn test_objective_not_met_initially() {
    let objective = Objective::MicrobialStability {
        min_biomass: 1000.0,
        required_duration_steps: 100,
    };
    let mut eval = ObjectiveEvaluator::new(objective);
    let status = eval.evaluate(0.0, 0, 0);
    assert_eq!(status, ObjectiveStatus::InProgress);
}

#[test]
fn test_objective_met_after_sustained_biomass() {
    let objective = Objective::MicrobialStability {
        min_biomass: 100.0,
        required_duration_steps: 10,
    };
    let mut eval = ObjectiveEvaluator::new(objective);

    for step in 0..10 {
        let status = eval.evaluate(500.0, 1, step);
        if step < 9 {
            assert_eq!(status, ObjectiveStatus::InProgress);
        }
    }
    let final_status = eval.evaluate(500.0, 1, 10);
    assert_eq!(final_status, ObjectiveStatus::Complete);
}

#[test]
fn test_objective_fails_on_extinction() {
    let objective = Objective::MicrobialStability {
        min_biomass: 100.0,
        required_duration_steps: 100,
    };
    let mut eval = ObjectiveEvaluator::new(objective);

    eval.evaluate(500.0, 1, 0);
    let status = eval.evaluate(0.0, 0, 1);
    assert_eq!(status, ObjectiveStatus::Failed);
}

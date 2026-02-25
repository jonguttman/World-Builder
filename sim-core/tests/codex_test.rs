use planet_architect_sim::codex::*;

#[test]
fn test_codex_entry_deserialization() {
    let json = r#"{
        "id": "body_plan_001",
        "category": "BodyPlan",
        "name": "Single-Cell Photosynthesis",
        "unlock_trigger": { "type": "TraitStabilized", "trait_name": "photosynthesis", "min_duration": 1000 },
        "requirements_text": "Sustain a photosynthetic species for 1000 years",
        "facts_text": "The first organisms to harvest starlight.",
        "flavor_text": "A tiny cell turns toward a distant sun — and everything changes.",
        "related_entry_ids": ["species_001"],
        "icon_asset_id": "icon_photosynthesis"
    }"#;

    let entry: CodexEntry = serde_json::from_str(json).unwrap();
    assert_eq!(entry.id, "body_plan_001");
    assert_eq!(entry.category, CodexCategory::BodyPlan);
}

#[test]
fn test_codex_tracker_unlocks_on_condition() {
    let entry = CodexEntry {
        id: "species_first_life".to_string(),
        category: CodexCategory::Species,
        name: "First Life".to_string(),
        unlock_trigger: UnlockTrigger::SpeciesAppeared,
        requirements_text: "Any species appears".to_string(),
        facts_text: "Life finds a way.".to_string(),
        flavor_text: "From chemistry to biology, in a single step.".to_string(),
        related_entry_ids: vec![],
        icon_asset_id: "icon_first_life".to_string(),
    };

    let mut tracker = CodexTracker::new(vec![entry]);
    assert!(tracker.unlocked_ids().is_empty());

    let unlocks = tracker.check_species_appeared();
    assert_eq!(unlocks.len(), 1);
    assert_eq!(unlocks[0], "species_first_life");

    // Should not unlock twice
    let unlocks2 = tracker.check_species_appeared();
    assert!(unlocks2.is_empty());
}

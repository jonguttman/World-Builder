use planet_architect_sim::biosphere;
use planet_architect_sim::types::*;

#[test]
fn test_mutate_trait_stays_in_bounds() {
    let original = SpeciesTraits {
        temp_optimal: 20.0,
        temp_range: 40.0,
        o2_need: 0.1,
        toxin_resistance: 0.5,
        trophic_level: TrophicLevel::Producer,
        reproduction_rate: 0.05,
        dispersal: 0.3,
        mutation_rate: 0.01,
    };

    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let mutated = biosphere::mutate_traits(&original, &mut rng);

    assert!(mutated.reproduction_rate >= 0.0);
    assert!(mutated.dispersal >= 0.0 && mutated.dispersal <= 1.0);
    assert!(mutated.toxin_resistance >= 0.0 && mutated.toxin_resistance <= 1.0);
    assert!(mutated.temp_range > 0.0);
}

#[test]
fn test_speciation_creates_new_species() {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let parent = Species {
        id: 0,
        name: "Proto-algae".to_string(),
        traits: SpeciesTraits {
            temp_optimal: 20.0,
            temp_range: 40.0,
            o2_need: 0.0,
            toxin_resistance: 0.1,
            trophic_level: TrophicLevel::Producer,
            reproduction_rate: 0.05,
            dispersal: 0.3,
            mutation_rate: 0.5, // high mutation for test
        },
    };

    let child = biosphere::try_speciate(&parent, 1, &mut rng);
    assert!(child.is_some(), "High mutation rate should produce speciation");
    let child = child.unwrap();
    assert_eq!(child.id, 1);
    assert_ne!(child.name, parent.name);
}

use planet_architect_sim::sim::Simulation;
use planet_architect_sim::types::*;
use planet_architect_sim::biosphere;

#[test]
fn test_seeded_microbes_grow_over_time() {
    let mut params = PlanetParams::default();
    params.atmosphere.co2 = 0.01;
    params.core_heat = 0.8;

    let mut sim = Simulation::new(42, params);

    // Warm up: let nutrients accumulate before introducing life
    sim.step(500);

    let producer = Species {
        id: 0,
        name: "Proto-algae".to_string(),
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
    sim.add_species(producer.clone(), 100.0);

    let pop_before = biosphere::global_population(sim.grid(), 0);
    assert!(pop_before > 0.0, "Should have seeded population");

    sim.step(500);

    let pop_after = biosphere::global_population(sim.grid(), 0);
    assert!(pop_after > pop_before,
        "Population should grow over 500 steps: {} -> {}", pop_before, pop_after);
}

#[test]
fn test_food_web_consumer_coexists_with_producer() {
    let mut params = PlanetParams::default();
    params.core_heat = 0.8;

    let mut sim = Simulation::new(42, params);

    // Warm up: let nutrients accumulate before introducing life
    sim.step(500);

    let producer = Species {
        id: 0,
        name: "Algae".to_string(),
        traits: SpeciesTraits {
            temp_optimal: 15.0,
            temp_range: 50.0,
            o2_need: 0.0,
            toxin_resistance: 0.1,
            trophic_level: TrophicLevel::Producer,
            reproduction_rate: 0.08,
            dispersal: 0.2,
            mutation_rate: 0.01,
        },
    };
    sim.add_species(producer, 500.0);

    // Let producers establish before introducing consumers
    sim.step(200);

    let consumer = Species {
        id: 1,
        name: "Grazer".to_string(),
        traits: SpeciesTraits {
            temp_optimal: 15.0,
            temp_range: 50.0,
            o2_need: 0.1,
            toxin_resistance: 0.1,
            trophic_level: TrophicLevel::Consumer,
            reproduction_rate: 0.03,
            dispersal: 0.3,
            mutation_rate: 0.01,
        },
    };
    sim.add_species(consumer, 50.0);

    sim.step(200);

    let prod_pop = biosphere::global_population(sim.grid(), 0);
    let cons_pop = biosphere::global_population(sim.grid(), 1);

    // Both trophic levels should coexist in the food web
    assert!(prod_pop > 0.0, "Producers should survive: {}", prod_pop);
    assert!(cons_pop > 0.0, "Consumers should survive with producers: {}", cons_pop);

    // Biodiversity should reflect both surviving species
    let biodiv = biosphere::biodiversity_count(sim.grid(), sim.species());
    assert_eq!(biodiv, 2, "Both species should be counted in biodiversity");
}

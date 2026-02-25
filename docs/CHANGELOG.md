# Changelog

## 0.14.0 - 2026-02-24
- Created Swift bridge types mirroring Rust data structures in ios/PlanetArchitect/Core/SimulationBridge/SimTypes.swift
  - PlanetParams, AtmosphereState, HydroState with Earth-like defaults
  - TimeSpeed enum with stepsPerBatch computed property
  - Species, SpeciesTraits, TrophicLevel for biosphere model
  - InterventionKind, RegionTarget, Intervention for player actions
  - LevelSpec with Pack enum (FREE, PACK_CORE, PACK_ADV)
  - CodexEntry with CodexCategory enum
  - SimSnapshot for bridge-layer state transfer
- Created SwiftUI app shell in ios/PlanetArchitect/PlanetArchitectApp.swift
  - @main entry point launching LevelSelectView
- Created LevelSelectView in ios/PlanetArchitect/Features/Campaign/LevelSelectView.swift
  - NavigationStack with Training (3 free levels) and Core Challenge Pack (7 locked levels) sections
  - LevelRow component with lock icon for locked levels
  - Navigation to PlanetView for unlocked levels
- Created PlanetView in ios/PlanetArchitect/Features/Planet/PlanetView.swift
  - Placeholder planet visualization with radial gradient circle
  - Play/Pause button and TimeSpeed segmented picker
  - Step counter display
- Removed .gitkeep files from directories now containing Swift sources

## 0.13.0 - 2026-02-24
- Created Level 1 headless integration test suite (4 tests) in tests/level1_headless_test.rs
  - test_level1_can_be_loaded_and_started: loads level_01_first_breath.json, deserializes LevelSpec, creates Simulation from starting params and seed, verifies step counter starts at 0
  - test_level1_determinism_across_runs: two identical simulations with same seed, species, and 5000 steps produce bit-identical populations
  - test_level1_microbes_can_survive: simulates player interventions (CO2 boost, ice melt, core heat) on Level 1's harsh starting conditions, warms up nutrients for 500 steps, seeds Extremophile microbes, verifies population > 0 after 10k steps and biodiversity >= 1
  - test_level1_objective_evaluator_integration: verifies ObjectiveEvaluator starts InProgress, accumulates sustained steps when biomass exceeds threshold, confirms sustained_steps >= 20 after 20 evaluations

## 0.12.0 - 2026-02-24
- Implemented codex trigger system in sim-core/src/codex.rs
  - CodexCategory enum: Species, BodyPlan, Biome, PlanetarySystem, EvolutionaryEvent, FailureMode, RarePhenomenon, HistoricWorld
  - UnlockTrigger enum with serde tagged variants: SpeciesAppeared, TraitStabilized, BiodiversityThreshold, MassExtinction, SpeciationEvent
  - CodexEntry struct with id, category, name, trigger, requirement/facts/flavor text, related entries, and icon asset
  - CodexTracker with idempotent unlock checking (entries unlock at most once)
  - Trigger check methods: check_species_appeared, check_speciation, check_mass_extinction, check_biodiversity
- Created codex test suite (2 tests) in tests/codex_test.rs
  - JSON deserialization of CodexEntry with tagged UnlockTrigger
  - Tracker unlocks on matching condition and prevents duplicate unlocks

## 0.11.0 - 2026-02-24
- Implemented save/load with deterministic RNG state preservation in sim-core/src/sim.rs
  - SimState struct for bincode serialization of full simulation state including ChaCha8Rng
  - save_state: serializes Simulation to Vec<u8> via bincode, including RNG state for deterministic resumption
  - load_state: deserializes bytes back into a fully functional Simulation with restored RNG
- Enabled serde1 feature on rand_chacha dependency in Cargo.toml
- Created save/load test suite (2 tests) in tests/saveload_test.rs
  - Save and load produces identical state (grid tiles, step count)
  - Loaded simulation continues deterministically (same steps yield same results)

## 0.10.0 - 2026-02-24
- Implemented level spec deserialization and objective evaluator in sim-core/src/level.rs
  - LevelSpec struct with Pack enum (FREE, PACK_CORE, PACK_ADV), starting params, energy budget, and objectives
  - Objective enum with tagged serde variants: MicrobialStability, EcosystemStability, BiodiversityStability
  - ObjectiveEvaluator with sustained-step tracking, completion, and extinction failure detection
  - ObjectiveStatus enum: InProgress, Complete, Failed
- Created Level 1 JSON (levels/level_01_first_breath.json)
  - Barren rocky planet scenario with thin CO2 atmosphere, frozen oceans, weak magnetic field
  - MicrobialStability objective: 5000 min biomass sustained for 10M steps
  - Allowed interventions: AdjustCO2, AdjustO2, NutrientBloom, IceMeltPulse
- Created level test suite (4 tests) in tests/level_test.rs
  - JSON deserialization of LevelSpec with null starting_params
  - Objective not met initially (InProgress)
  - Objective met after sustained biomass above threshold
  - Objective fails on extinction (biomass drops to 0 after progress)

## 0.9.0 - 2026-02-24
- Implemented intervention system in sim-core/src/sim.rs
  - InterventionError enum for error handling
  - apply_intervention: dispatches by InterventionKind — AdjustCO2, AdjustO2, CloudSeeding, NutrientBloom, IceMeltPulse
  - apply_to_region: applies a mutation closure to all tiles within a RegionTarget radius, with safe isize casting for boundary checks
  - All values clamped to [0.0, 1.0] range
- Created intervention test suite (4 tests) in tests/intervention_test.rs
  - CO2 adjustment changes atmosphere by exact delta
  - O2 adjustment changes atmosphere by exact delta
  - Nutrient bloom increases nutrients in targeted region
  - Extreme values are clamped to valid range

## 0.8.0 - 2026-02-24
- Implemented mutation and speciation engine in sim-core/src/biosphere.rs
  - mutate_traits: nudges species traits (temp_optimal, temp_range, o2_need, toxin_resistance, reproduction_rate, dispersal, mutation_rate) with clamped random variation
  - try_speciate: probabilistic speciation roll based on parent's mutation_rate, creates child species with mutated traits
  - nudge helper function for bounded random trait perturbation
- Added epoch-based speciation to simulation tick loop in sim-core/src/sim.rs
  - SPECIATION_EPOCH constant (every 1000 ticks)
  - check_speciation: evaluates species with global_pop > 500 for speciation, seeds children on habitable tiles with existing life, emits Speciation events
- Created speciation test suite (2 tests) in tests/speciation_test.rs
  - Mutated traits stay within valid bounds
  - High mutation rate reliably produces child species with correct id and distinct name

## 0.7.0 - 2026-02-24
- Integrated biosphere into simulation tick loop (sim.rs)
  - biosphere::update_grid called each tick after climate updates
  - Added `use crate::biosphere` import
- Added `add_species` method to Simulation
  - Seeds initial population on tiles with suitability > 0.3
  - Emits SpeciesAppeared event and tracks next_species_id
- Created biosphere integration test suite (2 tests) in tests/integration_test.rs
  - Seeded microbes grow over time with nutrient warm-up
  - Food web coexistence: producers and consumers both survive

## 0.6.0 - 2026-02-24
- Implemented biosphere microbe model in sim-core/src/biosphere.rs
  - suitability: gaussian temperature falloff, O2 need, toxin resistance, nutrient availability
  - carrying_capacity: trophic-level-scaled capacity modulated by suitability and nutrients
  - update_tile_populations: logistic growth with suitability, mortality, and predation pressure
  - compute_predation: Holling Type II functional response (Consumer eats Producer, Predator eats Consumer)
  - update_grid: applies population dynamics across entire WorldGrid
  - global_population: sums species population across all tiles
  - biodiversity_count: counts distinct surviving species
  - Extinction threshold at 0.5 removes tiny populations
- Created biosphere integration test suite (6 tests) in tests/biosphere_test.rs
  - Optimal suitability conditions
  - Wrong temperature reduces suitability
  - Population growth in good conditions
  - Population decline in bad conditions
  - Population bounded by carrying capacity
  - Extinction of tiny populations in lethal conditions

## 0.5.0 - 2026-02-24
- Added nutrient cycling model in sim-core/src/climate.rs
  - update_nutrients: volcanism input, ocean upwelling, biomass decay, moisture leaching
  - Nutrients clamped to [0.0, 1.0] range per tile per tick
- Integrated nutrient update into simulation tick loop (sim.rs)
- Created climate integration test suite (5 tests) in tests/climate_test.rs
  - Greenhouse effect: higher CO2 raises average temperature
  - Ice albedo: more ice lowers average temperature
  - Temperature bounds: extreme params stay within [-80, 80] range
  - Latitude gradient: equator warmer than poles
  - Nutrient volcanism: higher core heat increases nutrient levels

## 0.4.0 - 2026-02-24
- Implemented deterministic tick loop with seeded ChaCha8 RNG in sim-core/src/sim.rs
  - Simulation struct with new(), step(), tick(), snapshot(), and accessor methods
  - Reproducible world generation from any u64 seed
- Implemented basic climate model in sim-core/src/climate.rs
  - init_grid: latitude-based temperature, random elevation/moisture/nutrients, radiation shielding
  - update: per-tick temperature with seasonal variation, lapse rate from elevation, albedo cooling
- Implemented SimSnapshot in sim-core/src/snapshot.rs for serializable simulation state
- Added `pub use sim::Simulation` re-export in lib.rs
- Created determinism integration tests (3 tests) in tests/determinism_test.rs
  - Same seed produces identical results after 1000 steps
  - Different seeds produce measurably different worlds
  - Step counter advances correctly

## 0.3.0 - 2026-02-24
- Implemented core data structures in sim-core/src/types.rs
  - PlanetParams with AtmosphereState and HydroState (Earth-like defaults)
  - WorldGrid with Tile map and coordinate helpers
  - Species and SpeciesTraits with TrophicLevel enum
  - Intervention system with InterventionKind variants and RegionTarget
  - TimeSpeed enum with steps_per_batch scaling
  - SimEvent enum for lifecycle and game events
- Added `pub use types::*` re-export in lib.rs
- Created integration test suite (5 tests) in tests/types_test.rs

## 0.2.0 - 2026-02-24
- Initialized Rust sim-core crate with Cargo.toml and dependencies
- Created placeholder modules: types, climate, biosphere, sim, level, codex, snapshot
- Added lib.rs with module declarations and main.rs entry point
- Added criterion benchmark scaffold

## 0.1.1 - 2026-02-24
- Project scaffold with full directory structure
- Added 11 design documents (codex, product brief, simulation spec, API spec, data models, UX flow, engineering plan, security, app store page, roadmap, agent prompts)
- Updated README with project overview

## 0.1.0 - 2026-02-24
- Initial project setup
- Added README, .gitignore, and docs structure

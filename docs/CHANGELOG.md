# Changelog

## 0.4.4 - 2026-02-24
### Sprint 4: SimEvent emission for extinction
- `biosphere::update_grid` now returns `Vec<u32>` of species IDs that went globally extinct this tick
- `Simulation::tick()` tracks alive species before/after update, emits `SimEvent::SpeciesExtinct` for each newly extinct species
- Mass extinction detection: emits `SimEvent::MassExtinction` when >50% of living species go extinct in one tick
- New `Simulation::events()` accessor returns `&[SimEvent]`
- All 52 Rust tests passing

## 0.4.3 - 2026-02-24
### Sprint 4: AdjustCurrents + AdjustSalinity interventions
- `AdjustCurrents { delta }` and `AdjustSalinity { delta }` variants added to `InterventionKind`
- `apply_intervention` handles new variants with clamped hydrology parameter updates
- `suitability()`, `carrying_capacity()`, `update_tile_populations()`, `update_grid()` signatures changed from `&AtmosphereState` to `&PlanetParams`
- Salinity suitability factor for ocean tiles: reduces fitness based on salinity vs toxin resistance
- 2 new FFI tests (`test_adjust_currents_intervention`, `test_adjust_salinity_intervention`); 52 total Rust tests passing

## 0.4.2 - 2026-02-24
### Sprint 4: 35 Codex Entries with Extended Unlock Triggers
- New `codex_entries.rs` module with `all_entries()` returning 35 codex entries across 8 categories: Species (8), Body Plans (4), Biomes (5), Planetary Systems (5), Evolutionary Events (5), Failure Modes (4), Rare Phenomena (2), Historic Worlds (2)
- Extended `UnlockTrigger` enum with 10 new variants: `BiomeCondition`, `ParamThreshold`, `PopulationExplosion`, `TrophicCascade`, `StableEcosystem`, `TotalExtinction`, `RunawayGreenhouse`, `FrozenDeath`, `TrophicCollapse`, `Placeholder`
- Each entry includes requirements text, scientific facts, flavor text, related entry cross-references, and SF Symbol icon IDs

## 0.4.1 - 2026-02-24
### Sprint 4: EcosystemStability trophic level evaluation
- `trophic_level_count` function in biosphere.rs counts distinct trophic levels (Producer/Consumer/Predator) with living species
- `EcosystemStability` objective now evaluates against actual trophic level count instead of biodiversity count
- `trophic_levels` field added to objective evaluation JSON result
- New FFI test `test_evaluate_ecosystem_stability_objective` (11 FFI tests total)

## 0.4.0 - 2026-02-24
### Sprint 3: Level System & Training Level 1
- `pa_sim_snapshot_total_biomass` and `pa_sim_evaluate_objective` FFI functions (2 new, 49 total Rust tests)
- `objective_result_cache` on SimHandle for safe C string lifetime management
- Level 1 JSON (`level_01_first_breath.json`) bundled in iOS app resources
- `LevelLoader.swift` with `LevelConfig` Codable model, `paramsJSON` and `objectiveJSON` helpers
- `SimulationEngine` extended with `totalBiomass` property and `evaluateObjective(json:)` method
- `SimulationViewModel` rewritten with level loading, objective tracking, energy budget, and intervention dispatch
- `InterventionTray` UI with horizontal scrolling, SF Symbols, energy cost display, and affordability gating
- `LevelBriefingView` pre-level screen with objective, allowed tools, and energy budget
- `LevelCompleteView` post-level screen with win/fail state, stats, restart/continue actions
- `TutorialOverlay` with step-through tutorial cards (Level 1: 4 steps)
- `PlanetView` rewritten with full level flow: briefing → simulation → tutorial → win/fail overlays
- `LevelSelectView` updated with string-based level IDs for JSON loading
- Objective progress bar with sustained-step tracking
- iOS simulator build verified (BUILD SUCCEEDED)

## 0.3.0 - 2026-02-24
### Sprint 2: iOS FFI Bridge & Visualization
- Rust FFI module with C-compatible API (18 extern functions, SimHandle with cached flat arrays)
- C header (`planet_architect.h`) for iOS bridging
- iOS cross-compilation build script (simulator + device)
- XcodeGen project spec with static library linking
- Swift `SimulationEngine` wrapper with type-safe FFI access
- `SimulationViewModel` with `@Observable` state management and simulation loop (~30fps)
- Canvas-based `GridRenderer` with temperature, nutrient, moisture, and population overlays
- Live `PlanetView` with overlay picker, time controls, and step/biodiversity display
- Tile inspector (tap grid to see tile details in sheet)
- 8 FFI tests (47 total Rust tests passing)
- iOS simulator build verified (BUILD SUCCEEDED)

## 0.2.0 - 2026-02-24
### Sprint 0+1: Foundations & Simulation Core
- Project scaffold with full design documents (11 spec docs)
- Rust sim-core crate with all dependencies
- Core data structures: PlanetParams, WorldGrid, Tile, Species, Interventions
- Deterministic tick loop with ChaCha8 seeded RNG
- Climate model: temperature, greenhouse effect, albedo, seasonal variation, nutrient cycling
- Biosphere model: suitability, logistic growth, carrying capacity, Holling Type II predation
- Mutation and speciation engine with epoch-based checks
- Intervention system: CO2, O2, cloud seeding, nutrient bloom, ice melt
- Level spec JSON format with objective evaluator
- Save/load with deterministic RNG state preservation
- Codex system with entry definitions, unlock triggers, and tracker
- Level 1 ("First Breath") JSON spec with headless integration tests
- Swift bridge types and SwiftUI app shell

## 0.1.0 - 2026-02-24
- Initial project setup

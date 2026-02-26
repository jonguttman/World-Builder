# Changelog

## 0.5.1 - 2026-02-25
### Full-Screen Fix & Level Loading Bug
- Added `UILaunchScreen` to Info.plist via XcodeGen — eliminates black letterbox bars on modern iPhones
- Migrated from `GENERATE_INFOPLIST_FILE` to explicit XcodeGen `info:` section for plist management
- Grid now expands to fill available vertical space with `frame(maxHeight: .infinity)`
- Fixed level JSON decoding failure caused by conflicting `CodingKeys` + `.convertFromSnakeCase` (BUG-001)

## 0.5.0 - 2026-02-25
### Sprint 4: Food Web v0 + Level 2 + Codex v1
- `AdjustCurrents` and `AdjustSalinity` intervention types with salinity suitability factor
- `SpeciesExtinct` and `MassExtinction` SimEvent emission in tick loop
- `EcosystemStability` objective refined to check distinct trophic levels
- 35 codex entries across 8 categories with diverse unlock triggers
- `CodexTracker` with full `check_all` logic (biomes, params, traits, milestones, failures)
- 3 new FFI functions: `pa_sim_codex_all_entries_json`, `pa_sim_codex_unlocked_json`, `pa_sim_codex_new_unlocks_json`
- Level 2 "Shallow Seas" — ocean world with 3-tier food web (Producer, Consumer, Predator)
- Level 2 tutorial (4 steps: food web, currents, oscillations, balance)
- `TabView` app structure with Campaign and Codex tabs
- `CodexView` with category filter chips, search, locked/unlocked entry rows
- `CodexEntryView` detail with facts, flavor text, requirements, and cross-linked related entries
- `CodexStore` with UserDefaults persistence and discovery tracking
- `NewDiscoveriesSheet` shown after level completion with unlocked entries
- 54 Rust tests passing, iOS simulator build verified (BUILD SUCCEEDED)

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

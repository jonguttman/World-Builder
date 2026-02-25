# Changelog

## 0.9.0 - 2026-02-24
- Replaced placeholder PlanetView with live simulation view using GridRenderer
- Added status bar showing current simulation step and biodiversity count
- Integrated overlay picker (temperature, nutrients, moisture, population) via segmented control
- Added time controls with play/pause button and speed picker (1x, 100x, 10K, 1M)
- Added TileInspectorView sheet with tile coordinates, terrain type, and stat readouts
- Tile tap on grid opens inspector with temperature, nutrients, moisture, and population
- PlanetView now accepts seed and paramsJSON parameters for simulation initialization
- Updated LevelSelectView NavigationLink to pass seed to PlanetView

## 0.8.0 - 2026-02-24
- Canvas-based GridRenderer component for drawing 64x32 tile grid
- Temperature overlay with cold-to-hot four-stop color gradient (blue -> cyan -> green/yellow -> red)
- Nutrient and moisture overlays with scalar color interpolation
- Population overlay with ocean/land distinction and intensity-based coloring
- Tap gesture handling with tile coordinate callback via GeometryReader
- Half-pixel overlap on tile rects to eliminate sub-pixel gaps
- Private Comparable.clamped(to:) and Color.components helpers for color blending

## 0.7.0 - 2026-02-24
- SimulationViewModel with @Observable and @MainActor for Swift 6 strict concurrency
- OverlayMode enum for switching between temperature, nutrients, moisture, population views
- Simulation loop running at ~30fps with configurable TimeSpeed batching
- Snapshot refresh pulling grid data from SimulationEngine
- TileInfo struct and tileInfo(x:y:) inspector method
- Level startup with 500-step nutrient warm-up and initial Thermophile species seeding

## 0.6.0 - 2026-02-24
- Swift `SimulationEngine` wrapper class for Rust FFI (`SimulationEngine.swift`)
- Type-safe lifecycle management of opaque `PASimHandle` (failable init, deinit cleanup)
- Swift-native accessors for snapshot data (temperatures, nutrients, moisture, population, ocean mask)
- Species introduction and intervention APIs with JSON bridge
- Save/load support serialising full simulation state to/from `Data`
- Marked `@unchecked Sendable` for actor/queue-based concurrency

## 0.5.0 - 2026-02-24
- XcodeGen project spec (`ios/project.yml`) with Rust static library linking
- Bridging header (`PlanetArchitect-Bridging-Header.h`) including `planet_architect.h`
- iOS 17.0 deployment target, Swift 6.0, strict concurrency enabled
- Library/header search paths configured for both simulator and device architectures
- Added `*.xcodeproj/` to `.gitignore` (regenerated via `xcodegen generate`)

## 0.4.0 - 2026-02-24
- C header file (`sim-core/include/planet_architect.h`) declaring all 18 FFI functions
- iOS cross-compilation build script (`sim-core/build-ios.sh`)
- Verified release builds for `aarch64-apple-ios-sim` (simulator) and `aarch64-apple-ios` (device)

## 0.3.0 - 2026-02-24
- FFI module (`sim-core/src/ffi.rs`) with C-compatible API for iOS bridge
- Opaque `SimHandle` type with cached flat arrays for zero-copy snapshot access
- Lifecycle functions: `pa_sim_create`, `pa_sim_destroy`
- Simulation control: `pa_sim_step`, `pa_sim_current_step`
- Snapshot accessors: temperatures, nutrients, moisture, population, ocean mask, biodiversity, species JSON
- Species management: `pa_sim_add_species_json`
- Intervention support: `pa_sim_apply_intervention_json`
- Save/load via `pa_sim_save_state`, `pa_sim_load_state`, `pa_free_bytes`
- Null-handle safety on all FFI functions
- 8 FFI integration tests (47 total tests passing)

## 0.2.0 - 2026-02-24
- Project scaffold with full design documents (11 spec docs)
- Rust sim-core crate initialized with all dependencies
- Core data structures: PlanetParams, WorldGrid, Tile, Species, Interventions
- Deterministic tick loop with ChaCha8 seeded RNG
- Climate model: temperature, greenhouse effect, albedo, seasonal variation, nutrient cycling
- Biosphere model: suitability calculation, logistic growth, carrying capacity, Holling Type II predation
- Mutation and speciation engine with epoch-based checks
- Intervention system: CO2, O2, cloud seeding, nutrient bloom, ice melt
- Level spec JSON format with objective evaluator (MicrobialStability, EcosystemStability, BiodiversityStability)
- Save/load with deterministic RNG state preservation via bincode
- Codex system with entry definitions, unlock triggers, and tracker
- Level 1 ("First Breath") JSON spec with full headless integration tests
- Swift bridge types mirroring Rust structures (SimTypes.swift)
- SwiftUI app shell: LevelSelectView with training/locked sections, PlanetView with time controls

## 0.1.0 - 2026-02-24
- Initial project setup
- Added README, .gitignore, and docs structure

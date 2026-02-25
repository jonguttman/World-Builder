# Changelog

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

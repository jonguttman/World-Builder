# Planet Architect — Agent Prompts

## Global constraints (ALL agents)
- Deterministic simulation: same seed + same interventions = same outcome
- No direct organism control. Only environment/interventions
- Levels 1–3 free, 4–10 gated $4.99, 11–20 gated $8.99
- Codex is progression (knowledge unlocks)
- MVP: Levels 1–3, Codex v1, StoreKit2, save/load

## Systems Agent
Goal: sim-core mechanics for Levels 1–3
- Data structures, deterministic tick loop, microbe model, speciation rule v0, objective evaluation

## Physics Agent
Goal: Simple, legible climate/atmosphere/hydrology models
- Temperature, greenhouse, albedo, hydrology, seasonality

## UI/UX Agent
Goal: iOS-first minimal UI
- Level Select, Planet View, Recap, Codex, Store flows

## Art/Rendering Agent
Goal: Procedural planet rendering in Metal
- Planet sphere, heatmap overlays, species silhouettes

## Narrative/Codex Agent
Goal: Codex entries + writing system
- 40 initial entries, entry template, level recap copy

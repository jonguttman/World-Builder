# API Spec (iOS to sim-core)

## Core Functions
- init_sim(seed, level_id, initial_planet_params) -> sim_handle
- step(sim_handle, steps)
- apply_intervention(sim_handle, intervention) -> success/fail
- snapshot(sim_handle) -> time, maps, populations, biodiversity, objective_state, codex_unlocks
- save_state/load_state

## Intervention Types (MVP)
- AdjustCO2(delta), AdjustO2(delta), CloudSeeding(region, magnitude), NutrientBloom(region, magnitude), IceMeltPulse(region, magnitude)

## Level Spec Format
level.json: level_id, starting_planet_params, starting_seed, allowed_interventions + cooldowns, energy_budget, objectives

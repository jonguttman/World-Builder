# Simulation Spec (v0.1 MVP)

## Determinism Contract
- All randomness comes from seeded PRNG inside sim-core.
- Inputs are only: initial seed, planet params, level constraints, ordered list of interventions with timestamps
- Output must be identical across devices for the same inputs.

## World Representation
Use a lightweight grid (e.g., 64x32 tiles) representing latitude/longitude bands.

Tile stores:
- elevation, land/ocean flag, temperature, moisture, nutrients, radiation, biome_id (derived), populations by species_id (sparse map)

## Planet Params (Player-controlled)
Physics: gravity, rotation_rate, axial_tilt, core_heat, magnetic_field
Atmosphere: pressure, o2, co2, toxicity
Hydrology: ocean_coverage, salinity, current_strength, ice_fraction

## Climate Model
For each tick:
1) base_temp(lat) = f(insolation, axial_tilt, season_phase)
2) greenhouse = k * log(1 + co2)
3) albedo = mix(ocean_albedo, land_albedo, ice_albedo)
4) temp = base_temp + greenhouse - albedo_cooling + core_heat_term
Clamp temps to safe bounds.

## Nutrient Model
- Increase: volcanism/core_heat, upwelling from currents, decay from dead biomass
- Decrease: producer uptake, leaching

## Biosphere Model (MVP)
Species traits: temp_optimal, temp_range, o2_need, toxin_resistance, trophic_level, reproduction_cost, dispersal, mutation_rate

Population update per tile:
- carrying_capacity = f(nutrients, temp suitability, pressure suitability)
- growth = r * suitability * pop * (1 - pop/capacity)
- mortality = m * (1 - suitability) + predation_pressure

Predation: consumed = a * predator_pop * prey_pop / (1 + h*prey_pop)

## Mutation & Speciation v0
Each epoch: chance mutate based on mutation_rate * stress_factor.
Speciation: geographic isolation + trait divergence threshold for duration D.

## Objectives (Levels 1–3)
Level 1: total microbial biomass > X for 10M years
Level 2: producers + consumers stable for 20M years
Level 3: biodiversity >= N for 50M years + climate variance below threshold

## Time Scaling
Fixed-step: dt = 1 year per step. Speed = steps per frame batch.

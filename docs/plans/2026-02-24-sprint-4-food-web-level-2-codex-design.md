# Sprint 4 Design: Food Web v0 + Level 2 + Codex v1

**Goal:** Add hydrology interventions, author Level 2 ("Shallow Seas") with a 3-tier food web, and ship the full Codex v1 with 35 entries, search/filter UI, and a "New Discoveries" feed.

---

## 1. Simulation Changes

### New Interventions

**`AdjustCurrents { delta: f32 }`**
- Modifies `hydrology.current_strength` by `delta` (clamped 0.0–1.0)
- Stronger currents → more nutrient upwelling on ocean tiles (already wired in `update_nutrients`)

**`AdjustSalinity { delta: f32 }`**
- Modifies `hydrology.salinity` by `delta` (clamped 0.0–1.0)
- New suitability factor on ocean tiles: `suit_salinity = 1.0 - (salinity * (1.0 - toxin_resistance))`
- High salinity penalizes species with low toxin_resistance

### SimEvent Emission

- Emit `SpeciesExtinct { species_id, step }` when a species' global population drops to 0
- Emit `MassExtinction { survivors, step }` when >50% of species go extinct in a single tick
- Both are needed to trigger Codex unlocks

### Codex Integration in sim-core

- Define 35 `CodexEntry` structs in `codex_entries.rs`
- New unlock trigger: `BiomeCondition { tile_criteria, min_tiles }` — checked every 1000 steps
- Hook `CodexTracker::check_*` methods into the tick loop (after biosphere update, alongside speciation check)
- New FFI functions:
  - `pa_sim_codex_unlocked_json(handle)` → JSON array of unlocked entry IDs
  - `pa_sim_codex_new_unlocks_json(handle)` → entries unlocked since last call
  - `pa_sim_codex_all_entries_json(handle)` → all entry definitions

---

## 2. Level 2 — "Shallow Seas"

### Starting World
- Ocean planet: `ocean_coverage: 0.85`, low elevation
- Weak currents: `current_strength: 0.1` (nutrient-poor)
- Mild atmosphere: `o2: 0.10`, `co2: 0.03`, `pressure: 0.8`
- Moderate temperature: `core_heat: 0.4`, `ice_fraction: 0.15`
- `gravity: 8.5`, `rotation_rate: 0.9`, `axial_tilt: 12.0`, `magnetic_field: 0.5`
- `salinity: 0.04`, `toxicity: 0.05`

### Pre-Seeded Species (3)
1. **Planktonic Algae** (Producer) — `temp_optimal: 18.0`, `temp_range: 40.0`, `o2_need: 0.0`, `toxin_resistance: 0.1`, `reproduction_rate: 0.06`, `dispersal: 0.5`, `mutation_rate: 0.01`, initial_pop: 200.0
2. **Grazer** (Consumer) — `temp_optimal: 16.0`, `temp_range: 30.0`, `o2_need: 0.05`, `toxin_resistance: 0.2`, `reproduction_rate: 0.03`, `dispersal: 0.3`, `mutation_rate: 0.008`, initial_pop: 50.0
3. **Apex Filter** (Predator) — `temp_optimal: 17.0`, `temp_range: 25.0`, `o2_need: 0.08`, `toxin_resistance: 0.15`, `reproduction_rate: 0.015`, `dispersal: 0.2`, `mutation_rate: 0.005`, initial_pop: 15.0

### Objective
- Type: `EcosystemStability`
- Requires all 3 trophic levels to have living species (refine FFI evaluator to check distinct trophic levels, not just species count)
- `required_duration_steps: 20_000_000` (20M years)

### Fail Conditions
- Extinction (total biomass = 0)
- Trophic collapse (any trophic level has 0 living species for sustained period)

### Allowed Interventions
`AdjustCurrents`, `NutrientBloom`, `AdjustCO2`, `AdjustO2`, `AdjustSalinity`

### Energy Budget
60.0

### Tutorial (4 steps)
1. "The Food Web" — introduces 3 trophic levels and their dependencies
2. "Ocean Currents" — explains current_strength → nutrient upwelling
3. "Boom and Bust" — warns about Lotka-Volterra oscillations
4. "Balance is Key" — subtle interventions, watch the population overlay

---

## 3. Codex v1 — 35 Entries

### Species (8 entries) — `SpeciesAppeared`
1. Thermophile (L1 starter)
2. Planktonic Algae (L2 starter)
3. Grazer (L2 starter)
4. Apex Filter (L2 starter)
5. First Mutant (first speciation event creates any new species)
6. Deep Dweller (speciation creates species with temp_optimal < 0)
7. Radiation Resistant (speciation creates species with toxin_resistance > 0.6)
8. Cold Adapted (speciation creates species with temp_optimal < -20)

### Body Plans (4 entries) — `TraitStabilized`
9. Chemosynthetic (temp_optimal < -10 stabilized 500K steps)
10. Thermophilic (temp_optimal > 40 stabilized 500K steps)
11. Pressure Adapted (o2_need < 0.02 stabilized 500K steps)
12. Toxin Immune (toxin_resistance > 0.8 stabilized 500K steps)

### Biomes (5 entries) — `BiomeCondition`
13. Hydrothermal Vent (ocean tile + temp > 40 + nutrients > 0.5, min 3 tiles)
14. Frozen Waste (land tile + temp < -30, min 10 tiles)
15. Shallow Reef (ocean tile + temp 10–30 + population > 100, min 5 tiles)
16. Toxic Basin (toxicity > 0.5 + any population present, min 3 tiles)
17. Nutrient Desert (land tile + nutrients < 0.05, min 20 tiles)

### Planetary Systems (5 entries) — param threshold checks
18. Greenhouse World (co2 > 0.2)
19. Oxygenated Atmosphere (o2 > 0.15)
20. Strong Magnetosphere (magnetic_field > 0.7)
21. Ocean World (ocean_coverage > 0.7)
22. Tidal Engine (current_strength > 0.7)

### Evolutionary Events (5 entries) — milestone triggers
23. First Speciation (first `Speciation` SimEvent)
24. Adaptive Radiation (5+ species alive simultaneously)
25. Trophic Cascade (consumer/predator crash → producer boom detected)
26. Population Explosion (any species > 50K global pop)
27. Stable Ecosystem (all 3 trophic levels sustained 1M steps)

### Failure Modes (4 entries) — failure triggers
28. Total Extinction (biomass hits 0)
29. Runaway Greenhouse (average temp > 60)
30. Frozen Death (ice_fraction > 0.9)
31. Trophic Collapse (a trophic level has 0 living species)

### Rare Phenomena (2 entries) — locked preview
32. Silicon Life (placeholder — later levels)
33. Living Crust (placeholder — later levels)

### Historic Worlds (2 entries) — locked preview
34. Earth Analog (placeholder — later levels)
35. Titan Echo (placeholder — later levels)

---

## 4. Codex iOS UI

### App Navigation
Replace root `NavigationStack` with `TabView`:
- Tab 1: **Campaign** (existing LevelSelectView in NavigationStack)
- Tab 2: **Codex** (new CodexView in NavigationStack)

### CodexView (index)
- `List` with sections per category
- Each row: icon, entry name, category tag
- Undiscovered entries: "???" with lock icon, dimmed
- Search bar (filters by name)
- Category filter (horizontal chip picker)
- Badge on tab for unviewed discoveries

### CodexEntryView (detail)
- Name, category, icon
- Facts text (scientific tone)
- Flavor text (poetic, italic)
- Discovery source ("Discovered in: Level X")
- Requirements text
- Related entries (tappable cross-links)

### New Discoveries Sheet
- Shown after level completion if new entries unlocked
- Card-stack: entry name, category, flavor snippet
- "View in Codex" or "Dismiss All"

### Data Flow
- `SimulationEngine` wraps codex FFI methods
- `SimulationViewModel` tracks `newUnlocks` after each run
- `CodexStore` — `@Observable` class persisted to UserDefaults, holds set of unlocked entry IDs across sessions
- Entry definitions loaded once from engine at app launch via `pa_sim_codex_all_entries_json`

---

## 5. EcosystemStability Objective Refinement

The current FFI evaluator for `EcosystemStability` checks `biodiversity >= min_trophic_levels`. This needs refinement to check that **distinct trophic levels** are represented, not just total species count.

New logic:
- Query all living species
- Count distinct trophic levels with at least one living species
- `condition_met = distinct_trophic_levels >= min_trophic_levels`

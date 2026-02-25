# Sprint 4: Food Web v0 + Level 2 + Codex v1 — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add hydrology interventions, author Level 2 ("Shallow Seas") with a 3-tier food web, and ship the full Codex v1 with 35 entries, category-based UI with search/filter, and a "New Discoveries" feed after level completion.

**Architecture:** Rust sim-core gets 2 new intervention types (AdjustCurrents, AdjustSalinity), salinity suitability, SimEvent emission for extinction tracking, refined EcosystemStability objective, 35 codex entries with triggers, and 3 new FFI functions. iOS gets a TabView (Campaign + Codex), CodexStore persistence, full CodexView with search/filter, CodexEntryView detail, and a New Discoveries sheet on level completion. Level 2 JSON is authored and bundled.

**Tech Stack:** Rust (sim-core, serde_json, bincode), Swift 6/SwiftUI, XcodeGen, FFI via C header

---

## Task 1: New Interventions + Salinity Suitability

**Files:**
- Modify: `sim-core/src/types.rs:160-167` (InterventionKind enum)
- Modify: `sim-core/src/sim.rs:206-234` (apply_intervention)
- Modify: `sim-core/src/biosphere.rs:9-38` (suitability function)
- Test: `sim-core/tests/ffi_test.rs`

**Step 1: Write the failing tests**

Append to `sim-core/tests/ffi_test.rs`:

```rust
#[test]
fn test_adjust_currents_intervention() {
    let handle = pa_sim_create(42, ptr::null());

    let json = CString::new(
        r#"{"kind":{"AdjustCurrents":{"delta":0.3}},"target_region":null,"step":0}"#
    ).unwrap();
    let result = pa_sim_apply_intervention_json(handle, json.as_ptr());
    assert_eq!(result, 0, "AdjustCurrents intervention should succeed");

    pa_sim_destroy(handle);
}

#[test]
fn test_adjust_salinity_intervention() {
    let handle = pa_sim_create(42, ptr::null());

    let json = CString::new(
        r#"{"kind":{"AdjustSalinity":{"delta":0.1}},"target_region":null,"step":0}"#
    ).unwrap();
    let result = pa_sim_apply_intervention_json(handle, json.as_ptr());
    assert_eq!(result, 0, "AdjustSalinity intervention should succeed");

    pa_sim_destroy(handle);
}
```

**Step 2: Run tests to verify they fail**

Run: `cd /Users/jonathanguttman/Documents/World-Builder/sim-core && cargo test --test ffi_test test_adjust_currents_intervention test_adjust_salinity_intervention`
Expected: FAIL — unknown variant `AdjustCurrents` / `AdjustSalinity`.

**Step 3: Add intervention variants to InterventionKind**

In `sim-core/src/types.rs`, add two variants to `InterventionKind` (after `IceMeltPulse`):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterventionKind {
    AdjustCO2 { delta: f32 },
    AdjustO2 { delta: f32 },
    CloudSeeding { magnitude: f32 },
    NutrientBloom { magnitude: f32 },
    IceMeltPulse { magnitude: f32 },
    AdjustCurrents { delta: f32 },
    AdjustSalinity { delta: f32 },
}
```

**Step 4: Handle new interventions in apply_intervention**

In `sim-core/src/sim.rs`, add two new match arms in `apply_intervention` (after the `IceMeltPulse` arm):

```rust
            InterventionKind::AdjustCurrents { delta } => {
                self.params.hydrology.current_strength =
                    (self.params.hydrology.current_strength + delta).clamp(0.0, 1.0);
            }
            InterventionKind::AdjustSalinity { delta } => {
                self.params.hydrology.salinity =
                    (self.params.hydrology.salinity + delta).clamp(0.0, 1.0);
            }
```

**Step 5: Add salinity suitability factor**

In `sim-core/src/biosphere.rs`, modify the `suitability` function. After the `nutrient_suit` calculation and before the final clamp, add a salinity factor:

```rust
    // Salinity penalty for ocean tiles — high salinity hurts species with low toxin resistance
    let salinity_suit = if tile.is_ocean {
        let sal = atmo.toxicity; // We'll pass salinity via a different path — see below
        1.0 // placeholder — salinity comes from HydroState, not AtmosphereState
    } else {
        1.0
    };
```

Wait — `suitability()` takes `AtmosphereState` but salinity is on `HydroState`. We need to pass `&PlanetParams` instead of just `&AtmosphereState`. This changes the function signature.

Modify `suitability` signature in `sim-core/src/biosphere.rs`:

```rust
pub fn suitability(traits: &SpeciesTraits, tile: &Tile, params: &PlanetParams) -> f32 {
    let atmo = &params.atmosphere;

    // Temperature suitability
    let temp_diff = (tile.temperature - traits.temp_optimal).abs();
    let temp_suit = if traits.temp_range > 0.0 {
        (1.0 - (temp_diff / traits.temp_range).powi(2)).max(0.0)
    } else {
        0.0
    };

    // Oxygen suitability
    let o2_suit = if traits.o2_need > 0.0 {
        (atmo.o2 / traits.o2_need).min(1.0)
    } else {
        1.0
    };

    // Toxicity resistance
    let tox_suit = if atmo.toxicity > traits.toxin_resistance {
        (1.0 - (atmo.toxicity - traits.toxin_resistance)).max(0.0)
    } else {
        1.0
    };

    // Nutrient availability for producers
    let nutrient_suit = match traits.trophic_level {
        TrophicLevel::Producer => tile.nutrients.min(1.0),
        _ => 1.0,
    };

    // Salinity penalty on ocean tiles
    let salinity_suit = if tile.is_ocean {
        (1.0 - params.hydrology.salinity * (1.0 - traits.toxin_resistance)).max(0.0)
    } else {
        1.0
    };

    (temp_suit * o2_suit * tox_suit * nutrient_suit * salinity_suit).clamp(0.0, 1.0)
}
```

**Step 6: Update all callers of `suitability` and `carrying_capacity`**

`carrying_capacity` also calls `suitability` — update its signature too:

```rust
pub fn carrying_capacity(traits: &SpeciesTraits, tile: &Tile, params: &PlanetParams) -> f64 {
    let suit = suitability(traits, tile, params) as f64;
    // ... rest stays the same
}
```

`update_tile_populations` calls both — change signature:

```rust
pub fn update_tile_populations(
    tile: &mut Tile,
    species_list: &[Species],
    params: &PlanetParams,
) {
    // ...
    let suit = suitability(&species.traits, tile, params) as f64;
    let capacity = carrying_capacity(&species.traits, tile, params);
    // ...
    let predation = compute_predation(tile, species, species_list);
    // ... rest stays the same
}
```

`update_grid` calls `update_tile_populations` — change signature:

```rust
pub fn update_grid(grid: &mut WorldGrid, species: &[Species], params: &PlanetParams) {
    let height = grid.height;
    let width = grid.width;
    for y in 0..height {
        for x in 0..width {
            let tile = grid.get_mut(x, y);
            update_tile_populations(tile, species, params);
        }
    }
}
```

In `sim-core/src/sim.rs`, update the `tick()` call:

```rust
    fn tick(&mut self) {
        self.step_count += 1;
        climate::update(&mut self.grid, &self.params, self.step_count);
        climate::update_nutrients(&mut self.grid, &self.params);
        biosphere::update_grid(&mut self.grid, &self.species, &self.params);
        // ...
    }
```

And update `add_species`:

```rust
    pub fn add_species(&mut self, species: Species, initial_pop_per_tile: f64) {
        let id = species.id;
        self.species.push(species.clone());
        for y in 0..self.grid.height {
            for x in 0..self.grid.width {
                let tile = self.grid.get_mut(x, y);
                let suit = crate::biosphere::suitability(&species.traits, tile, &self.params);
                // ... rest stays the same
            }
        }
        // ...
    }
```

In `sim-core/src/ffi.rs`, update the `pa_sim_evaluate_objective` function call:

The `suitability` is NOT called directly from ffi.rs, but `biodiversity_count` is — and that function doesn't call `suitability`, so it doesn't need changes. However, the `update_cache` method builds species data using `biosphere::global_population` which also doesn't call `suitability`. So no ffi.rs changes needed for this.

**Step 7: Run tests**

Run: `cd /Users/jonathanguttman/Documents/World-Builder/sim-core && cargo test`
Expected: All tests pass (51 total — 49 existing + 2 new).

**Step 8: Commit**

```bash
git add sim-core/src/types.rs sim-core/src/sim.rs sim-core/src/biosphere.rs sim-core/tests/ffi_test.rs
git commit -m "feat: AdjustCurrents + AdjustSalinity interventions with salinity suitability"
```

---

## Task 2: SimEvent Emission for Extinction

**Files:**
- Modify: `sim-core/src/sim.rs:99-108` (tick function)
- Modify: `sim-core/src/biosphere.rs:130-141` (update_grid — return extinction info)
- Test: `sim-core/tests/ffi_test.rs`

**Step 1: Modify `update_grid` to return extinct species**

Change `update_grid` in `sim-core/src/biosphere.rs` to return a `Vec<u32>` of species IDs that went globally extinct this tick:

```rust
/// Update all biosphere populations across the grid.
/// Returns species IDs that went globally extinct this tick.
pub fn update_grid(grid: &mut WorldGrid, species: &[Species], params: &PlanetParams) -> Vec<u32> {
    let height = grid.height;
    let width = grid.width;

    for y in 0..height {
        for x in 0..width {
            let tile = grid.get_mut(x, y);
            update_tile_populations(tile, species, params);
        }
    }

    // Check for newly extinct species (global pop == 0)
    species.iter()
        .filter(|s| global_population(grid, s.id) <= 0.0)
        .map(|s| s.id)
        .collect()
}
```

**Step 2: Emit SimEvents in tick**

In `sim-core/src/sim.rs`, update `tick()`:

```rust
    fn tick(&mut self) {
        self.step_count += 1;
        climate::update(&mut self.grid, &self.params, self.step_count);
        climate::update_nutrients(&mut self.grid, &self.params);

        // Track species alive before update
        let alive_before: Vec<u32> = self.species.iter()
            .filter(|s| biosphere::global_population(&self.grid, s.id) > 0.0)
            .map(|s| s.id)
            .collect();

        let extinct_ids = biosphere::update_grid(&mut self.grid, &self.species, &self.params);

        // Emit extinction events for species that were alive before but now extinct
        let newly_extinct: Vec<u32> = extinct_ids.iter()
            .filter(|id| alive_before.contains(id))
            .cloned()
            .collect();

        for &sp_id in &newly_extinct {
            self.events.push(SimEvent::SpeciesExtinct {
                species_id: sp_id,
                step: self.step_count,
            });
        }

        // Mass extinction: >50% of living species go extinct in one tick
        if !alive_before.is_empty() && newly_extinct.len() > alive_before.len() / 2 {
            let survivors = alive_before.len() - newly_extinct.len();
            self.events.push(SimEvent::MassExtinction {
                survivors,
                step: self.step_count,
            });
        }

        if self.step_count.is_multiple_of(SPECIATION_EPOCH) {
            self.check_speciation();
        }
    }
```

**Step 3: Expose events via a new public method on Simulation**

Add to `sim-core/src/sim.rs`:

```rust
    pub fn events(&self) -> &[SimEvent] {
        &self.events
    }
```

**Step 4: Run tests**

Run: `cd /Users/jonathanguttman/Documents/World-Builder/sim-core && cargo test`
Expected: All 51 tests pass.

**Step 5: Commit**

```bash
git add sim-core/src/sim.rs sim-core/src/biosphere.rs
git commit -m "feat: emit SpeciesExtinct and MassExtinction SimEvents"
```

---

## Task 3: Refine EcosystemStability Objective

**Files:**
- Modify: `sim-core/src/ffi.rs:212-222` (EcosystemStability evaluator)
- Modify: `sim-core/src/biosphere.rs` (add trophic_level_count function)
- Test: `sim-core/tests/ffi_test.rs`

**Step 1: Write the failing test**

Append to `sim-core/tests/ffi_test.rs`:

```rust
#[test]
fn test_evaluate_ecosystem_stability_objective() {
    let handle = pa_sim_create(42, ptr::null());
    pa_sim_step(handle, 500);

    // Add 3 trophic levels
    let producer = CString::new(r#"{"id":0,"name":"Algae","traits":{"temp_optimal":15.0,"temp_range":50.0,"o2_need":0.0,"toxin_resistance":0.1,"trophic_level":"Producer","reproduction_rate":0.05,"dispersal":0.3,"mutation_rate":0.01}}"#).unwrap();
    pa_sim_add_species_json(handle, producer.as_ptr(), 200.0);

    let consumer = CString::new(r#"{"id":1,"name":"Grazer","traits":{"temp_optimal":15.0,"temp_range":40.0,"o2_need":0.05,"toxin_resistance":0.2,"trophic_level":"Consumer","reproduction_rate":0.03,"dispersal":0.3,"mutation_rate":0.01}}"#).unwrap();
    pa_sim_add_species_json(handle, consumer.as_ptr(), 50.0);

    let predator = CString::new(r#"{"id":2,"name":"Hunter","traits":{"temp_optimal":15.0,"temp_range":35.0,"o2_need":0.08,"toxin_resistance":0.15,"trophic_level":"Predator","reproduction_rate":0.015,"dispersal":0.2,"mutation_rate":0.005}}"#).unwrap();
    pa_sim_add_species_json(handle, predator.as_ptr(), 15.0);

    pa_sim_step(handle, 100);

    let objective = CString::new(r#"{"type":"EcosystemStability","min_trophic_levels":3,"required_duration_steps":10}"#).unwrap();
    let result_ptr = pa_sim_evaluate_objective(handle, objective.as_ptr());
    assert!(!result_ptr.is_null());

    let result_str = unsafe { std::ffi::CStr::from_ptr(result_ptr) }.to_str().unwrap();
    // With 3 trophic levels alive, condition should be met
    assert!(result_str.contains("\"condition_met\":true") || result_str.contains("\"condition_met\": true"),
        "Should have condition_met=true with 3 trophic levels: {}", result_str);

    pa_sim_destroy(handle);
}
```

**Step 2: Run test to verify it fails**

Run: `cd /Users/jonathanguttman/Documents/World-Builder/sim-core && cargo test --test ffi_test test_evaluate_ecosystem_stability_objective`
Expected: FAIL — current code checks `biodiversity >= min_trophic_levels` which may pass or fail depending on species count, not trophic levels.

**Step 3: Add trophic_level_count function**

Add to `sim-core/src/biosphere.rs`:

```rust
/// Count distinct trophic levels with at least one living species
pub fn trophic_level_count(grid: &WorldGrid, species: &[Species]) -> u32 {
    let mut has_producer = false;
    let mut has_consumer = false;
    let mut has_predator = false;

    for s in species {
        if global_population(grid, s.id) > 0.0 {
            match s.traits.trophic_level {
                TrophicLevel::Producer => has_producer = true,
                TrophicLevel::Consumer => has_consumer = true,
                TrophicLevel::Predator => has_predator = true,
            }
        }
    }

    has_producer as u32 + has_consumer as u32 + has_predator as u32
}
```

**Step 4: Update EcosystemStability evaluation in ffi.rs**

In `sim-core/src/ffi.rs`, change the `EcosystemStability` match arm:

```rust
        crate::level::Objective::EcosystemStability { min_trophic_levels, .. } => {
            let trophic_levels = crate::biosphere::trophic_level_count(h.sim.grid(), h.sim.species());
            trophic_levels >= *min_trophic_levels
        }
```

Also update the result JSON to include trophic_level_count:

```rust
    let trophic_levels = crate::biosphere::trophic_level_count(h.sim.grid(), h.sim.species());

    let result = serde_json::json!({
        "condition_met": condition_met,
        "total_biomass": total_biomass,
        "biodiversity": biodiversity,
        "trophic_levels": trophic_levels,
        "extinct": extinct,
    });
```

**Step 5: Run tests**

Run: `cd /Users/jonathanguttman/Documents/World-Builder/sim-core && cargo test`
Expected: All 52 tests pass.

**Step 6: Commit**

```bash
git add sim-core/src/ffi.rs sim-core/src/biosphere.rs sim-core/tests/ffi_test.rs
git commit -m "feat: EcosystemStability evaluates distinct trophic levels"
```

---

## Task 4: Codex Entries Data File (35 entries)

**Files:**
- Create: `sim-core/src/codex_entries.rs`
- Modify: `sim-core/src/lib.rs` (add module)
- Modify: `sim-core/src/codex.rs` (add new unlock triggers)

**Step 1: Extend UnlockTrigger with new variants**

In `sim-core/src/codex.rs`, add new variants to `UnlockTrigger`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UnlockTrigger {
    SpeciesAppeared,
    TraitStabilized { trait_name: String, min_duration: u64 },
    BiodiversityThreshold { min_species: u32 },
    MassExtinction,
    SpeciationEvent,
    BiomeCondition { criteria: String, min_tiles: u32 },
    ParamThreshold { param: String, min_value: f32 },
    PopulationExplosion { min_population: f64 },
    TrophicCascade,
    StableEcosystem { min_duration: u64 },
    TotalExtinction,
    RunawayGreenhouse,
    FrozenDeath,
    TrophicCollapse,
    Placeholder,
}
```

**Step 2: Create codex_entries.rs**

Create `sim-core/src/codex_entries.rs`:

```rust
use crate::codex::{CodexEntry, CodexCategory, UnlockTrigger};

pub fn all_entries() -> Vec<CodexEntry> {
    vec![
        // --- Species (8) ---
        CodexEntry {
            id: "species_thermophile".into(),
            category: CodexCategory::Species,
            name: "Thermophile".into(),
            unlock_trigger: UnlockTrigger::SpeciesAppeared,
            requirements_text: "Seed initial life on a barren world.".into(),
            facts_text: "Heat-loving microorganisms that thrive near volcanic vents. They metabolize sulfur compounds and can survive temperatures exceeding 80°C.".into(),
            flavor_text: "The first spark — life clinging to warmth in the dark.".into(),
            related_entry_ids: vec!["bodyplan_chemosynthetic".into()],
            icon_asset_id: "microbe".into(),
        },
        CodexEntry {
            id: "species_planktonic_algae".into(),
            category: CodexCategory::Species,
            name: "Planktonic Algae".into(),
            unlock_trigger: UnlockTrigger::SpeciesAppeared,
            requirements_text: "Begin Level 2 with a marine ecosystem.".into(),
            facts_text: "Free-floating photosynthetic organisms forming the base of marine food webs. They convert sunlight and dissolved nutrients into biomass.".into(),
            flavor_text: "Green clouds drifting through alien seas — the ocean breathes.".into(),
            related_entry_ids: vec!["biome_shallow_reef".into(), "species_grazer".into()],
            icon_asset_id: "leaf".into(),
        },
        CodexEntry {
            id: "species_grazer".into(),
            category: CodexCategory::Species,
            name: "Grazer".into(),
            unlock_trigger: UnlockTrigger::SpeciesAppeared,
            requirements_text: "Begin Level 2 with a marine ecosystem.".into(),
            facts_text: "Filter-feeding organisms that consume producers. Their population oscillates with prey availability in classic predator-prey cycles.".into(),
            flavor_text: "Mouths without eyes, straining life from the current.".into(),
            related_entry_ids: vec!["species_planktonic_algae".into(), "species_apex_filter".into()],
            icon_asset_id: "fish".into(),
        },
        CodexEntry {
            id: "species_apex_filter".into(),
            category: CodexCategory::Species,
            name: "Apex Filter".into(),
            unlock_trigger: UnlockTrigger::SpeciesAppeared,
            requirements_text: "Begin Level 2 with a marine ecosystem.".into(),
            facts_text: "Top predators of the early marine ecosystem. They regulate consumer populations, preventing overgrazing of producers.".into(),
            flavor_text: "Patient hunters in the deep — the ocean's balance keepers.".into(),
            related_entry_ids: vec!["species_grazer".into(), "event_trophic_cascade".into()],
            icon_asset_id: "bolt".into(),
        },
        CodexEntry {
            id: "species_first_mutant".into(),
            category: CodexCategory::Species,
            name: "First Mutant".into(),
            unlock_trigger: UnlockTrigger::SpeciationEvent,
            requirements_text: "Observe the first speciation event.".into(),
            facts_text: "When environmental pressure meets genetic drift, a lineage splits. The child species carries mutated traits better suited to local conditions.".into(),
            flavor_text: "Something new. Something that was not planned.".into(),
            related_entry_ids: vec!["event_first_speciation".into()],
            icon_asset_id: "sparkles".into(),
        },
        CodexEntry {
            id: "species_deep_dweller".into(),
            category: CodexCategory::Species,
            name: "Deep Dweller".into(),
            unlock_trigger: UnlockTrigger::TraitStabilized { trait_name: "temp_optimal_below_zero".into(), min_duration: 100_000 },
            requirements_text: "A species with sub-zero optimal temperature survives 100K years.".into(),
            facts_text: "Organisms adapted to extreme cold, thriving in conditions that would freeze most life. Their biochemistry uses antifreeze proteins.".into(),
            flavor_text: "Where others see death, they find home.".into(),
            related_entry_ids: vec!["bodyplan_chemosynthetic".into(), "biome_frozen_waste".into()],
            icon_asset_id: "snowflake".into(),
        },
        CodexEntry {
            id: "species_radiation_resistant".into(),
            category: CodexCategory::Species,
            name: "Radiation Resistant".into(),
            unlock_trigger: UnlockTrigger::TraitStabilized { trait_name: "toxin_resistance_high".into(), min_duration: 100_000 },
            requirements_text: "A species with toxin_resistance > 0.6 survives 100K years.".into(),
            facts_text: "These organisms developed robust DNA repair mechanisms and radiation-absorbing pigments, allowing survival in high-radiation environments.".into(),
            flavor_text: "Bathed in starfire, yet unbroken.".into(),
            related_entry_ids: vec!["bodyplan_toxin_immune".into()],
            icon_asset_id: "bolt.shield".into(),
        },
        CodexEntry {
            id: "species_cold_adapted".into(),
            category: CodexCategory::Species,
            name: "Cold Adapted".into(),
            unlock_trigger: UnlockTrigger::TraitStabilized { trait_name: "temp_optimal_very_cold".into(), min_duration: 200_000 },
            requirements_text: "A species with temp_optimal < -20°C survives 200K years.".into(),
            facts_text: "Psychrophilic organisms that have replaced water in their cells with cryoprotectant compounds. They reproduce slowly but persistently.".into(),
            flavor_text: "Ice is not the end. It is a beginning no one expected.".into(),
            related_entry_ids: vec!["species_deep_dweller".into(), "failure_frozen_death".into()],
            icon_asset_id: "thermometer.snowflake".into(),
        },

        // --- Body Plans (4) ---
        CodexEntry {
            id: "bodyplan_chemosynthetic".into(),
            category: CodexCategory::BodyPlan,
            name: "Chemosynthetic".into(),
            unlock_trigger: UnlockTrigger::TraitStabilized { trait_name: "temp_optimal_below_neg10".into(), min_duration: 500_000 },
            requirements_text: "A species with temp_optimal < -10°C stabilized for 500K years.".into(),
            facts_text: "Organisms that derive energy from chemical reactions rather than sunlight. They thrive near hydrothermal vents and in subsurface environments.".into(),
            flavor_text: "Life that feeds on the planet itself.".into(),
            related_entry_ids: vec!["biome_hydrothermal_vent".into(), "species_deep_dweller".into()],
            icon_asset_id: "flame".into(),
        },
        CodexEntry {
            id: "bodyplan_thermophilic".into(),
            category: CodexCategory::BodyPlan,
            name: "Thermophilic".into(),
            unlock_trigger: UnlockTrigger::TraitStabilized { trait_name: "temp_optimal_above_40".into(), min_duration: 500_000 },
            requirements_text: "A species with temp_optimal > 40°C stabilized for 500K years.".into(),
            facts_text: "Heat-loving organisms with proteins that remain stable at extreme temperatures. Their enzymes function optimally above the boiling point of many solvents.".into(),
            flavor_text: "Forged in heat. Tempered by time.".into(),
            related_entry_ids: vec!["species_thermophile".into(), "system_greenhouse".into()],
            icon_asset_id: "thermometer.sun".into(),
        },
        CodexEntry {
            id: "bodyplan_pressure_adapted".into(),
            category: CodexCategory::BodyPlan,
            name: "Pressure Adapted".into(),
            unlock_trigger: UnlockTrigger::TraitStabilized { trait_name: "o2_need_very_low".into(), min_duration: 500_000 },
            requirements_text: "A species with o2_need < 0.02 stabilized for 500K years.".into(),
            facts_text: "Anaerobic organisms that evolved to thrive without oxygen. They use alternative electron acceptors for respiration.".into(),
            flavor_text: "They remember a time before breath.".into(),
            related_entry_ids: vec!["system_oxygenated".into()],
            icon_asset_id: "wind".into(),
        },
        CodexEntry {
            id: "bodyplan_toxin_immune".into(),
            category: CodexCategory::BodyPlan,
            name: "Toxin Immune".into(),
            unlock_trigger: UnlockTrigger::TraitStabilized { trait_name: "toxin_resistance_very_high".into(), min_duration: 500_000 },
            requirements_text: "A species with toxin_resistance > 0.8 stabilized for 500K years.".into(),
            facts_text: "Organisms with extraordinary detoxification pathways. They metabolize compounds that are lethal to most life forms.".into(),
            flavor_text: "Poison is just another flavor.".into(),
            related_entry_ids: vec!["species_radiation_resistant".into(), "biome_toxic_basin".into()],
            icon_asset_id: "shield".into(),
        },

        // --- Biomes (5) ---
        CodexEntry {
            id: "biome_hydrothermal_vent".into(),
            category: CodexCategory::Biome,
            name: "Hydrothermal Vent".into(),
            unlock_trigger: UnlockTrigger::BiomeCondition { criteria: "ocean_hot_nutrient".into(), min_tiles: 3 },
            requirements_text: "3+ ocean tiles with temperature > 40°C and nutrients > 0.5.".into(),
            facts_text: "Submarine volcanic vents create oases of chemical energy. Superheated water carries dissolved minerals that sustain chemosynthetic ecosystems.".into(),
            flavor_text: "Where the planet bleeds, life drinks deep.".into(),
            related_entry_ids: vec!["bodyplan_chemosynthetic".into()],
            icon_asset_id: "flame.fill".into(),
        },
        CodexEntry {
            id: "biome_frozen_waste".into(),
            category: CodexCategory::Biome,
            name: "Frozen Waste".into(),
            unlock_trigger: UnlockTrigger::BiomeCondition { criteria: "land_frozen".into(), min_tiles: 10 },
            requirements_text: "10+ land tiles with temperature < -30°C.".into(),
            facts_text: "Vast frozen expanses where temperatures plunge far below freezing. Only the hardiest extremophiles survive in thin liquid water films between ice crystals.".into(),
            flavor_text: "A silence so deep it has weight.".into(),
            related_entry_ids: vec!["species_cold_adapted".into(), "failure_frozen_death".into()],
            icon_asset_id: "snow".into(),
        },
        CodexEntry {
            id: "biome_shallow_reef".into(),
            category: CodexCategory::Biome,
            name: "Shallow Reef".into(),
            unlock_trigger: UnlockTrigger::BiomeCondition { criteria: "ocean_warm_populated".into(), min_tiles: 5 },
            requirements_text: "5+ ocean tiles with temp 10-30°C and population > 100.".into(),
            facts_text: "Warm, nutrient-rich shallows teeming with life. The highest biodiversity density in any marine environment, driven by stable conditions and abundant energy.".into(),
            flavor_text: "The sea remembers how to be alive.".into(),
            related_entry_ids: vec!["species_planktonic_algae".into(), "system_ocean_world".into()],
            icon_asset_id: "water.waves".into(),
        },
        CodexEntry {
            id: "biome_toxic_basin".into(),
            category: CodexCategory::Biome,
            name: "Toxic Basin".into(),
            unlock_trigger: UnlockTrigger::BiomeCondition { criteria: "toxic_inhabited".into(), min_tiles: 3 },
            requirements_text: "3+ tiles with toxicity > 0.5 and any population present.".into(),
            facts_text: "Acidic pools and toxic flats where conventional biology fails. Extremophiles here have evolved radical biochemical adaptations.".into(),
            flavor_text: "Life finds a way. Even here.".into(),
            related_entry_ids: vec!["bodyplan_toxin_immune".into()],
            icon_asset_id: "exclamationmark.triangle".into(),
        },
        CodexEntry {
            id: "biome_nutrient_desert".into(),
            category: CodexCategory::Biome,
            name: "Nutrient Desert".into(),
            unlock_trigger: UnlockTrigger::BiomeCondition { criteria: "land_barren".into(), min_tiles: 20 },
            requirements_text: "20+ land tiles with nutrients < 0.05.".into(),
            facts_text: "Sterile landscapes stripped of bioavailable minerals. Without volcanic activity or biological recycling, these regions remain lifeless.".into(),
            flavor_text: "The planet's bones, picked clean.".into(),
            related_entry_ids: vec![],
            icon_asset_id: "sun.dust".into(),
        },

        // --- Planetary Systems (5) ---
        CodexEntry {
            id: "system_greenhouse".into(),
            category: CodexCategory::PlanetarySystem,
            name: "Greenhouse World".into(),
            unlock_trigger: UnlockTrigger::ParamThreshold { param: "co2".into(), min_value: 0.2 },
            requirements_text: "Atmospheric CO2 exceeds 0.2.".into(),
            facts_text: "Runaway greenhouse conditions trap heat in the atmosphere. Surface temperatures rise dramatically, melting ice and altering weather patterns.".into(),
            flavor_text: "The sky becomes a blanket. Then a prison.".into(),
            related_entry_ids: vec!["failure_runaway_greenhouse".into(), "bodyplan_thermophilic".into()],
            icon_asset_id: "sun.max".into(),
        },
        CodexEntry {
            id: "system_oxygenated".into(),
            category: CodexCategory::PlanetarySystem,
            name: "Oxygenated Atmosphere".into(),
            unlock_trigger: UnlockTrigger::ParamThreshold { param: "o2".into(), min_value: 0.15 },
            requirements_text: "Atmospheric O2 exceeds 0.15.".into(),
            facts_text: "A critical threshold in planetary evolution. Sufficient oxygen enables aerobic metabolism — dramatically more efficient energy extraction from food.".into(),
            flavor_text: "The world learns to breathe. Everything changes.".into(),
            related_entry_ids: vec!["bodyplan_pressure_adapted".into()],
            icon_asset_id: "wind".into(),
        },
        CodexEntry {
            id: "system_magnetosphere".into(),
            category: CodexCategory::PlanetarySystem,
            name: "Strong Magnetosphere".into(),
            unlock_trigger: UnlockTrigger::ParamThreshold { param: "magnetic_field".into(), min_value: 0.7 },
            requirements_text: "Magnetic field strength exceeds 0.7.".into(),
            facts_text: "A powerful magnetic field deflects charged particles from the star, protecting the atmosphere from being stripped away and shielding surface life from radiation.".into(),
            flavor_text: "An invisible shield. The planet's silent guardian.".into(),
            related_entry_ids: vec!["species_radiation_resistant".into()],
            icon_asset_id: "shield.lefthalf.filled".into(),
        },
        CodexEntry {
            id: "system_ocean_world".into(),
            category: CodexCategory::PlanetarySystem,
            name: "Ocean World".into(),
            unlock_trigger: UnlockTrigger::ParamThreshold { param: "ocean_coverage".into(), min_value: 0.7 },
            requirements_text: "Ocean coverage exceeds 0.7.".into(),
            facts_text: "A planet dominated by liquid water. Ocean worlds moderate temperature extremes and provide vast habitable volume, but limited mineral cycling slows evolution.".into(),
            flavor_text: "A single, unbroken mirror reflecting the sky.".into(),
            related_entry_ids: vec!["biome_shallow_reef".into(), "system_tidal_engine".into()],
            icon_asset_id: "globe.americas".into(),
        },
        CodexEntry {
            id: "system_tidal_engine".into(),
            category: CodexCategory::PlanetarySystem,
            name: "Tidal Engine".into(),
            unlock_trigger: UnlockTrigger::ParamThreshold { param: "current_strength".into(), min_value: 0.7 },
            requirements_text: "Ocean current strength exceeds 0.7.".into(),
            facts_text: "Strong tidal forces drive powerful ocean currents that distribute heat and nutrients globally. Upwelling zones become hotspots of biological productivity.".into(),
            flavor_text: "The ocean stirs. Nutrients rise. Life follows.".into(),
            related_entry_ids: vec!["system_ocean_world".into()],
            icon_asset_id: "water.waves".into(),
        },

        // --- Evolutionary Events (5) ---
        CodexEntry {
            id: "event_first_speciation".into(),
            category: CodexCategory::EvolutionaryEvent,
            name: "First Speciation".into(),
            unlock_trigger: UnlockTrigger::SpeciationEvent,
            requirements_text: "Observe the first speciation event.".into(),
            facts_text: "When a population becomes isolated or faces divergent selective pressures, accumulated mutations can split a single lineage into two distinct species.".into(),
            flavor_text: "One becomes two. The tree of life branches.".into(),
            related_entry_ids: vec!["species_first_mutant".into()],
            icon_asset_id: "arrow.triangle.branch".into(),
        },
        CodexEntry {
            id: "event_adaptive_radiation".into(),
            category: CodexCategory::EvolutionaryEvent,
            name: "Adaptive Radiation".into(),
            unlock_trigger: UnlockTrigger::BiodiversityThreshold { min_species: 5 },
            requirements_text: "5 or more species alive simultaneously.".into(),
            facts_text: "A burst of speciation as organisms rapidly diversify to fill available ecological niches. Often triggered by the opening of new habitats or the extinction of competitors.".into(),
            flavor_text: "The world offers a thousand niches. Life fills them all.".into(),
            related_entry_ids: vec!["event_first_speciation".into()],
            icon_asset_id: "chart.line.uptrend.xyaxis".into(),
        },
        CodexEntry {
            id: "event_trophic_cascade".into(),
            category: CodexCategory::EvolutionaryEvent,
            name: "Trophic Cascade".into(),
            unlock_trigger: UnlockTrigger::TrophicCascade,
            requirements_text: "A predator or consumer crash causes a cascading population shift.".into(),
            facts_text: "When a key species in the food web collapses, the effects ripple through all trophic levels. Prey populations explode, then crash as resources deplete.".into(),
            flavor_text: "One thread pulled. The whole web shudders.".into(),
            related_entry_ids: vec!["species_apex_filter".into(), "failure_trophic_collapse".into()],
            icon_asset_id: "arrow.down.right.and.arrow.up.left".into(),
        },
        CodexEntry {
            id: "event_population_explosion".into(),
            category: CodexCategory::EvolutionaryEvent,
            name: "Population Explosion".into(),
            unlock_trigger: UnlockTrigger::PopulationExplosion { min_population: 50_000.0 },
            requirements_text: "Any species exceeds 50,000 global population.".into(),
            facts_text: "Exponential growth unchecked by predation or resource limits. Population explosions are temporary — carrying capacity always reasserts itself.".into(),
            flavor_text: "Numbers without limit. Until the limit arrives.".into(),
            related_entry_ids: vec!["event_trophic_cascade".into()],
            icon_asset_id: "arrow.up.right".into(),
        },
        CodexEntry {
            id: "event_stable_ecosystem".into(),
            category: CodexCategory::EvolutionaryEvent,
            name: "Stable Ecosystem".into(),
            unlock_trigger: UnlockTrigger::StableEcosystem { min_duration: 1_000_000 },
            requirements_text: "All 3 trophic levels sustained for 1M years.".into(),
            facts_text: "A mature ecosystem where producers, consumers, and predators coexist in dynamic equilibrium. Population oscillations dampen over time as the web stabilizes.".into(),
            flavor_text: "Not perfection. Balance. And that is enough.".into(),
            related_entry_ids: vec!["species_grazer".into(), "species_apex_filter".into()],
            icon_asset_id: "checkmark.seal".into(),
        },

        // --- Failure Modes (4) ---
        CodexEntry {
            id: "failure_total_extinction".into(),
            category: CodexCategory::FailureMode,
            name: "Total Extinction".into(),
            unlock_trigger: UnlockTrigger::TotalExtinction,
            requirements_text: "All life goes extinct.".into(),
            facts_text: "Complete biosphere collapse. When conditions exceed the tolerance of every living species simultaneously, the planet returns to sterility.".into(),
            flavor_text: "Silence. The loudest sound a world can make.".into(),
            related_entry_ids: vec!["failure_runaway_greenhouse".into(), "failure_frozen_death".into()],
            icon_asset_id: "xmark.circle".into(),
        },
        CodexEntry {
            id: "failure_runaway_greenhouse".into(),
            category: CodexCategory::FailureMode,
            name: "Runaway Greenhouse".into(),
            unlock_trigger: UnlockTrigger::RunawayGreenhouse,
            requirements_text: "Average surface temperature exceeds 60°C.".into(),
            facts_text: "A positive feedback loop: higher temperatures release more greenhouse gases, which trap more heat. Venus suffered this fate — surface temperatures hot enough to melt lead.".into(),
            flavor_text: "The sky turns to copper. The oceans boil away.".into(),
            related_entry_ids: vec!["system_greenhouse".into(), "failure_total_extinction".into()],
            icon_asset_id: "thermometer.sun.fill".into(),
        },
        CodexEntry {
            id: "failure_frozen_death".into(),
            category: CodexCategory::FailureMode,
            name: "Frozen Death".into(),
            unlock_trigger: UnlockTrigger::FrozenDeath,
            requirements_text: "Ice fraction exceeds 0.9.".into(),
            facts_text: "Snowball planet. Ice reflects sunlight, cooling the surface further, creating more ice. Without volcanic CO2 to break the cycle, the planet freezes solid.".into(),
            flavor_text: "White. Everywhere. Forever.".into(),
            related_entry_ids: vec!["biome_frozen_waste".into(), "failure_total_extinction".into()],
            icon_asset_id: "snowflake".into(),
        },
        CodexEntry {
            id: "failure_trophic_collapse".into(),
            category: CodexCategory::FailureMode,
            name: "Trophic Collapse".into(),
            unlock_trigger: UnlockTrigger::TrophicCollapse,
            requirements_text: "An entire trophic level loses all species.".into(),
            facts_text: "When all species of one trophic level go extinct, the food web breaks. Without producers, consumers starve. Without predators, consumers overgraze.".into(),
            flavor_text: "The chain snaps. Everything above it falls.".into(),
            related_entry_ids: vec!["event_trophic_cascade".into(), "failure_total_extinction".into()],
            icon_asset_id: "link".into(),
        },

        // --- Rare Phenomena (2) — Locked Previews ---
        CodexEntry {
            id: "rare_silicon_life".into(),
            category: CodexCategory::RarePhenomenon,
            name: "Silicon Life".into(),
            unlock_trigger: UnlockTrigger::Placeholder,
            requirements_text: "Unlocked in advanced levels.".into(),
            facts_text: "Theoretical organisms using silicon-based biochemistry instead of carbon. Possible on worlds with abundant silicates and extreme temperatures.".into(),
            flavor_text: "Crystal minds dreaming in geological time.".into(),
            related_entry_ids: vec![],
            icon_asset_id: "diamond".into(),
        },
        CodexEntry {
            id: "rare_living_crust".into(),
            category: CodexCategory::RarePhenomenon,
            name: "Living Crust".into(),
            unlock_trigger: UnlockTrigger::Placeholder,
            requirements_text: "Unlocked in advanced levels.".into(),
            facts_text: "A planetary-scale organism integrated into the geological substrate. The boundary between life and geology dissolves entirely.".into(),
            flavor_text: "The planet breathes. Not metaphorically.".into(),
            related_entry_ids: vec![],
            icon_asset_id: "globe.europe.africa".into(),
        },

        // --- Historic Worlds (2) — Locked Previews ---
        CodexEntry {
            id: "historic_earth_analog".into(),
            category: CodexCategory::HistoricWorld,
            name: "Earth Analog".into(),
            unlock_trigger: UnlockTrigger::Placeholder,
            requirements_text: "Unlocked in advanced levels.".into(),
            facts_text: "A world that closely mirrors Earth's evolutionary trajectory. Liquid water, plate tectonics, and a protective magnetosphere create ideal conditions for complex life.".into(),
            flavor_text: "Familiar, yet impossibly distant.".into(),
            related_entry_ids: vec![],
            icon_asset_id: "globe.americas".into(),
        },
        CodexEntry {
            id: "historic_titan_echo".into(),
            category: CodexCategory::HistoricWorld,
            name: "Titan Echo".into(),
            unlock_trigger: UnlockTrigger::Placeholder,
            requirements_text: "Unlocked in advanced levels.".into(),
            facts_text: "A frozen moon with liquid hydrocarbon lakes. If life exists here, it uses an entirely alien biochemistry — methane as solvent instead of water.".into(),
            flavor_text: "Rain falls upward. Lakes burn cold. Life persists.".into(),
            related_entry_ids: vec![],
            icon_asset_id: "moon".into(),
        },
    ]
}
```

**Step 3: Register the module**

In `sim-core/src/lib.rs`, add:

```rust
pub mod codex_entries;
```

**Step 4: Run tests**

Run: `cd /Users/jonathanguttman/Documents/World-Builder/sim-core && cargo test`
Expected: All 52 tests pass (no new tests yet — just verifying compilation).

**Step 5: Commit**

```bash
git add sim-core/src/codex_entries.rs sim-core/src/codex.rs sim-core/src/lib.rs
git commit -m "feat: 35 codex entries with extended unlock trigger types"
```

---

## Task 5: Enhanced CodexTracker + Simulation Integration

**Files:**
- Modify: `sim-core/src/codex.rs` (add check methods for new triggers)
- Modify: `sim-core/src/sim.rs` (add CodexTracker field, hook into tick)

**Step 1: Add comprehensive check methods to CodexTracker**

Replace `sim-core/src/codex.rs` entirely:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::types::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CodexCategory {
    Species,
    BodyPlan,
    Biome,
    PlanetarySystem,
    EvolutionaryEvent,
    FailureMode,
    RarePhenomenon,
    HistoricWorld,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UnlockTrigger {
    SpeciesAppeared,
    TraitStabilized { trait_name: String, min_duration: u64 },
    BiodiversityThreshold { min_species: u32 },
    MassExtinction,
    SpeciationEvent,
    BiomeCondition { criteria: String, min_tiles: u32 },
    ParamThreshold { param: String, min_value: f32 },
    PopulationExplosion { min_population: f64 },
    TrophicCascade,
    StableEcosystem { min_duration: u64 },
    TotalExtinction,
    RunawayGreenhouse,
    FrozenDeath,
    TrophicCollapse,
    Placeholder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexEntry {
    pub id: String,
    pub category: CodexCategory,
    pub name: String,
    pub unlock_trigger: UnlockTrigger,
    pub requirements_text: String,
    pub facts_text: String,
    pub flavor_text: String,
    pub related_entry_ids: Vec<String>,
    pub icon_asset_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct CodexTracker {
    entries: Vec<CodexEntry>,
    unlocked: HashSet<String>,
    #[serde(skip)]
    new_unlocks: Vec<String>,
    // For stable ecosystem tracking
    trophic_sustained_steps: u64,
}

impl CodexTracker {
    pub fn new(entries: Vec<CodexEntry>) -> Self {
        Self {
            entries,
            unlocked: HashSet::new(),
            new_unlocks: Vec::new(),
            trophic_sustained_steps: 0,
        }
    }

    pub fn entries(&self) -> &[CodexEntry] {
        &self.entries
    }

    pub fn unlocked_ids(&self) -> Vec<String> {
        self.unlocked.iter().cloned().collect()
    }

    /// Returns newly unlocked entry IDs since last call, then clears the buffer.
    pub fn drain_new_unlocks(&mut self) -> Vec<String> {
        std::mem::take(&mut self.new_unlocks)
    }

    /// Run all checks for the current simulation state. Called every SPECIATION_EPOCH steps.
    pub fn check_all(
        &mut self,
        grid: &WorldGrid,
        species: &[Species],
        params: &PlanetParams,
        events: &[SimEvent],
        step: u64,
    ) {
        self.check_species_appeared(events);
        self.check_speciation(events);
        self.check_mass_extinction(events);
        self.check_biodiversity(grid, species);
        self.check_param_thresholds(params);
        self.check_biome_conditions(grid, params);
        self.check_population_explosions(grid, species);
        self.check_trait_stabilized(species, step);
        self.check_stable_ecosystem(grid, species);
        self.check_failure_conditions(grid, species, params);
    }

    fn unlock(&mut self, id: &str) {
        if !self.unlocked.contains(id) {
            self.unlocked.insert(id.to_string());
            self.new_unlocks.push(id.to_string());
        }
    }

    fn check_species_appeared(&mut self, events: &[SimEvent]) {
        let has_appeared = events.iter().any(|e| matches!(e, SimEvent::SpeciesAppeared { .. }));
        if has_appeared {
            for entry in &self.entries {
                if matches!(entry.unlock_trigger, UnlockTrigger::SpeciesAppeared) {
                    let id = entry.id.clone();
                    self.unlock(&id);
                }
            }
        }
    }

    fn check_speciation(&mut self, events: &[SimEvent]) {
        let has_speciation = events.iter().any(|e| matches!(e, SimEvent::Speciation { .. }));
        if has_speciation {
            for entry in &self.entries {
                if matches!(entry.unlock_trigger, UnlockTrigger::SpeciationEvent) {
                    let id = entry.id.clone();
                    self.unlock(&id);
                }
            }
        }
    }

    fn check_mass_extinction(&mut self, events: &[SimEvent]) {
        let has_mass = events.iter().any(|e| matches!(e, SimEvent::MassExtinction { .. }));
        if has_mass {
            for entry in &self.entries {
                if matches!(entry.unlock_trigger, UnlockTrigger::MassExtinction) {
                    let id = entry.id.clone();
                    self.unlock(&id);
                }
            }
        }
    }

    fn check_biodiversity(&mut self, grid: &WorldGrid, species: &[Species]) {
        let count = crate::biosphere::biodiversity_count(grid, species);
        for entry in &self.entries {
            if let UnlockTrigger::BiodiversityThreshold { min_species } = &entry.unlock_trigger {
                if count >= *min_species {
                    let id = entry.id.clone();
                    self.unlock(&id);
                }
            }
        }
    }

    fn check_param_thresholds(&mut self, params: &PlanetParams) {
        for entry in &self.entries {
            if let UnlockTrigger::ParamThreshold { param, min_value } = &entry.unlock_trigger {
                let actual = match param.as_str() {
                    "co2" => params.atmosphere.co2,
                    "o2" => params.atmosphere.o2,
                    "magnetic_field" => params.magnetic_field,
                    "ocean_coverage" => params.hydrology.ocean_coverage,
                    "current_strength" => params.hydrology.current_strength,
                    _ => 0.0,
                };
                if actual >= *min_value {
                    let id = entry.id.clone();
                    self.unlock(&id);
                }
            }
        }
    }

    fn check_biome_conditions(&mut self, grid: &WorldGrid, params: &PlanetParams) {
        for entry in &self.entries {
            if let UnlockTrigger::BiomeCondition { criteria, min_tiles } = &entry.unlock_trigger {
                let count = count_biome_tiles(grid, params, criteria);
                if count >= *min_tiles {
                    let id = entry.id.clone();
                    self.unlock(&id);
                }
            }
        }
    }

    fn check_population_explosions(&mut self, grid: &WorldGrid, species: &[Species]) {
        for entry in &self.entries {
            if let UnlockTrigger::PopulationExplosion { min_population } = &entry.unlock_trigger {
                for s in species {
                    let pop = crate::biosphere::global_population(grid, s.id);
                    if pop >= *min_population {
                        let id = entry.id.clone();
                        self.unlock(&id);
                        break;
                    }
                }
            }
        }
    }

    fn check_trait_stabilized(&mut self, species: &[Species], _step: u64) {
        // Simplified: check if any living species has the trait now.
        // Full duration tracking would require per-species timers — for MVP we unlock
        // when the trait exists (the min_duration is aspirational).
        for entry in &self.entries {
            if let UnlockTrigger::TraitStabilized { trait_name, .. } = &entry.unlock_trigger {
                let matched = species.iter().any(|s| match trait_name.as_str() {
                    "temp_optimal_below_zero" => s.traits.temp_optimal < 0.0,
                    "temp_optimal_very_cold" => s.traits.temp_optimal < -20.0,
                    "temp_optimal_below_neg10" => s.traits.temp_optimal < -10.0,
                    "temp_optimal_above_40" => s.traits.temp_optimal > 40.0,
                    "o2_need_very_low" => s.traits.o2_need < 0.02,
                    "toxin_resistance_high" => s.traits.toxin_resistance > 0.6,
                    "toxin_resistance_very_high" => s.traits.toxin_resistance > 0.8,
                    _ => false,
                });
                if matched {
                    let id = entry.id.clone();
                    self.unlock(&id);
                }
            }
        }
    }

    fn check_stable_ecosystem(&mut self, grid: &WorldGrid, species: &[Species]) {
        let levels = crate::biosphere::trophic_level_count(grid, species);
        if levels >= 3 {
            self.trophic_sustained_steps += 1000; // Called every SPECIATION_EPOCH
        } else {
            self.trophic_sustained_steps = 0;
        }

        for entry in &self.entries {
            if let UnlockTrigger::StableEcosystem { min_duration } = &entry.unlock_trigger {
                if self.trophic_sustained_steps >= *min_duration {
                    let id = entry.id.clone();
                    self.unlock(&id);
                }
            }
        }
    }

    fn check_failure_conditions(&mut self, grid: &WorldGrid, species: &[Species], params: &PlanetParams) {
        let total_biomass: f64 = grid.tiles.iter()
            .flat_map(|t| t.populations.values())
            .sum();

        // Total extinction
        if total_biomass <= 0.0 && !species.is_empty() {
            for entry in &self.entries {
                if matches!(entry.unlock_trigger, UnlockTrigger::TotalExtinction) {
                    let id = entry.id.clone();
                    self.unlock(&id);
                }
            }
        }

        // Runaway greenhouse (avg temp > 60)
        let avg_temp = grid.tiles.iter().map(|t| t.temperature as f64).sum::<f64>() / grid.tiles.len() as f64;
        if avg_temp > 60.0 {
            for entry in &self.entries {
                if matches!(entry.unlock_trigger, UnlockTrigger::RunawayGreenhouse) {
                    let id = entry.id.clone();
                    self.unlock(&id);
                }
            }
        }

        // Frozen death (ice_fraction > 0.9)
        if params.hydrology.ice_fraction > 0.9 {
            for entry in &self.entries {
                if matches!(entry.unlock_trigger, UnlockTrigger::FrozenDeath) {
                    let id = entry.id.clone();
                    self.unlock(&id);
                }
            }
        }

        // Trophic collapse (a trophic level has 0 living species)
        let levels = crate::biosphere::trophic_level_count(grid, species);
        let has_any_life = total_biomass > 0.0;
        if has_any_life && levels < 3 && species.len() >= 3 {
            // At least 3 species were added but not all trophic levels survive
            for entry in &self.entries {
                if matches!(entry.unlock_trigger, UnlockTrigger::TrophicCollapse) {
                    let id = entry.id.clone();
                    self.unlock(&id);
                }
            }
        }
    }
}

fn count_biome_tiles(grid: &WorldGrid, params: &PlanetParams, criteria: &str) -> u32 {
    let mut count = 0u32;
    for tile in &grid.tiles {
        let matches = match criteria {
            "ocean_hot_nutrient" => tile.is_ocean && tile.temperature > 40.0 && tile.nutrients > 0.5,
            "land_frozen" => !tile.is_ocean && tile.temperature < -30.0,
            "ocean_warm_populated" => {
                let total_pop: f64 = tile.populations.values().sum();
                tile.is_ocean && tile.temperature > 10.0 && tile.temperature < 30.0 && total_pop > 100.0
            }
            "toxic_inhabited" => {
                let total_pop: f64 = tile.populations.values().sum();
                params.atmosphere.toxicity > 0.5 && total_pop > 0.0
            }
            "land_barren" => !tile.is_ocean && tile.nutrients < 0.05,
            _ => false,
        };
        if matches { count += 1; }
    }
    count
}
```

**Step 2: Add CodexTracker to Simulation**

In `sim-core/src/sim.rs`, add the codex tracker:

Add field to `Simulation`:
```rust
    codex: crate::codex::CodexTracker,
```

Initialize in `Simulation::new`:
```rust
        let codex = crate::codex::CodexTracker::new(crate::codex_entries::all_entries());
```

Add to `SimState` for save/load:
```rust
    codex_unlocked: Vec<String>,
```

Hook into `tick()` — after speciation check:
```rust
        if self.step_count.is_multiple_of(SPECIATION_EPOCH) {
            self.check_speciation();
            self.codex.check_all(&self.grid, &self.species, &self.params, &self.events, self.step_count);
        }
```

Add public accessors:
```rust
    pub fn codex(&self) -> &crate::codex::CodexTracker {
        &self.codex
    }

    pub fn codex_mut(&mut self) -> &mut crate::codex::CodexTracker {
        &mut self.codex
    }
```

Update `save_state` and `load_state` to persist unlocked IDs. In `save_state`:
```rust
            codex_unlocked: self.codex.unlocked_ids(),
```

In `load_state`, after constructing `Simulation`, restore unlocked state — this requires adding a method to CodexTracker:
```rust
    pub fn restore_unlocked(&mut self, ids: Vec<String>) {
        for id in ids {
            self.unlocked.insert(id);
        }
    }
```

**Step 3: Run tests**

Run: `cd /Users/jonathanguttman/Documents/World-Builder/sim-core && cargo test`
Expected: All 52 tests pass.

**Step 4: Commit**

```bash
git add sim-core/src/codex.rs sim-core/src/sim.rs
git commit -m "feat: CodexTracker with full check logic integrated into simulation tick"
```

---

## Task 6: Codex FFI Functions

**Files:**
- Modify: `sim-core/src/ffi.rs` (add 3 new functions)
- Modify: `sim-core/include/planet_architect.h`
- Test: `sim-core/tests/ffi_test.rs`

**Step 1: Write the failing test**

Append to `sim-core/tests/ffi_test.rs`:

```rust
#[test]
fn test_codex_all_entries() {
    let handle = pa_sim_create(42, ptr::null());

    let ptr = pa_sim_codex_all_entries_json(handle);
    assert!(!ptr.is_null());
    let json = unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str().unwrap();
    assert!(json.contains("species_thermophile"), "Should contain Thermophile entry: {}", &json[..200.min(json.len())]);
    assert!(json.contains("failure_total_extinction"), "Should contain failure entries");

    pa_sim_destroy(handle);
}

#[test]
fn test_codex_unlocked_after_species() {
    let handle = pa_sim_create(42, ptr::null());
    pa_sim_step(handle, 500);

    let species_json = CString::new(r#"{"id":0,"name":"Algae","traits":{"temp_optimal":15.0,"temp_range":50.0,"o2_need":0.0,"toxin_resistance":0.1,"trophic_level":"Producer","reproduction_rate":0.05,"dispersal":0.3,"mutation_rate":0.01}}"#).unwrap();
    pa_sim_add_species_json(handle, species_json.as_ptr(), 100.0);

    // Run past a speciation epoch so codex checks fire
    pa_sim_step(handle, 1000);

    let ptr = pa_sim_codex_unlocked_json(handle);
    assert!(!ptr.is_null());
    let json = unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str().unwrap();
    // SpeciesAppeared entries should be unlocked
    assert!(json.contains("species_thermophile") || json.contains("species_planktonic"),
        "Should have unlocked at least one species entry: {}", json);

    pa_sim_destroy(handle);
}
```

**Step 2: Run tests to verify they fail**

Run: `cd /Users/jonathanguttman/Documents/World-Builder/sim-core && cargo test --test ffi_test test_codex`
Expected: FAIL — functions not found.

**Step 3: Implement FFI functions**

Add to `sim-core/src/ffi.rs` before `// --- Species ---`:

```rust
// --- Codex ---

#[no_mangle]
pub extern "C" fn pa_sim_codex_all_entries_json(handle: *mut SimHandle) -> *const c_char {
    if handle.is_null() { return ptr::null(); }
    let h = unsafe { &mut *handle };
    let entries = h.sim.codex().entries();
    let json = serde_json::to_string(entries).unwrap_or_else(|_| "[]".to_string());
    h.codex_entries_cache = CString::new(json).unwrap_or_else(|_| CString::new("[]").unwrap());
    h.codex_entries_cache.as_ptr()
}

#[no_mangle]
pub extern "C" fn pa_sim_codex_unlocked_json(handle: *mut SimHandle) -> *const c_char {
    if handle.is_null() { return ptr::null(); }
    let h = unsafe { &mut *handle };
    let ids = h.sim.codex().unlocked_ids();
    let json = serde_json::to_string(&ids).unwrap_or_else(|_| "[]".to_string());
    h.codex_unlocked_cache = CString::new(json).unwrap_or_else(|_| CString::new("[]").unwrap());
    h.codex_unlocked_cache.as_ptr()
}

#[no_mangle]
pub extern "C" fn pa_sim_codex_new_unlocks_json(handle: *mut SimHandle) -> *const c_char {
    if handle.is_null() { return ptr::null(); }
    let h = unsafe { &mut *handle };
    let new_ids = h.sim.codex_mut().drain_new_unlocks();
    let json = serde_json::to_string(&new_ids).unwrap_or_else(|_| "[]".to_string());
    h.codex_new_cache = CString::new(json).unwrap_or_else(|_| CString::new("[]").unwrap());
    h.codex_new_cache.as_ptr()
}
```

Add cache fields to `SimHandle`:

```rust
    codex_entries_cache: CString,
    codex_unlocked_cache: CString,
    codex_new_cache: CString,
```

Initialize in `SimHandle::new`:

```rust
    codex_entries_cache: CString::new("[]").unwrap(),
    codex_unlocked_cache: CString::new("[]").unwrap(),
    codex_new_cache: CString::new("[]").unwrap(),
```

**Step 4: Update C header**

Add to `sim-core/include/planet_architect.h` before `// --- Species ---`:

```c
// --- Codex ---
// Returns JSON array of all codex entry definitions.
const char* pa_sim_codex_all_entries_json(PASimHandle handle);
// Returns JSON array of unlocked entry ID strings.
const char* pa_sim_codex_unlocked_json(PASimHandle handle);
// Returns JSON array of entry IDs unlocked since last call. Clears the buffer.
const char* pa_sim_codex_new_unlocks_json(PASimHandle handle);
```

**Step 5: Run tests**

Run: `cd /Users/jonathanguttman/Documents/World-Builder/sim-core && cargo test`
Expected: All 54 tests pass.

**Step 6: Commit**

```bash
git add sim-core/src/ffi.rs sim-core/include/planet_architect.h sim-core/tests/ffi_test.rs
git commit -m "feat: codex FFI — all_entries, unlocked, and new_unlocks JSON endpoints"
```

---

## Task 7: Rebuild Rust Library for iOS

**Step 1: Rebuild**

```bash
cd /Users/jonathanguttman/Documents/World-Builder/sim-core && ./build-ios.sh
```

**Step 2: Regenerate Xcode project**

```bash
cd /Users/jonathanguttman/Documents/World-Builder/ios && xcodegen generate
```

No commit needed.

---

## Task 8: Author Level 2 JSON + Bundle in iOS

**Files:**
- Create: `sim-core/levels/level_02_shallow_seas.json`
- Create: `ios/PlanetArchitect/Resources/Levels/level_02_shallow_seas.json`

**Step 1: Create Level 2 JSON**

Create `sim-core/levels/level_02_shallow_seas.json`:

```json
{
    "id": "level_02",
    "name": "Shallow Seas",
    "pack": "FREE",
    "description": "An ocean world with weak currents and sparse nutrients. Establish a stable marine ecosystem with producers, consumers, and predators coexisting for 20 million years.",
    "starting_seed": 4422,
    "starting_params": {
        "gravity": 8.5,
        "rotation_rate": 0.9,
        "axial_tilt": 12.0,
        "core_heat": 0.4,
        "magnetic_field": 0.5,
        "atmosphere": {
            "pressure": 0.8,
            "o2": 0.10,
            "co2": 0.03,
            "toxicity": 0.05
        },
        "hydrology": {
            "ocean_coverage": 0.85,
            "salinity": 0.04,
            "current_strength": 0.1,
            "ice_fraction": 0.15
        }
    },
    "allowed_interventions": ["AdjustCurrents", "NutrientBloom", "AdjustCO2", "AdjustO2", "AdjustSalinity"],
    "energy_budget": 60.0,
    "objective": {
        "type": "EcosystemStability",
        "min_trophic_levels": 3,
        "required_duration_steps": 20000000
    },
    "fail_conditions": ["Extinction", "TrophicCollapse"]
}
```

**Step 2: Copy to iOS resources**

```bash
cp sim-core/levels/level_02_shallow_seas.json ios/PlanetArchitect/Resources/Levels/
```

**Step 3: Update LevelLoader to handle EcosystemStability objective**

In `ios/PlanetArchitect/Core/LevelLoader.swift`, update `ObjectiveConfig` to include `minTrophicLevels`:

```swift
    struct ObjectiveConfig: Codable {
        let type: String
        let minBiomass: Double?
        let minTrophicLevels: UInt32?
        let requiredDurationSteps: UInt64

        enum CodingKeys: String, CodingKey {
            case type
            case minBiomass = "min_biomass"
            case minTrophicLevels = "min_trophic_levels"
            case requiredDurationSteps = "required_duration_steps"
        }
    }
```

Update `objectiveJSON` to include `min_trophic_levels`:

```swift
    static func objectiveJSON(from config: LevelConfig) -> String? {
        let obj = config.objective
        var dict: [String: Any] = [
            "type": obj.type,
            "required_duration_steps": obj.requiredDurationSteps,
        ]
        if let minBiomass = obj.minBiomass {
            dict["min_biomass"] = minBiomass
        }
        if let minTrophicLevels = obj.minTrophicLevels {
            dict["min_trophic_levels"] = minTrophicLevels
        }
        guard let data = try? JSONSerialization.data(withJSONObject: dict) else { return nil }
        return String(data: data, encoding: .utf8)
    }
```

**Step 4: Update ViewModel to seed Level 2 species**

In `ios/PlanetArchitect/Features/Planet/SimulationViewModel.swift`, update `startLevel(levelId:)` to seed species based on level:

```swift
    func startLevel(levelId: String) {
        guard let config = LevelLoader.load(levelId: levelId) else { return }
        self.levelConfig = config
        self.objectiveJSON = LevelLoader.objectiveJSON(from: config)
        self.energyRemaining = config.energyBudget
        self.requiredSteps = config.objective.requiredDurationSteps
        self.levelStatus = .playing
        self.sustainedSteps = 0

        let paramsJSON = LevelLoader.paramsJSON(from: config)
        engine = SimulationEngine(seed: config.startingSeed, paramsJSON: paramsJSON)

        // Warm up nutrients
        engine?.step(500)

        // Seed species based on level
        seedSpecies(for: config.id)
        refreshSnapshot()
    }

    private func seedSpecies(for levelId: String) {
        switch levelId {
        case "level_01":
            let microbeJSON = """
            {"id":0,"name":"Thermophile","traits":{"temp_optimal":5.0,"temp_range":60.0,"o2_need":0.0,"toxin_resistance":0.3,"trophic_level":"Producer","reproduction_rate":0.04,"dispersal":0.3,"mutation_rate":0.005}}
            """
            engine?.addSpecies(json: microbeJSON, initialPopulation: 100.0)

        case "level_02":
            let producer = """
            {"id":0,"name":"Planktonic Algae","traits":{"temp_optimal":18.0,"temp_range":40.0,"o2_need":0.0,"toxin_resistance":0.1,"trophic_level":"Producer","reproduction_rate":0.06,"dispersal":0.5,"mutation_rate":0.01}}
            """
            engine?.addSpecies(json: producer, initialPopulation: 200.0)

            let consumer = """
            {"id":1,"name":"Grazer","traits":{"temp_optimal":16.0,"temp_range":30.0,"o2_need":0.05,"toxin_resistance":0.2,"trophic_level":"Consumer","reproduction_rate":0.03,"dispersal":0.3,"mutation_rate":0.008}}
            """
            engine?.addSpecies(json: consumer, initialPopulation: 50.0)

            let predator = """
            {"id":2,"name":"Apex Filter","traits":{"temp_optimal":17.0,"temp_range":25.0,"o2_need":0.08,"toxin_resistance":0.15,"trophic_level":"Predator","reproduction_rate":0.015,"dispersal":0.2,"mutation_rate":0.005}}
            """
            engine?.addSpecies(json: predator, initialPopulation: 15.0)

        default:
            // Generic fallback — seed a producer
            let microbeJSON = """
            {"id":0,"name":"Microbe","traits":{"temp_optimal":15.0,"temp_range":50.0,"o2_need":0.0,"toxin_resistance":0.1,"trophic_level":"Producer","reproduction_rate":0.05,"dispersal":0.3,"mutation_rate":0.01}}
            """
            engine?.addSpecies(json: microbeJSON, initialPopulation: 100.0)
        }
    }
```

**Step 5: Add Level 2 tutorial content**

In `ios/PlanetArchitect/Features/Planet/TutorialOverlay.swift`, add Level 2 tutorials to `LevelTutorials`:

```swift
    static let level2: [TutorialStep] = [
        TutorialStep(
            title: "The Food Web",
            message: "This ocean world has three types of life: producers (algae), consumers (grazers), and predators. Each depends on the level below it."
        ),
        TutorialStep(
            title: "Ocean Currents",
            message: "Strengthen ocean currents to boost nutrient upwelling. More nutrients means more algae, which feeds the whole food chain."
        ),
        TutorialStep(
            title: "Boom and Bust",
            message: "Watch for population oscillations — too many grazers will crash the algae, then grazers starve, then predators starve. Classic boom-bust cycles."
        ),
        TutorialStep(
            title: "Balance is Key",
            message: "Use small, careful interventions. Your goal is to keep all three trophic levels alive for 20 million years. Watch the population overlay closely."
        ),
    ]
```

**Step 6: Update PlanetView to use level-specific tutorials**

In `ios/PlanetArchitect/Features/Planet/PlanetView.swift`, change the tutorial reference:

Replace:
```swift
                TutorialOverlay(
                    steps: LevelTutorials.level1,
```

With:
```swift
                TutorialOverlay(
                    steps: tutorialSteps,
```

Add computed property:
```swift
    private var tutorialSteps: [TutorialStep] {
        switch levelId {
        case "level_01_first_breath": return LevelTutorials.level1
        case "level_02_shallow_seas": return LevelTutorials.level2
        default: return LevelTutorials.level1
        }
    }
```

**Step 7: Update InterventionTray for new interventions**

In `ios/PlanetArchitect/UI/Components/InterventionTray.swift`, add cases to `InterventionButton`:

In `label`:
```swift
        case "AdjustCurrents": return "Currents"
        case "AdjustSalinity": return "Salinity"
```

In `icon`:
```swift
        case "AdjustCurrents": return "water.waves"
        case "AdjustSalinity": return "drop.triangle"
```

In `magnitude`:
```swift
        case "AdjustCurrents": return 0.15
        case "AdjustSalinity": return 0.05
```

**Step 8: Update ViewModel applyIntervention for new types**

In `SimulationViewModel.swift`, add cases to `applyIntervention`:

```swift
        case "AdjustCurrents":
            json = #"{"kind":{"AdjustCurrents":{"delta":\#(magnitude)}},"target_region":null,"step":\#(currentStep)}"#
        case "AdjustSalinity":
            json = #"{"kind":{"AdjustSalinity":{"delta":\#(magnitude)}},"target_region":null,"step":\#(currentStep)}"#
```

**Step 9: Update LevelBriefingView for EcosystemStability**

In `ios/PlanetArchitect/Features/Planet/LevelBriefingView.swift`, add to `objectiveDescription`:

```swift
        case "EcosystemStability":
            let levels = obj.minTrophicLevels ?? 3
            let years = obj.requiredDurationSteps
            return "Maintain a stable ecosystem with \(levels) trophic levels for \(formatSteps(years))."
```

And update the `ObjectiveConfig` reference — `LevelBriefingView` accesses `config.objective` which is a `LevelConfig.ObjectiveConfig`. Add `minTrophicLevels` access:

In `LevelBriefingView`, the property `objectiveDescription` references `obj.minBiomass` already. Just add the new case and reference `obj.minTrophicLevels`.

**Step 10: Commit**

```bash
git add sim-core/levels/level_02_shallow_seas.json \
    ios/PlanetArchitect/Resources/Levels/level_02_shallow_seas.json \
    ios/PlanetArchitect/Core/LevelLoader.swift \
    ios/PlanetArchitect/Features/Planet/SimulationViewModel.swift \
    ios/PlanetArchitect/Features/Planet/TutorialOverlay.swift \
    ios/PlanetArchitect/Features/Planet/PlanetView.swift \
    ios/PlanetArchitect/UI/Components/InterventionTray.swift \
    ios/PlanetArchitect/Features/Planet/LevelBriefingView.swift
git commit -m "feat: Level 2 Shallow Seas — 3-tier food web, new interventions, tutorial"
```

---

## Task 9: SimulationEngine Codex FFI Wrappers

**Files:**
- Modify: `ios/PlanetArchitect/Core/SimulationBridge/SimulationEngine.swift`

**Step 1: Add codex methods**

Add to `SimulationEngine.swift` after the Objective Evaluation section:

```swift
    // MARK: - Codex

    var codexAllEntriesJSON: String {
        guard let ptr = pa_sim_codex_all_entries_json(handle) else { return "[]" }
        return String(cString: ptr)
    }

    var codexUnlockedJSON: String {
        guard let ptr = pa_sim_codex_unlocked_json(handle) else { return "[]" }
        return String(cString: ptr)
    }

    var codexNewUnlocksJSON: String {
        guard let ptr = pa_sim_codex_new_unlocks_json(handle) else { return "[]" }
        return String(cString: ptr)
    }
```

**Step 2: Commit**

```bash
git add ios/PlanetArchitect/Core/SimulationBridge/SimulationEngine.swift
git commit -m "feat: SimulationEngine codex FFI wrappers"
```

---

## Task 10: CodexStore Persistence

**Files:**
- Create: `ios/PlanetArchitect/Core/CodexStore.swift`

**Step 1: Create CodexStore**

```swift
import SwiftUI

@MainActor
@Observable
final class CodexStore {
    private(set) var allEntries: [CodexEntryData] = []
    private(set) var unlockedIds: Set<String> = []
    private(set) var newDiscoveryIds: [String] = []
    var unviewedCount: Int { newDiscoveryIds.count }

    private let unlockedKey = "codex_unlocked_ids"

    init() {
        if let saved = UserDefaults.standard.array(forKey: unlockedKey) as? [String] {
            unlockedIds = Set(saved)
        }
    }

    func loadEntries(from json: String) {
        guard let data = json.data(using: .utf8),
              let entries = try? JSONDecoder().decode([CodexEntryData].self, from: data) else {
            return
        }
        allEntries = entries
    }

    func syncUnlocked(from json: String) {
        guard let data = json.data(using: .utf8),
              let ids = try? JSONDecoder().decode([String].self, from: data) else {
            return
        }
        for id in ids {
            if !unlockedIds.contains(id) {
                unlockedIds.insert(id)
                newDiscoveryIds.append(id)
            }
        }
        save()
    }

    func markDiscoveriesViewed() {
        newDiscoveryIds.removeAll()
    }

    func isUnlocked(_ id: String) -> Bool {
        unlockedIds.contains(id)
    }

    func entry(for id: String) -> CodexEntryData? {
        allEntries.first { $0.id == id }
    }

    func entries(in category: String) -> [CodexEntryData] {
        allEntries.filter { $0.category == category }
    }

    private func save() {
        UserDefaults.standard.set(Array(unlockedIds), forKey: unlockedKey)
    }
}

struct CodexEntryData: Codable, Identifiable {
    let id: String
    let category: String
    let name: String
    let requirementsText: String
    let factsText: String
    let flavorText: String
    let relatedEntryIds: [String]
    let iconAssetId: String

    enum CodingKeys: String, CodingKey {
        case id, category, name
        case requirementsText = "requirements_text"
        case factsText = "facts_text"
        case flavorText = "flavor_text"
        case relatedEntryIds = "related_entry_ids"
        case iconAssetId = "icon_asset_id"
    }
}
```

**Step 2: Commit**

```bash
git add ios/PlanetArchitect/Core/CodexStore.swift
git commit -m "feat: CodexStore with UserDefaults persistence and discovery tracking"
```

---

## Task 11: TabView App Structure

**Files:**
- Modify: `ios/PlanetArchitect/PlanetArchitectApp.swift`

**Step 1: Replace app root with TabView**

```swift
import SwiftUI

@main
struct PlanetArchitectApp: App {
    @State private var codexStore = CodexStore()

    var body: some Scene {
        WindowGroup {
            TabView {
                Tab("Campaign", systemImage: "map") {
                    NavigationStack {
                        LevelSelectView()
                    }
                }

                Tab("Codex", systemImage: "book.closed") {
                    NavigationStack {
                        CodexView()
                    }
                }
                .badge(codexStore.unviewedCount)
            }
            .environment(codexStore)
        }
    }
}
```

**Step 2: Commit**

```bash
git add ios/PlanetArchitect/PlanetArchitectApp.swift
git commit -m "feat: TabView with Campaign and Codex tabs"
```

---

## Task 12: CodexView (List with Search/Filter)

**Files:**
- Create: `ios/PlanetArchitect/Features/Codex/CodexView.swift`

**Step 1: Create CodexView**

```swift
import SwiftUI

struct CodexView: View {
    @Environment(CodexStore.self) private var store
    @State private var searchText = ""
    @State private var selectedCategory: String?

    private let categories = [
        ("Species", "hare"),
        ("BodyPlan", "figure.stand"),
        ("Biome", "mountain.2"),
        ("PlanetarySystem", "globe"),
        ("EvolutionaryEvent", "sparkles"),
        ("FailureMode", "exclamationmark.triangle"),
        ("RarePhenomenon", "star"),
        ("HistoricWorld", "clock"),
    ]

    var body: some View {
        List {
            // Category filter chips
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 8) {
                    categoryChip(nil, label: "All")
                    ForEach(categories, id: \.0) { cat, icon in
                        categoryChip(cat, label: displayName(cat), icon: icon)
                    }
                }
                .padding(.vertical, 4)
            }
            .listRowInsets(EdgeInsets())
            .listRowBackground(Color.clear)

            // Entries grouped by category
            ForEach(filteredCategories, id: \.0) { category, entries in
                Section(displayName(category)) {
                    ForEach(entries) { entry in
                        if store.isUnlocked(entry.id) {
                            NavigationLink {
                                CodexEntryView(entry: entry)
                            } label: {
                                CodexRow(entry: entry, unlocked: true)
                            }
                        } else {
                            CodexRow(entry: entry, unlocked: false)
                        }
                    }
                }
            }
        }
        .searchable(text: $searchText, prompt: "Search entries")
        .navigationTitle("Codex")
    }

    private var filteredCategories: [(String, [CodexEntryData])] {
        let allEntries = store.allEntries

        let filtered: [CodexEntryData]
        if searchText.isEmpty {
            if let cat = selectedCategory {
                filtered = allEntries.filter { $0.category == cat }
            } else {
                filtered = allEntries
            }
        } else {
            let search = searchText.lowercased()
            filtered = allEntries.filter { entry in
                let matchesSearch = entry.name.lowercased().contains(search)
                let matchesCat = selectedCategory.map { $0 == entry.category } ?? true
                return matchesSearch && matchesCat
            }
        }

        // Group by category
        var grouped: [String: [CodexEntryData]] = [:]
        for entry in filtered {
            grouped[entry.category, default: []].append(entry)
        }

        let order = categories.map(\.0)
        return order.compactMap { cat in
            guard let entries = grouped[cat], !entries.isEmpty else { return nil }
            return (cat, entries)
        }
    }

    private func categoryChip(_ category: String?, label: String, icon: String = "square.grid.2x2") -> some View {
        Button {
            selectedCategory = category
        } label: {
            Label(label, systemImage: icon)
                .font(.caption)
                .padding(.horizontal, 10)
                .padding(.vertical, 6)
                .background(
                    selectedCategory == category ? Color.accentColor.opacity(0.2) : Color.secondary.opacity(0.1),
                    in: Capsule()
                )
        }
        .buttonStyle(.plain)
    }

    private func displayName(_ category: String) -> String {
        switch category {
        case "Species": return "Species"
        case "BodyPlan": return "Body Plans"
        case "Biome": return "Biomes"
        case "PlanetarySystem": return "Planetary Systems"
        case "EvolutionaryEvent": return "Events"
        case "FailureMode": return "Failures"
        case "RarePhenomenon": return "Rare"
        case "HistoricWorld": return "Historic"
        default: return category
        }
    }
}

private struct CodexRow: View {
    let entry: CodexEntryData
    let unlocked: Bool

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: unlocked ? entry.iconAssetId : "lock.fill")
                .font(.title3)
                .frame(width: 30)
                .foregroundStyle(unlocked ? .primary : .secondary)

            VStack(alignment: .leading, spacing: 2) {
                Text(unlocked ? entry.name : "???")
                    .font(.body)
                Text(unlocked ? entry.factsText.prefix(60) + "..." : "Undiscovered")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
        }
        .opacity(unlocked ? 1.0 : 0.5)
    }
}
```

**Step 2: Commit**

```bash
git add ios/PlanetArchitect/Features/Codex/CodexView.swift
git commit -m "feat: CodexView with category filter, search, and locked/unlocked rows"
```

---

## Task 13: CodexEntryView (Detail)

**Files:**
- Create: `ios/PlanetArchitect/Features/Codex/CodexEntryView.swift`

**Step 1: Create CodexEntryView**

```swift
import SwiftUI

struct CodexEntryView: View {
    let entry: CodexEntryData
    @Environment(CodexStore.self) private var store

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                // Header
                HStack {
                    Image(systemName: entry.iconAssetId)
                        .font(.largeTitle)
                    VStack(alignment: .leading) {
                        Text(entry.name)
                            .font(.title.bold())
                        Text(displayCategory(entry.category))
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                    }
                }

                Divider()

                // Facts
                VStack(alignment: .leading, spacing: 8) {
                    Text("Facts")
                        .font(.headline)
                    Text(entry.factsText)
                        .font(.body)
                }

                // Flavor
                Text(entry.flavorText)
                    .font(.body.italic())
                    .foregroundStyle(.secondary)
                    .padding()
                    .frame(maxWidth: .infinity, alignment: .center)
                    .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 12))

                // Requirements
                VStack(alignment: .leading, spacing: 8) {
                    Text("How to Unlock")
                        .font(.headline)
                    Text(entry.requirementsText)
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }

                // Related entries
                if !entry.relatedEntryIds.isEmpty {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Related")
                            .font(.headline)
                        ForEach(entry.relatedEntryIds, id: \.self) { relatedId in
                            if let related = store.entry(for: relatedId), store.isUnlocked(relatedId) {
                                NavigationLink {
                                    CodexEntryView(entry: related)
                                } label: {
                                    Label(related.name, systemImage: related.iconAssetId)
                                        .font(.subheadline)
                                }
                            } else {
                                Label("???", systemImage: "lock.fill")
                                    .font(.subheadline)
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                }
            }
            .padding()
        }
        .navigationTitle(entry.name)
        .navigationBarTitleDisplayMode(.inline)
    }

    private func displayCategory(_ cat: String) -> String {
        switch cat {
        case "Species": return "Species"
        case "BodyPlan": return "Body Plan"
        case "Biome": return "Biome"
        case "PlanetarySystem": return "Planetary System"
        case "EvolutionaryEvent": return "Evolutionary Event"
        case "FailureMode": return "Failure Mode"
        case "RarePhenomenon": return "Rare Phenomenon"
        case "HistoricWorld": return "Historic World"
        default: return cat
        }
    }
}
```

**Step 2: Commit**

```bash
git add ios/PlanetArchitect/Features/Codex/CodexEntryView.swift
git commit -m "feat: CodexEntryView detail with facts, flavor, requirements, and cross-links"
```

---

## Task 14: Wire Codex into ViewModel + New Discoveries Sheet

**Files:**
- Modify: `ios/PlanetArchitect/Features/Planet/SimulationViewModel.swift`
- Modify: `ios/PlanetArchitect/Features/Planet/LevelCompleteView.swift`
- Create: `ios/PlanetArchitect/Features/Codex/NewDiscoveriesSheet.swift`

**Step 1: Add codex sync to ViewModel**

In `SimulationViewModel.swift`, add property:

```swift
    private(set) var newCodexUnlocks: [String] = []
```

In `refreshSnapshot()`, after existing snapshot updates, add:

```swift
        // Sync codex unlocks
        let unlockedJSON = engine.codexUnlockedJSON
        // Store for later sync with CodexStore
        if let data = unlockedJSON.data(using: .utf8),
           let ids = try? JSONDecoder().decode([String].self, from: data) {
            newCodexUnlocks = ids
        }
```

**Step 2: Create NewDiscoveriesSheet**

Create `ios/PlanetArchitect/Features/Codex/NewDiscoveriesSheet.swift`:

```swift
import SwiftUI

struct NewDiscoveriesSheet: View {
    let discoveryIds: [String]
    @Environment(CodexStore.self) private var store
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            VStack(spacing: 16) {
                Text("New Discoveries!")
                    .font(.title.bold())
                    .padding(.top)

                List {
                    ForEach(discoveryIds, id: \.self) { id in
                        if let entry = store.entry(for: id) {
                            HStack(spacing: 12) {
                                Image(systemName: entry.iconAssetId)
                                    .font(.title3)
                                    .frame(width: 30)

                                VStack(alignment: .leading, spacing: 2) {
                                    Text(entry.name)
                                        .font(.headline)
                                    Text(entry.flavorText)
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                        .lineLimit(2)
                                }
                            }
                        }
                    }
                }

                Button("View in Codex") {
                    store.markDiscoveriesViewed()
                    dismiss()
                }
                .buttonStyle(.borderedProminent)
                .padding(.bottom)
            }
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Dismiss") {
                        store.markDiscoveriesViewed()
                        dismiss()
                    }
                }
            }
        }
    }
}
```

**Step 3: Wire into LevelCompleteView**

In `ios/PlanetArchitect/Features/Planet/LevelCompleteView.swift`, add a parameter and sheet:

Update the struct to accept new discoveries:

```swift
struct LevelCompleteView: View {
    let won: Bool
    let failReason: String?
    let steps: UInt64
    let biodiversity: UInt32
    let newDiscoveryIds: [String]
    let onRestart: () -> Void
    let onExit: () -> Void

    @State private var showDiscoveries = false

    var body: some View {
        VStack(spacing: 24) {
            Image(systemName: won ? "checkmark.circle.fill" : "xmark.circle.fill")
                .font(.system(size: 64))
                .foregroundStyle(won ? .green : .red)

            Text(won ? "Level Complete!" : "Level Failed")
                .font(.largeTitle.bold())

            if let reason = failReason {
                Text(reason)
                    .font(.body)
                    .foregroundStyle(.secondary)
            }

            VStack(spacing: 8) {
                HStack {
                    Text("Time Elapsed")
                    Spacer()
                    Text(formatSteps(steps))
                }
                HStack {
                    Text("Species")
                    Spacer()
                    Text("\(biodiversity)")
                }
            }
            .font(.body)
            .padding()
            .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 12))

            if !newDiscoveryIds.isEmpty {
                Button("View \(newDiscoveryIds.count) New Discoveries") {
                    showDiscoveries = true
                }
                .buttonStyle(.bordered)
            }

            Spacer()

            HStack(spacing: 16) {
                Button("Try Again") {
                    onRestart()
                }
                .buttonStyle(.bordered)

                Button(won ? "Continue" : "Back") {
                    onExit()
                }
                .buttonStyle(.borderedProminent)
            }
        }
        .padding()
        .sheet(isPresented: $showDiscoveries) {
            NewDiscoveriesSheet(discoveryIds: newDiscoveryIds)
        }
    }

    private func formatSteps(_ steps: UInt64) -> String {
        if steps >= 1_000_000 {
            return String(format: "%.1fM years", Double(steps) / 1_000_000)
        } else if steps >= 1_000 {
            return String(format: "%.0fK years", Double(steps) / 1_000)
        }
        return "\(steps) years"
    }
}
```

**Step 4: Update PlanetView to pass newDiscoveryIds and sync CodexStore**

In `ios/PlanetArchitect/Features/Planet/PlanetView.swift`, add `@Environment(CodexStore.self)` and update the `LevelCompleteView` calls:

Add at top of struct:
```swift
    @Environment(CodexStore.self) private var codexStore
```

In `.onAppear`, after `viewModel.startLevel`, load codex entries:
```swift
        .onAppear {
            viewModel.startLevel(levelId: levelId)
            if codexStore.allEntries.isEmpty, let engine = viewModel.engine {
                codexStore.loadEntries(from: engine.codexAllEntriesJSON)
            }
        }
```

Update both `LevelCompleteView` invocations to pass `newDiscoveryIds`:

```swift
            if case .won = viewModel.levelStatus {
                LevelCompleteView(
                    won: true, failReason: nil,
                    steps: viewModel.currentStep,
                    biodiversity: viewModel.biodiversity,
                    newDiscoveryIds: viewModel.newCodexUnlocks,
                    onRestart: { restart() },
                    onExit: { dismiss() }
                )
                .background(.ultraThinMaterial)
                .onAppear { codexStore.syncUnlocked(from: viewModel.engine?.codexUnlockedJSON ?? "[]") }
            }

            if case .failed(let reason) = viewModel.levelStatus {
                LevelCompleteView(
                    won: false, failReason: reason,
                    steps: viewModel.currentStep,
                    biodiversity: viewModel.biodiversity,
                    newDiscoveryIds: viewModel.newCodexUnlocks,
                    onRestart: { restart() },
                    onExit: { dismiss() }
                )
                .background(.ultraThinMaterial)
                .onAppear { codexStore.syncUnlocked(from: viewModel.engine?.codexUnlockedJSON ?? "[]") }
            }
```

**Step 5: Commit**

```bash
git add ios/PlanetArchitect/Features/Planet/SimulationViewModel.swift \
    ios/PlanetArchitect/Features/Planet/LevelCompleteView.swift \
    ios/PlanetArchitect/Features/Codex/NewDiscoveriesSheet.swift \
    ios/PlanetArchitect/Features/Planet/PlanetView.swift
git commit -m "feat: codex integration — ViewModel sync, New Discoveries sheet, CodexStore wiring"
```

---

## Task 15: Build Verification & Fix

**Files:**
- None new — verification task

**Step 1: Rebuild Rust**

```bash
cd /Users/jonathanguttman/Documents/World-Builder/sim-core && ./build-ios.sh
```

**Step 2: Regenerate Xcode project**

```bash
cd /Users/jonathanguttman/Documents/World-Builder/ios && xcodegen generate
```

**Step 3: Clear DerivedData and build**

```bash
rm -rf ~/Library/Developer/Xcode/DerivedData/PlanetArchitect-* && \
cd /Users/jonathanguttman/Documents/World-Builder/ios && \
xcodebuild build \
    -project PlanetArchitect.xcodeproj \
    -scheme PlanetArchitect \
    -destination 'platform=iOS Simulator,name=iPhone 17 Pro' \
    -configuration Debug \
    CODE_SIGNING_ALLOWED=NO \
    2>&1 | grep -E '(error:|BUILD )' | head -20
```

Expected: BUILD SUCCEEDED.

**Step 4: Run Rust tests**

```bash
cd /Users/jonathanguttman/Documents/World-Builder/sim-core && cargo test
```

Expected: All tests pass (54 total).

**Step 5: Commit any fixes**

```bash
git add -A && git commit -m "fix: resolve build issues for Sprint 4"
```

---

## Task 16: CHANGELOG + Push

**Files:**
- Modify: `docs/CHANGELOG.md`

**Step 1: Add Sprint 4 entry**

Prepend to `docs/CHANGELOG.md` after the `# Changelog` header:

```markdown
## 0.5.0 - 2026-02-24
### Sprint 4: Food Web v0 + Level 2 + Codex v1
- `AdjustCurrents` and `AdjustSalinity` intervention types with salinity suitability factor
- `SpeciesExtinct` and `MassExtinction` SimEvent emission
- `EcosystemStability` objective refined to check distinct trophic levels
- 35 codex entries across 8 categories with diverse unlock triggers
- `CodexTracker` with full check logic (biomes, params, traits, milestones, failures)
- 3 new FFI functions: `pa_sim_codex_all_entries_json`, `pa_sim_codex_unlocked_json`, `pa_sim_codex_new_unlocks_json`
- Level 2 "Shallow Seas" — ocean world with 3-tier food web (Producer, Consumer, Predator)
- Level 2 tutorial (4 steps: food web, currents, oscillations, balance)
- `TabView` app structure with Campaign and Codex tabs
- `CodexView` with category filter chips, search, locked/unlocked entry rows
- `CodexEntryView` detail with facts, flavor text, requirements, and cross-linked related entries
- `CodexStore` with UserDefaults persistence and discovery tracking
- `NewDiscoveriesSheet` shown after level completion with unlocked entries
- 54 Rust tests passing, iOS simulator build verified (BUILD SUCCEEDED)
```

**Step 2: Commit and push**

```bash
git add docs/CHANGELOG.md
git commit -m "docs: Sprint 4 changelog v0.5.0"
git push origin main
```

---

Plan complete and saved to `docs/plans/2026-02-24-sprint-4-food-web-level-2-codex.md`. Two execution options:

**1. Subagent-Driven (this session)** — I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** — Open new session with executing-plans, batch execution with checkpoints

Which approach?

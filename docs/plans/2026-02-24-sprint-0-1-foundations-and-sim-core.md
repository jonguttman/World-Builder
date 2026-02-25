# Planet Architect Sprint 0+1: Foundations & Simulation Core

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Set up the full project scaffold with all design documents, initialize the Rust simulation core, and implement a deterministic sim engine that can run Level 1 ("First Breath") headlessly with verified reproducibility.

**Architecture:** Rust `sim-core` crate handles all simulation logic — climate, biosphere, objectives, codex triggers. iOS app (future sprint) is a thin SwiftUI client that calls sim-core via FFI. This plan builds the sim-core end-to-end and validates it headlessly. iOS UI is Sprint 2+.

**Tech Stack:** Rust 1.75+ (sim-core), `rand`/`rand_chacha` (deterministic RNG), `serde`/`serde_json` (serialization), `bincode` (save/load), JSON (level specs)

---

## Task 1: Project Scaffold & Design Documents

**Files:**
- Create: Full directory tree (see below)
- Create: `PLANET_ARCHITECT_CODEX.md`
- Create: `PRODUCT_BRIEF.md`
- Create: `APP_STORE_PAGE.md`
- Create: `ROADMAP_SPRINTS.md`
- Create: `AGENT_PROMPTS.md`
- Create: `SIMULATION_SPEC.md`
- Create: `API_SPEC.md`
- Create: `DATA_MODELS.md`
- Create: `UX_FLOW.md`
- Create: `ENGINEERING_PLAN.md`
- Create: `SECURITY_AND_ENTITLEMENTS.md`
- Modify: `README.md`

**Step 1: Create directory scaffold**

```bash
mkdir -p ios/PlanetArchitect/Features/{Campaign,Codex,Planet,Store,Timeline}
mkdir -p ios/PlanetArchitect/Core/{Rendering,SimulationBridge,Persistence,Networking}
mkdir -p ios/PlanetArchitect/UI/{Components,Styles}
mkdir -p sim-core/src
mkdir -p sim-core/tests
mkdir -p sim-core/benches
mkdir -p tools/{seedgen,tuning}
mkdir -p assets/{screenshots,video}
mkdir -p docs/plans
```

**Step 2: Save all design documents**

Save the following files to the repo root with content from the user's design spec (provided in the conversation that triggered this plan):

| File | Content Source |
|------|---------------|
| `PLANET_ARCHITECT_CODEX.md` | Section 4 of user spec — Codex, Campaign, Monetization combined doc |
| `PRODUCT_BRIEF.md` | Section 3 of user spec — Product Brief |
| `APP_STORE_PAGE.md` | Section 13 of user spec — App Store copy |
| `ROADMAP_SPRINTS.md` | Section 5 of user spec — Sprint breakdown |
| `AGENT_PROMPTS.md` | Section 6 of user spec — Agent team prompts |
| `SIMULATION_SPEC.md` | Section 7 of user spec — Simulation pseudocode |
| `API_SPEC.md` | Section 8 of user spec — iOS ↔ sim-core interfaces |
| `DATA_MODELS.md` | Section 9 of user spec — Level authoring + codex data models |
| `UX_FLOW.md` | Section 10 of user spec — UX flows |
| `ENGINEERING_PLAN.md` | Section 11 of user spec — Engineering approach |
| `SECURITY_AND_ENTITLEMENTS.md` | Section 12 of user spec — StoreKit2 plan |

Each document should be saved verbatim from the user's spec.

**Step 3: Update README.md**

Replace `README.md` with the full README from user spec Section 2:

```markdown
# Planet Architect

Speculative evolution simulation game for iOS. Players shape planetary physics and observe emergent biospheres evolve over millions of years.

## Monetization
- Free: Levels 1–3 (training)
- $4.99: Levels 4–10 (Core Pack)
- $8.99: Levels 11–20 (Advanced Pack)
No ads, no subscription.

## Repo Structure
- `/ios` SwiftUI + Metal renderer, StoreKit2, UI, persistence
- `/sim-core` deterministic simulation engine (Rust)
- `/docs` specs, roadmap, plans

## MVP (v0.1)
- Levels 1–3
- Codex v1
- Deterministic sim loop + time controls
- Save/load
- StoreKit entitlements

## Build Notes
- iOS calls into sim-core via a thin bridge (FFI).
- Simulation must be deterministic: same seed + same interventions = same outcomes.
```

**Step 4: Update CHANGELOG.md**

Add entry for scaffold setup.

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: project scaffold and design documents"
```

---

## Task 2: Initialize Rust Sim-Core Crate

**Files:**
- Create: `sim-core/Cargo.toml`
- Create: `sim-core/src/lib.rs`
- Create: `sim-core/src/main.rs`

**Step 1: Initialize Cargo project**

```bash
cd sim-core
cargo init --lib --name planet-architect-sim
```

**Step 2: Configure Cargo.toml**

```toml
[package]
name = "planet-architect-sim"
version = "0.1.0"
edition = "2021"
description = "Deterministic simulation engine for Planet Architect"

[dependencies]
rand = "0.8"
rand_chacha = "0.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
bincode = "1"

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "sim_benchmark"
harness = false

[lib]
crate-type = ["lib", "staticlib"]
```

**Step 3: Create minimal lib.rs**

```rust
//! Planet Architect — Deterministic Simulation Core

pub mod types;
pub mod climate;
pub mod biosphere;
pub mod sim;
pub mod level;
pub mod codex;
pub mod snapshot;

pub use sim::Simulation;
pub use types::*;
```

**Step 4: Create placeholder modules**

Create empty files:
- `sim-core/src/types.rs`
- `sim-core/src/climate.rs`
- `sim-core/src/biosphere.rs`
- `sim-core/src/sim.rs`
- `sim-core/src/level.rs`
- `sim-core/src/codex.rs`
- `sim-core/src/snapshot.rs`

Each should contain just: `// TODO: implement`

**Step 5: Create main.rs for headless testing**

```rust
use planet_architect_sim::Simulation;

fn main() {
    println!("Planet Architect Simulation Core v0.1");
    println!("Use `cargo test` to run simulation tests.");
}
```

**Step 6: Verify it compiles**

Run: `cd sim-core && cargo check`
Expected: Compiles with no errors (warnings OK for unused modules).

**Step 7: Create benchmark placeholder**

Create `sim-core/benches/sim_benchmark.rs`:
```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_placeholder(c: &mut Criterion) {
    c.bench_function("placeholder", |b| {
        b.iter(|| {
            // Will benchmark sim stepping here
            1 + 1
        })
    });
}

criterion_group!(benches, bench_placeholder);
criterion_main!(benches);
```

**Step 8: Commit**

```bash
git add sim-core/
git commit -m "feat: initialize Rust sim-core crate with dependencies"
```

---

## Task 3: Core Data Structures

**Files:**
- Create: `sim-core/src/types.rs` (full implementation)
- Test: `sim-core/tests/types_test.rs`

**Step 1: Write the failing test**

Create `sim-core/tests/types_test.rs`:

```rust
use planet_architect_sim::types::*;

#[test]
fn test_planet_params_default_creates_valid_planet() {
    let params = PlanetParams::default();
    assert!(params.gravity > 0.0);
    assert!(params.atmosphere.pressure > 0.0);
    assert!(params.hydrology.ocean_coverage >= 0.0);
    assert!(params.hydrology.ocean_coverage <= 1.0);
}

#[test]
fn test_tile_default_is_barren() {
    let tile = Tile::default();
    assert_eq!(tile.elevation, 0.0);
    assert!(!tile.is_ocean);
    assert!(tile.populations.is_empty());
}

#[test]
fn test_world_grid_dimensions() {
    let grid = WorldGrid::new(64, 32);
    assert_eq!(grid.width, 64);
    assert_eq!(grid.height, 32);
    assert_eq!(grid.tiles.len(), 64 * 32);
}

#[test]
fn test_species_traits_serialization() {
    let species = Species {
        id: 0,
        name: "Proto-microbe".to_string(),
        traits: SpeciesTraits {
            temp_optimal: 25.0,
            temp_range: 40.0,
            o2_need: 0.0,
            toxin_resistance: 0.1,
            trophic_level: TrophicLevel::Producer,
            reproduction_rate: 0.05,
            dispersal: 0.3,
            mutation_rate: 0.01,
        },
    };
    let json = serde_json::to_string(&species).unwrap();
    let deserialized: Species = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "Proto-microbe");
}

#[test]
fn test_intervention_types() {
    let intervention = Intervention {
        kind: InterventionKind::AdjustCO2 { delta: 0.1 },
        target_region: None,
        step: 1000,
    };
    assert_eq!(intervention.step, 1000);
}
```

**Step 2: Run test to verify it fails**

Run: `cd sim-core && cargo test --test types_test`
Expected: FAIL — module `types` has no items.

**Step 3: Implement types.rs**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- Planet Parameters (player-controlled) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetParams {
    pub gravity: f32,           // m/s², Earth = 9.8
    pub rotation_rate: f32,     // relative to Earth = 1.0
    pub axial_tilt: f32,        // degrees, Earth = 23.4
    pub core_heat: f32,         // 0.0–1.0 scalar
    pub magnetic_field: f32,    // 0.0–1.0 scalar
    pub atmosphere: AtmosphereState,
    pub hydrology: HydroState,
}

impl Default for PlanetParams {
    fn default() -> Self {
        Self {
            gravity: 9.8,
            rotation_rate: 1.0,
            axial_tilt: 23.4,
            core_heat: 0.4,
            magnetic_field: 0.6,
            atmosphere: AtmosphereState::default(),
            hydrology: HydroState::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtmosphereState {
    pub pressure: f32,    // atm, Earth = 1.0
    pub o2: f32,          // fraction, Earth ≈ 0.21
    pub co2: f32,         // fraction, Earth ≈ 0.0004
    pub toxicity: f32,    // 0.0–1.0 aggregate
}

impl Default for AtmosphereState {
    fn default() -> Self {
        Self {
            pressure: 1.0,
            o2: 0.21,
            co2: 0.0004,
            toxicity: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydroState {
    pub ocean_coverage: f32,    // 0.0–1.0
    pub salinity: f32,          // 0.0–1.0 scalar
    pub current_strength: f32,  // 0.0–1.0 scalar
    pub ice_fraction: f32,      // 0.0–1.0
}

impl Default for HydroState {
    fn default() -> Self {
        Self {
            ocean_coverage: 0.7,
            salinity: 0.035,
            current_strength: 0.5,
            ice_fraction: 0.1,
        }
    }
}

// --- World Grid ---

pub const DEFAULT_WIDTH: usize = 64;
pub const DEFAULT_HEIGHT: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldGrid {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Tile>,
}

impl WorldGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            tiles: vec![Tile::default(); width * height],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> &Tile {
        &self.tiles[y * self.width + x]
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut Tile {
        &mut self.tiles[y * self.width + x]
    }

    /// Returns latitude in degrees (-90 to 90) for a given y row
    pub fn latitude(&self, y: usize) -> f32 {
        90.0 - (y as f32 / self.height as f32) * 180.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub elevation: f32,
    pub is_ocean: bool,
    pub temperature: f32,
    pub moisture: f32,
    pub nutrients: f32,
    pub radiation: f32,
    pub biome_id: u16,
    pub populations: HashMap<u32, f64>, // species_id -> population count
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            elevation: 0.0,
            is_ocean: false,
            temperature: 0.0,
            moisture: 0.0,
            nutrients: 0.0,
            radiation: 0.0,
            biome_id: 0,
            populations: HashMap::new(),
        }
    }
}

// --- Species ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrophicLevel {
    Producer,
    Consumer,
    Predator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeciesTraits {
    pub temp_optimal: f32,
    pub temp_range: f32,
    pub o2_need: f32,
    pub toxin_resistance: f32,
    pub trophic_level: TrophicLevel,
    pub reproduction_rate: f32,
    pub dispersal: f32,       // 0.0–1.0, migration capability
    pub mutation_rate: f32,   // probability per epoch
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Species {
    pub id: u32,
    pub name: String,
    pub traits: SpeciesTraits,
}

// --- Interventions ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterventionKind {
    AdjustCO2 { delta: f32 },
    AdjustO2 { delta: f32 },
    CloudSeeding { magnitude: f32 },
    NutrientBloom { magnitude: f32 },
    IceMeltPulse { magnitude: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionTarget {
    pub x: usize,
    pub y: usize,
    pub radius: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intervention {
    pub kind: InterventionKind,
    pub target_region: Option<RegionTarget>,
    pub step: u64, // simulation step when applied
}

// --- Simulation Time ---

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TimeSpeed {
    Observe,  // 1x
    Adapt,    // 100x
    Epoch,    // 10_000x
    Eon,      // 1_000_000x
}

impl TimeSpeed {
    pub fn steps_per_batch(&self) -> u64 {
        match self {
            TimeSpeed::Observe => 1,
            TimeSpeed::Adapt => 100,
            TimeSpeed::Epoch => 10_000,
            TimeSpeed::Eon => 1_000_000,
        }
    }
}

// --- Events ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimEvent {
    SpeciesAppeared { species_id: u32, step: u64 },
    SpeciesExtinct { species_id: u32, step: u64 },
    Speciation { parent_id: u32, child_id: u32, step: u64 },
    MassExtinction { survivors: usize, step: u64 },
    CodexUnlock { entry_id: String, step: u64 },
    ObjectiveMet { objective_id: String, step: u64 },
    ObjectiveFailed { objective_id: String, step: u64 },
}
```

**Step 4: Run tests to verify they pass**

Run: `cd sim-core && cargo test --test types_test`
Expected: All 5 tests PASS.

**Step 5: Commit**

```bash
cd sim-core && git add -A
git commit -m "feat: core data structures — PlanetParams, WorldGrid, Species, Interventions"
```

---

## Task 4: Seeded RNG & Deterministic Tick Loop

**Files:**
- Create: `sim-core/src/sim.rs`
- Test: `sim-core/tests/determinism_test.rs`

**Step 1: Write the failing test**

Create `sim-core/tests/determinism_test.rs`:

```rust
use planet_architect_sim::sim::Simulation;
use planet_architect_sim::types::PlanetParams;

#[test]
fn test_determinism_same_seed_same_result() {
    let seed = 42u64;
    let params = PlanetParams::default();

    let mut sim1 = Simulation::new(seed, params.clone());
    sim1.step(1000);
    let snap1 = sim1.snapshot();

    let mut sim2 = Simulation::new(seed, params);
    sim2.step(1000);
    let snap2 = sim2.snapshot();

    assert_eq!(snap1.current_step, snap2.current_step);
    assert_eq!(snap1.current_step, 1000);
    // Temperature at same tile must be identical
    for i in 0..snap1.grid.tiles.len() {
        assert_eq!(
            snap1.grid.tiles[i].temperature,
            snap2.grid.tiles[i].temperature,
            "Temperature mismatch at tile {}",
            i
        );
        assert_eq!(
            snap1.grid.tiles[i].nutrients,
            snap2.grid.tiles[i].nutrients,
            "Nutrient mismatch at tile {}",
            i
        );
    }
}

#[test]
fn test_different_seed_different_result() {
    let params = PlanetParams::default();

    let mut sim1 = Simulation::new(1, params.clone());
    sim1.step(100);

    let mut sim2 = Simulation::new(2, params);
    sim2.step(100);

    // At least some tiles should differ
    let snap1 = sim1.snapshot();
    let snap2 = sim2.snapshot();
    let diffs: usize = snap1.grid.tiles.iter().zip(snap2.grid.tiles.iter())
        .filter(|(a, b)| (a.temperature - b.temperature).abs() > 0.001)
        .count();
    assert!(diffs > 0, "Different seeds should produce different worlds");
}

#[test]
fn test_step_advances_time() {
    let mut sim = Simulation::new(42, PlanetParams::default());
    assert_eq!(sim.current_step(), 0);
    sim.step(500);
    assert_eq!(sim.current_step(), 500);
    sim.step(500);
    assert_eq!(sim.current_step(), 1000);
}
```

**Step 2: Run test to verify it fails**

Run: `cd sim-core && cargo test --test determinism_test`
Expected: FAIL — `Simulation` not found or has no `new`/`step`/`snapshot`.

**Step 3: Implement sim.rs**

```rust
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::types::*;
use crate::climate;
use crate::snapshot::SimSnapshot;

pub struct Simulation {
    seed: u64,
    rng: ChaCha8Rng,
    step_count: u64,
    params: PlanetParams,
    grid: WorldGrid,
    species: Vec<Species>,
    events: Vec<SimEvent>,
    next_species_id: u32,
}

impl Simulation {
    pub fn new(seed: u64, params: PlanetParams) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut grid = WorldGrid::new(DEFAULT_WIDTH, DEFAULT_HEIGHT);

        // Initialize grid from params + seed
        climate::init_grid(&mut grid, &params, &mut rng);

        Self {
            seed,
            rng,
            step_count: 0,
            params,
            grid,
            species: Vec::new(),
            events: Vec::new(),
            next_species_id: 0,
        }
    }

    pub fn current_step(&self) -> u64 {
        self.step_count
    }

    pub fn step(&mut self, steps: u64) {
        for _ in 0..steps {
            self.tick();
        }
    }

    fn tick(&mut self) {
        self.step_count += 1;
        climate::update(&mut self.grid, &self.params, self.step_count);
    }

    pub fn snapshot(&self) -> SimSnapshot {
        SimSnapshot {
            seed: self.seed,
            current_step: self.step_count,
            grid: self.grid.clone(),
            params: self.params.clone(),
            species: self.species.clone(),
            events: self.events.clone(),
            biodiversity_count: self.species.len() as u32,
        }
    }

    pub fn species(&self) -> &[Species] {
        &self.species
    }

    pub fn grid(&self) -> &WorldGrid {
        &self.grid
    }

    pub fn params(&self) -> &PlanetParams {
        &self.params
    }
}
```

**Step 4: Create snapshot.rs stub**

```rust
use serde::{Deserialize, Serialize};
use crate::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimSnapshot {
    pub seed: u64,
    pub current_step: u64,
    pub grid: WorldGrid,
    pub params: PlanetParams,
    pub species: Vec<Species>,
    pub events: Vec<SimEvent>,
    pub biodiversity_count: u32,
}
```

**Step 5: Create climate.rs stub (minimal for tests to pass)**

```rust
use rand_chacha::ChaCha8Rng;
use rand::Rng;

use crate::types::*;

/// Initialize grid tiles from planet params and seed
pub fn init_grid(grid: &mut WorldGrid, params: &PlanetParams, rng: &mut ChaCha8Rng) {
    for y in 0..grid.height {
        let lat = grid.latitude(y);
        for x in 0..grid.width {
            let tile = grid.get_mut(x, y);

            // Set elevation with some noise
            tile.elevation = rng.gen_range(-1.0..1.0);
            tile.is_ocean = tile.elevation < 0.0
                && (rng.gen::<f32>() < params.hydrology.ocean_coverage);

            // Base temperature from latitude
            let base_temp = base_temperature(lat, params);
            tile.temperature = base_temp;
            tile.moisture = if tile.is_ocean { 1.0 } else { rng.gen_range(0.0..0.5) };
            tile.nutrients = rng.gen_range(0.0..0.3);
            tile.radiation = radiation_level(lat, params);
        }
    }
}

/// Update climate for one tick
pub fn update(grid: &mut WorldGrid, params: &PlanetParams, step: u64) {
    let season_phase = (step as f32 * 0.001) % (2.0 * std::f32::consts::PI);

    for y in 0..grid.height {
        let lat = grid.latitude(y);
        for x in 0..grid.width {
            let tile = grid.get_mut(x, y);
            tile.temperature = compute_temperature(lat, params, season_phase);
        }
    }
}

fn base_temperature(lat: f32, params: &PlanetParams) -> f32 {
    // Insolation decreases with latitude
    let insolation = (lat.to_radians().cos()).max(0.0);
    let base = -20.0 + 50.0 * insolation;

    // Greenhouse effect
    let greenhouse = 33.0 * (1.0 + params.atmosphere.co2).ln();

    // Core heat contribution
    let core = params.core_heat * 5.0;

    base + greenhouse + core
}

fn compute_temperature(lat: f32, params: &PlanetParams, season_phase: f32) -> f32 {
    let base = base_temperature(lat, params);

    // Seasonal variation from axial tilt
    let seasonal = params.axial_tilt.to_radians().sin()
        * season_phase.sin()
        * lat.to_radians().sin()
        * 15.0;

    // Albedo cooling from ice
    let albedo_cooling = params.hydrology.ice_fraction * 10.0;

    let temp = base + seasonal - albedo_cooling;
    temp.clamp(-80.0, 80.0)
}

fn radiation_level(lat: f32, params: &PlanetParams) -> f32 {
    let base_radiation = (lat.to_radians().cos()).max(0.0);
    let shielding = params.magnetic_field;
    (base_radiation * (1.0 - shielding * 0.8)).clamp(0.0, 1.0)
}
```

**Step 6: Run tests to verify they pass**

Run: `cd sim-core && cargo test --test determinism_test`
Expected: All 3 tests PASS.

**Step 7: Commit**

```bash
git add sim-core/src/sim.rs sim-core/src/climate.rs sim-core/src/snapshot.rs sim-core/tests/determinism_test.rs
git commit -m "feat: deterministic tick loop with seeded RNG and climate initialization"
```

---

## Task 5: Full Climate Model

**Files:**
- Modify: `sim-core/src/climate.rs`
- Test: `sim-core/tests/climate_test.rs`

**Step 1: Write the failing test**

Create `sim-core/tests/climate_test.rs`:

```rust
use planet_architect_sim::sim::Simulation;
use planet_architect_sim::types::*;

#[test]
fn test_greenhouse_effect_raises_temperature() {
    let mut low_co2 = PlanetParams::default();
    low_co2.atmosphere.co2 = 0.0001;

    let mut high_co2 = PlanetParams::default();
    high_co2.atmosphere.co2 = 0.05;

    let mut sim_low = Simulation::new(42, low_co2);
    let mut sim_high = Simulation::new(42, high_co2);

    sim_low.step(10);
    sim_high.step(10);

    let avg_low = avg_temperature(&sim_low.snapshot().grid);
    let avg_high = avg_temperature(&sim_high.snapshot().grid);

    assert!(avg_high > avg_low, "Higher CO2 should mean higher temps: {} vs {}", avg_high, avg_low);
}

#[test]
fn test_ice_fraction_cools_planet() {
    let mut no_ice = PlanetParams::default();
    no_ice.hydrology.ice_fraction = 0.0;

    let mut lots_ice = PlanetParams::default();
    lots_ice.hydrology.ice_fraction = 0.8;

    let mut sim_warm = Simulation::new(42, no_ice);
    let mut sim_cold = Simulation::new(42, lots_ice);

    sim_warm.step(10);
    sim_cold.step(10);

    let avg_warm = avg_temperature(&sim_warm.snapshot().grid);
    let avg_cold = avg_temperature(&sim_cold.snapshot().grid);

    assert!(avg_warm > avg_cold, "More ice should mean cooler temps: {} vs {}", avg_warm, avg_cold);
}

#[test]
fn test_temperature_bounded() {
    let mut extreme = PlanetParams::default();
    extreme.atmosphere.co2 = 1.0; // extreme greenhouse
    extreme.core_heat = 1.0;

    let mut sim = Simulation::new(42, extreme);
    sim.step(100);

    let snap = sim.snapshot();
    for tile in &snap.grid.tiles {
        assert!(tile.temperature >= -80.0 && tile.temperature <= 80.0,
            "Temperature {} out of bounds", tile.temperature);
    }
}

#[test]
fn test_equator_warmer_than_poles() {
    let mut sim = Simulation::new(42, PlanetParams::default());
    sim.step(10);
    let snap = sim.snapshot();

    // Row 16 is equator (height/2), row 0 is north pole
    let equator_temp = snap.grid.get(32, 16).temperature;
    let pole_temp = snap.grid.get(32, 0).temperature;

    assert!(equator_temp > pole_temp,
        "Equator ({}) should be warmer than pole ({})", equator_temp, pole_temp);
}

fn avg_temperature(grid: &WorldGrid) -> f32 {
    let sum: f32 = grid.tiles.iter().map(|t| t.temperature).sum();
    sum / grid.tiles.len() as f32
}
```

**Step 2: Run tests to verify they pass (or identify gaps)**

Run: `cd sim-core && cargo test --test climate_test`
Expected: All 4 tests PASS with current implementation. If any fail, adjust climate model.

**Step 3: Add nutrient model to climate.rs**

Add to `climate.rs`:

```rust
/// Update nutrients for one tick
pub fn update_nutrients(grid: &mut WorldGrid, params: &PlanetParams) {
    let height = grid.height;
    let width = grid.width;

    for y in 0..height {
        for x in 0..width {
            let tile = grid.get_mut(x, y);

            // Nutrient sources
            let volcanism = params.core_heat * 0.001;
            let upwelling = if tile.is_ocean {
                params.hydrology.current_strength * 0.002
            } else {
                0.0
            };

            // Biomass decay feeds nutrients
            let decay: f64 = tile.populations.values().sum::<f64>() * 0.0001;

            // Nutrient sinks
            let leaching = if !tile.is_ocean { tile.moisture * 0.001 } else { 0.0 };

            tile.nutrients = (tile.nutrients + volcanism + upwelling + decay as f32 - leaching)
                .clamp(0.0, 1.0);
        }
    }
}
```

**Step 4: Integrate nutrient update into sim tick**

In `sim.rs`, update the `tick` method:

```rust
fn tick(&mut self) {
    self.step_count += 1;
    climate::update(&mut self.grid, &self.params, self.step_count);
    climate::update_nutrients(&mut self.grid, &self.params);
}
```

**Step 5: Add nutrient test**

Append to `climate_test.rs`:

```rust
#[test]
fn test_nutrients_increase_with_volcanism() {
    let mut low_heat = PlanetParams::default();
    low_heat.core_heat = 0.0;

    let mut high_heat = PlanetParams::default();
    high_heat.core_heat = 1.0;

    let mut sim_low = Simulation::new(42, low_heat);
    let mut sim_high = Simulation::new(42, high_heat);

    sim_low.step(1000);
    sim_high.step(1000);

    let avg_low: f32 = sim_low.snapshot().grid.tiles.iter().map(|t| t.nutrients).sum::<f32>()
        / sim_low.snapshot().grid.tiles.len() as f32;
    let avg_high: f32 = sim_high.snapshot().grid.tiles.iter().map(|t| t.nutrients).sum::<f32>()
        / sim_high.snapshot().grid.tiles.len() as f32;

    assert!(avg_high > avg_low, "Higher core heat should mean more nutrients: {} vs {}", avg_high, avg_low);
}
```

**Step 6: Run all tests**

Run: `cd sim-core && cargo test`
Expected: All tests PASS.

**Step 7: Commit**

```bash
git add sim-core/
git commit -m "feat: climate model with greenhouse, albedo, seasons, and nutrient cycling"
```

---

## Task 6: Biosphere — Microbe Model

**Files:**
- Create: `sim-core/src/biosphere.rs`
- Test: `sim-core/tests/biosphere_test.rs`

**Step 1: Write the failing test**

Create `sim-core/tests/biosphere_test.rs`:

```rust
use planet_architect_sim::biosphere;
use planet_architect_sim::types::*;

fn make_producer() -> Species {
    Species {
        id: 0,
        name: "Proto-algae".to_string(),
        traits: SpeciesTraits {
            temp_optimal: 20.0,
            temp_range: 40.0,
            o2_need: 0.0,
            toxin_resistance: 0.1,
            trophic_level: TrophicLevel::Producer,
            reproduction_rate: 0.05,
            dispersal: 0.2,
            mutation_rate: 0.01,
        },
    }
}

fn make_habitable_tile() -> Tile {
    Tile {
        elevation: -0.5,
        is_ocean: true,
        temperature: 20.0,
        moisture: 1.0,
        nutrients: 0.5,
        radiation: 0.2,
        biome_id: 1,
        populations: std::collections::HashMap::new(),
    }
}

#[test]
fn test_suitability_optimal_conditions() {
    let species = make_producer();
    let tile = make_habitable_tile();
    let atmo = AtmosphereState::default();

    let suit = biosphere::suitability(&species.traits, &tile, &atmo);
    assert!(suit > 0.5, "Optimal conditions should give high suitability: {}", suit);
}

#[test]
fn test_suitability_wrong_temperature() {
    let species = make_producer();
    let mut tile = make_habitable_tile();
    tile.temperature = -60.0; // way too cold

    let atmo = AtmosphereState::default();
    let suit = biosphere::suitability(&species.traits, &tile, &atmo);
    assert!(suit < 0.1, "Wrong temp should give low suitability: {}", suit);
}

#[test]
fn test_population_grows_in_good_conditions() {
    let species = make_producer();
    let mut tile = make_habitable_tile();
    tile.populations.insert(0, 100.0);

    let atmo = AtmosphereState::default();
    let species_list = vec![species];

    biosphere::update_tile_populations(&mut tile, &species_list, &atmo);

    let pop = tile.populations.get(&0).unwrap();
    assert!(*pop > 100.0, "Population should grow in good conditions: {}", pop);
}

#[test]
fn test_population_declines_in_bad_conditions() {
    let species = make_producer();
    let mut tile = make_habitable_tile();
    tile.temperature = -50.0; // hostile
    tile.populations.insert(0, 100.0);

    let atmo = AtmosphereState::default();
    let species_list = vec![species];

    biosphere::update_tile_populations(&mut tile, &species_list, &atmo);

    let pop = tile.populations.get(&0).unwrap();
    assert!(*pop < 100.0, "Population should decline in bad conditions: {}", pop);
}

#[test]
fn test_population_bounded_by_carrying_capacity() {
    let species = make_producer();
    let mut tile = make_habitable_tile();
    tile.populations.insert(0, 1_000_000.0); // way over capacity

    let atmo = AtmosphereState::default();
    let species_list = vec![species];

    // Run many updates
    for _ in 0..100 {
        biosphere::update_tile_populations(&mut tile, &species_list, &atmo);
    }

    let pop = tile.populations.get(&0).unwrap();
    assert!(*pop < 1_000_000.0, "Population should be bounded by carrying capacity: {}", pop);
}

#[test]
fn test_extinct_species_removed() {
    let species = make_producer();
    let mut tile = make_habitable_tile();
    tile.temperature = -70.0; // lethal
    tile.nutrients = 0.0;
    tile.populations.insert(0, 1.0); // tiny population

    let atmo = AtmosphereState::default();
    let species_list = vec![species];

    for _ in 0..100 {
        biosphere::update_tile_populations(&mut tile, &species_list, &atmo);
    }

    let pop = *tile.populations.get(&0).unwrap_or(&0.0);
    assert!(pop == 0.0, "Tiny population in lethal conditions should go extinct: {}", pop);
}
```

**Step 2: Run test to verify it fails**

Run: `cd sim-core && cargo test --test biosphere_test`
Expected: FAIL — `biosphere::suitability` not found.

**Step 3: Implement biosphere.rs**

```rust
use crate::types::*;

/// Minimum population before species is considered extinct on a tile
const EXTINCTION_THRESHOLD: f64 = 0.5;

/// How suitable a tile is for a species (0.0–1.0)
pub fn suitability(traits: &SpeciesTraits, tile: &Tile, atmo: &AtmosphereState) -> f32 {
    // Temperature suitability: gaussian-like falloff from optimal
    let temp_diff = (tile.temperature - traits.temp_optimal).abs();
    let temp_suit = if traits.temp_range > 0.0 {
        (1.0 - (temp_diff / traits.temp_range).powi(2)).max(0.0)
    } else {
        0.0
    };

    // Oxygen suitability: producers don't need O2, consumers do
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

    // Nutrient availability matters for producers
    let nutrient_suit = match traits.trophic_level {
        TrophicLevel::Producer => tile.nutrients.min(1.0),
        _ => 1.0, // consumers get energy from prey, not directly from nutrients
    };

    // Combine multiplicatively
    (temp_suit * o2_suit * tox_suit * nutrient_suit).clamp(0.0, 1.0)
}

/// Carrying capacity for a species on a tile
pub fn carrying_capacity(traits: &SpeciesTraits, tile: &Tile, atmo: &AtmosphereState) -> f64 {
    let suit = suitability(traits, tile, atmo) as f64;
    let base_capacity = match traits.trophic_level {
        TrophicLevel::Producer => 10_000.0,
        TrophicLevel::Consumer => 1_000.0,
        TrophicLevel::Predator => 100.0,
    };
    base_capacity * suit * tile.nutrients as f64
}

/// Update populations on a single tile for one tick
pub fn update_tile_populations(
    tile: &mut Tile,
    species_list: &[Species],
    atmo: &AtmosphereState,
) {
    // Collect species IDs present on this tile
    let present_ids: Vec<u32> = tile.populations.keys().cloned().collect();

    for &sp_id in &present_ids {
        let species = match species_list.iter().find(|s| s.id == sp_id) {
            Some(s) => s,
            None => continue,
        };

        let pop = *tile.populations.get(&sp_id).unwrap_or(&0.0);
        if pop <= 0.0 {
            continue;
        }

        let suit = suitability(&species.traits, tile, atmo) as f64;
        let capacity = carrying_capacity(&species.traits, tile, atmo);

        // Logistic growth
        let r = species.traits.reproduction_rate as f64;
        let growth = if capacity > 0.0 {
            r * suit * pop * (1.0 - pop / capacity)
        } else {
            -pop * 0.1 // no capacity = decline
        };

        // Base mortality from poor suitability
        let mortality_rate = 0.02 * (1.0 - suit);
        let mortality = mortality_rate * pop;

        // Predation pressure (simplified: look for predators on this tile)
        let predation = compute_predation(tile, species, species_list);

        let new_pop = (pop + growth - mortality - predation).max(0.0);

        if new_pop < EXTINCTION_THRESHOLD {
            tile.populations.insert(sp_id, 0.0);
        } else {
            tile.populations.insert(sp_id, new_pop);
        }
    }
}

/// Compute predation pressure on a species from predators on the same tile
fn compute_predation(tile: &Tile, prey_species: &Species, all_species: &[Species]) -> f64 {
    let prey_pop = *tile.populations.get(&prey_species.id).unwrap_or(&0.0);
    if prey_pop <= 0.0 {
        return 0.0;
    }

    let prey_level = &prey_species.traits.trophic_level;
    let mut total_consumed = 0.0;

    for predator in all_species {
        // Predators eat consumers, consumers eat producers
        let eats_prey = match (&predator.traits.trophic_level, prey_level) {
            (TrophicLevel::Predator, TrophicLevel::Consumer) => true,
            (TrophicLevel::Consumer, TrophicLevel::Producer) => true,
            _ => false,
        };

        if !eats_prey {
            continue;
        }

        let pred_pop = *tile.populations.get(&predator.id).unwrap_or(&0.0);
        if pred_pop <= 0.0 {
            continue;
        }

        // Holling Type II functional response
        let a = 0.01;  // attack rate
        let h = 0.001; // handling time
        let consumed = a * pred_pop * prey_pop / (1.0 + h * prey_pop);
        total_consumed += consumed;
    }

    total_consumed
}

/// Update all biosphere populations across the grid
pub fn update_grid(grid: &mut WorldGrid, species: &[Species], atmo: &AtmosphereState) {
    let height = grid.height;
    let width = grid.width;

    for y in 0..height {
        for x in 0..width {
            let tile = grid.get_mut(x, y);
            update_tile_populations(tile, species, atmo);
        }
    }
}

/// Count total global population of a species across the grid
pub fn global_population(grid: &WorldGrid, species_id: u32) -> f64 {
    grid.tiles.iter()
        .map(|t| t.populations.get(&species_id).unwrap_or(&0.0))
        .sum()
}

/// Count how many distinct species have non-zero global population
pub fn biodiversity_count(grid: &WorldGrid, species: &[Species]) -> u32 {
    species.iter()
        .filter(|s| global_population(grid, s.id) > 0.0)
        .count() as u32
}
```

**Step 4: Run tests to verify they pass**

Run: `cd sim-core && cargo test --test biosphere_test`
Expected: All 6 tests PASS.

**Step 5: Commit**

```bash
git add sim-core/
git commit -m "feat: biosphere model with suitability, logistic growth, predation, and extinction"
```

---

## Task 7: Integrate Biosphere Into Simulation Loop

**Files:**
- Modify: `sim-core/src/sim.rs`
- Test: `sim-core/tests/integration_test.rs`

**Step 1: Write the failing test**

Create `sim-core/tests/integration_test.rs`:

```rust
use planet_architect_sim::sim::Simulation;
use planet_architect_sim::types::*;
use planet_architect_sim::biosphere;

#[test]
fn test_seeded_microbes_grow_over_time() {
    let mut params = PlanetParams::default();
    params.atmosphere.co2 = 0.01; // some greenhouse
    params.core_heat = 0.5;       // warm core, nutrients

    let mut sim = Simulation::new(42, params);

    // Seed a producer species
    let producer = Species {
        id: 0,
        name: "Proto-algae".to_string(),
        traits: SpeciesTraits {
            temp_optimal: 15.0,
            temp_range: 50.0,
            o2_need: 0.0,
            toxin_resistance: 0.1,
            trophic_level: TrophicLevel::Producer,
            reproduction_rate: 0.05,
            dispersal: 0.3,
            mutation_rate: 0.01,
        },
    };
    sim.add_species(producer.clone(), 100.0);

    let pop_before = biosphere::global_population(sim.grid(), 0);
    assert!(pop_before > 0.0, "Should have seeded population");

    sim.step(500);

    let pop_after = biosphere::global_population(sim.grid(), 0);
    assert!(pop_after > pop_before,
        "Population should grow over 500 steps: {} -> {}", pop_before, pop_after);
}

#[test]
fn test_food_web_predator_consumer_producer() {
    let mut params = PlanetParams::default();
    params.core_heat = 0.5;

    let mut sim = Simulation::new(42, params);

    let producer = Species {
        id: 0,
        name: "Algae".to_string(),
        traits: SpeciesTraits {
            temp_optimal: 15.0,
            temp_range: 50.0,
            o2_need: 0.0,
            toxin_resistance: 0.1,
            trophic_level: TrophicLevel::Producer,
            reproduction_rate: 0.08,
            dispersal: 0.2,
            mutation_rate: 0.01,
        },
    };
    sim.add_species(producer, 500.0);

    let consumer = Species {
        id: 1,
        name: "Grazer".to_string(),
        traits: SpeciesTraits {
            temp_optimal: 15.0,
            temp_range: 50.0,
            o2_need: 0.1,
            toxin_resistance: 0.1,
            trophic_level: TrophicLevel::Consumer,
            reproduction_rate: 0.03,
            dispersal: 0.3,
            mutation_rate: 0.01,
        },
    };
    sim.add_species(consumer, 50.0);

    sim.step(200);

    let prod_pop = biosphere::global_population(sim.grid(), 0);
    let cons_pop = biosphere::global_population(sim.grid(), 1);

    assert!(prod_pop > 0.0, "Producers should survive");
    assert!(cons_pop > 0.0, "Consumers should survive with producers");
    assert!(prod_pop > cons_pop, "Producers should outnumber consumers");
}
```

**Step 2: Run test to verify it fails**

Run: `cd sim-core && cargo test --test integration_test`
Expected: FAIL — `add_species` method not found.

**Step 3: Add add_species and biosphere integration to sim.rs**

Add to `Simulation` impl:

```rust
/// Add a species and seed it on habitable tiles
pub fn add_species(&mut self, species: Species, initial_pop_per_tile: f64) {
    let id = species.id;
    self.species.push(species.clone());

    // Seed on tiles where suitability > 0.3
    for y in 0..self.grid.height {
        for x in 0..self.grid.width {
            let tile = self.grid.get_mut(x, y);
            let suit = crate::biosphere::suitability(
                &species.traits,
                tile,
                &self.params.atmosphere,
            );
            if suit > 0.3 {
                tile.populations.insert(id, initial_pop_per_tile);
            }
        }
    }

    self.events.push(SimEvent::SpeciesAppeared {
        species_id: id,
        step: self.step_count,
    });

    if id >= self.next_species_id {
        self.next_species_id = id + 1;
    }
}
```

Update the `tick` method:

```rust
fn tick(&mut self) {
    self.step_count += 1;
    climate::update(&mut self.grid, &self.params, self.step_count);
    climate::update_nutrients(&mut self.grid, &self.params);
    biosphere::update_grid(&mut self.grid, &self.species, &self.params.atmosphere);
}
```

**Step 4: Run tests to verify they pass**

Run: `cd sim-core && cargo test --test integration_test`
Expected: Both tests PASS.

**Step 5: Run all tests**

Run: `cd sim-core && cargo test`
Expected: All tests PASS (types, determinism, climate, biosphere, integration).

**Step 6: Commit**

```bash
git add sim-core/
git commit -m "feat: integrate biosphere into simulation loop with species seeding"
```

---

## Task 8: Mutation & Speciation

**Files:**
- Modify: `sim-core/src/biosphere.rs`
- Modify: `sim-core/src/sim.rs`
- Test: `sim-core/tests/speciation_test.rs`

**Step 1: Write the failing test**

Create `sim-core/tests/speciation_test.rs`:

```rust
use planet_architect_sim::biosphere;
use planet_architect_sim::types::*;

#[test]
fn test_mutate_trait_stays_in_bounds() {
    let original = SpeciesTraits {
        temp_optimal: 20.0,
        temp_range: 40.0,
        o2_need: 0.1,
        toxin_resistance: 0.5,
        trophic_level: TrophicLevel::Producer,
        reproduction_rate: 0.05,
        dispersal: 0.3,
        mutation_rate: 0.01,
    };

    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let mutated = biosphere::mutate_traits(&original, &mut rng);

    // Traits should be slightly different but valid
    assert!(mutated.reproduction_rate >= 0.0);
    assert!(mutated.dispersal >= 0.0 && mutated.dispersal <= 1.0);
    assert!(mutated.toxin_resistance >= 0.0 && mutated.toxin_resistance <= 1.0);
    assert!(mutated.temp_range > 0.0);
}

#[test]
fn test_speciation_creates_new_species() {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let parent = Species {
        id: 0,
        name: "Proto-algae".to_string(),
        traits: SpeciesTraits {
            temp_optimal: 20.0,
            temp_range: 40.0,
            o2_need: 0.0,
            toxin_resistance: 0.1,
            trophic_level: TrophicLevel::Producer,
            reproduction_rate: 0.05,
            dispersal: 0.3,
            mutation_rate: 0.5, // high mutation for test
        },
    };

    let child = biosphere::try_speciate(&parent, 1, &mut rng);
    assert!(child.is_some(), "High mutation rate should produce speciation");
    let child = child.unwrap();
    assert_eq!(child.id, 1);
    assert_ne!(child.name, parent.name);
}
```

**Step 2: Run test to verify it fails**

Run: `cd sim-core && cargo test --test speciation_test`
Expected: FAIL — `mutate_traits` and `try_speciate` not found.

**Step 3: Add mutation and speciation to biosphere.rs**

```rust
use rand::Rng;
use rand_chacha::ChaCha8Rng;

/// Slightly mutate species traits
pub fn mutate_traits(original: &SpeciesTraits, rng: &mut ChaCha8Rng) -> SpeciesTraits {
    let mut t = original.clone();

    let nudge = |val: f32, range: f32, rng: &mut ChaCha8Rng| -> f32 {
        val + rng.gen_range(-range..range)
    };

    t.temp_optimal = nudge(t.temp_optimal, 3.0, rng);
    t.temp_range = nudge(t.temp_range, 2.0, rng).max(5.0);
    t.o2_need = nudge(t.o2_need, 0.02, rng).max(0.0);
    t.toxin_resistance = nudge(t.toxin_resistance, 0.05, rng).clamp(0.0, 1.0);
    t.reproduction_rate = nudge(t.reproduction_rate, 0.005, rng).max(0.001);
    t.dispersal = nudge(t.dispersal, 0.05, rng).clamp(0.0, 1.0);
    t.mutation_rate = nudge(t.mutation_rate, 0.002, rng).clamp(0.001, 1.0);
    // trophic_level stays the same (no level jumps in MVP)
    t
}

/// Attempt speciation. Returns Some(new_species) if mutation fires.
pub fn try_speciate(
    parent: &Species,
    new_id: u32,
    rng: &mut ChaCha8Rng,
) -> Option<Species> {
    let roll: f32 = rng.gen();
    if roll < parent.traits.mutation_rate {
        let new_traits = mutate_traits(&parent.traits, rng);
        let name = format!("{}-v{}", parent.name, new_id);
        Some(Species {
            id: new_id,
            name,
            traits: new_traits,
        })
    } else {
        None
    }
}
```

**Step 4: Integrate speciation into sim.rs tick loop**

Add an epoch-based speciation check (every 1000 steps):

```rust
// In tick(), after biosphere update:
const SPECIATION_EPOCH: u64 = 1000;

if self.step_count % SPECIATION_EPOCH == 0 {
    self.check_speciation();
}
```

Add the method:

```rust
fn check_speciation(&mut self) {
    let mut new_species = Vec::new();

    for species in &self.species {
        let global_pop = biosphere::global_population(&self.grid, species.id);
        if global_pop > 500.0 {
            if let Some(child) = biosphere::try_speciate(
                species,
                self.next_species_id,
                &mut self.rng,
            ) {
                self.events.push(SimEvent::Speciation {
                    parent_id: species.id,
                    child_id: child.id,
                    step: self.step_count,
                });
                self.next_species_id += 1;
                new_species.push(child);
            }
        }
    }

    // Seed new species on tiles where parent exists
    for child in &new_species {
        let parent_id = child.id - 1; // simplified
        for y in 0..self.grid.height {
            for x in 0..self.grid.width {
                let tile = self.grid.get_mut(x, y);
                if let Some(&parent_pop) = tile.populations.get(&parent_id) {
                    if parent_pop > 10.0 {
                        tile.populations.insert(child.id, parent_pop * 0.1);
                    }
                }
            }
        }
    }

    self.species.extend(new_species);
}
```

**Step 5: Run tests**

Run: `cd sim-core && cargo test`
Expected: All tests PASS.

**Step 6: Commit**

```bash
git add sim-core/
git commit -m "feat: mutation and speciation engine with epoch-based checks"
```

---

## Task 9: Intervention System

**Files:**
- Modify: `sim-core/src/sim.rs`
- Test: `sim-core/tests/intervention_test.rs`

**Step 1: Write the failing test**

Create `sim-core/tests/intervention_test.rs`:

```rust
use planet_architect_sim::sim::Simulation;
use planet_architect_sim::types::*;

#[test]
fn test_adjust_co2_changes_atmosphere() {
    let mut sim = Simulation::new(42, PlanetParams::default());
    let co2_before = sim.params().atmosphere.co2;

    let intervention = Intervention {
        kind: InterventionKind::AdjustCO2 { delta: 0.01 },
        target_region: None,
        step: 0,
    };

    let result = sim.apply_intervention(intervention);
    assert!(result.is_ok());

    let co2_after = sim.params().atmosphere.co2;
    assert!((co2_after - co2_before - 0.01).abs() < 0.0001,
        "CO2 should increase by delta: {} -> {}", co2_before, co2_after);
}

#[test]
fn test_adjust_o2_changes_atmosphere() {
    let mut sim = Simulation::new(42, PlanetParams::default());
    let o2_before = sim.params().atmosphere.o2;

    let intervention = Intervention {
        kind: InterventionKind::AdjustO2 { delta: 0.05 },
        target_region: None,
        step: 0,
    };

    sim.apply_intervention(intervention).unwrap();
    let o2_after = sim.params().atmosphere.o2;
    assert!((o2_after - o2_before - 0.05).abs() < 0.0001);
}

#[test]
fn test_nutrient_bloom_increases_nutrients() {
    let mut sim = Simulation::new(42, PlanetParams::default());

    let target = RegionTarget { x: 32, y: 16, radius: 3 };
    let nutrients_before = sim.grid().get(32, 16).nutrients;

    let intervention = Intervention {
        kind: InterventionKind::NutrientBloom { magnitude: 0.5 },
        target_region: Some(target),
        step: 0,
    };

    sim.apply_intervention(intervention).unwrap();
    let nutrients_after = sim.grid().get(32, 16).nutrients;
    assert!(nutrients_after > nutrients_before,
        "Nutrient bloom should increase nutrients: {} -> {}", nutrients_before, nutrients_after);
}

#[test]
fn test_intervention_values_clamped() {
    let mut sim = Simulation::new(42, PlanetParams::default());

    // Try to set CO2 to absurd values
    let intervention = Intervention {
        kind: InterventionKind::AdjustCO2 { delta: 100.0 },
        target_region: None,
        step: 0,
    };

    sim.apply_intervention(intervention).unwrap();
    assert!(sim.params().atmosphere.co2 <= 1.0, "CO2 should be clamped");
}
```

**Step 2: Run test to verify it fails**

Run: `cd sim-core && cargo test --test intervention_test`
Expected: FAIL — `apply_intervention` not found.

**Step 3: Implement apply_intervention in sim.rs**

```rust
#[derive(Debug)]
pub enum InterventionError {
    InvalidRegion,
}

impl Simulation {
    pub fn apply_intervention(&mut self, intervention: Intervention) -> Result<(), InterventionError> {
        match intervention.kind {
            InterventionKind::AdjustCO2 { delta } => {
                self.params.atmosphere.co2 = (self.params.atmosphere.co2 + delta).clamp(0.0, 1.0);
            }
            InterventionKind::AdjustO2 { delta } => {
                self.params.atmosphere.o2 = (self.params.atmosphere.o2 + delta).clamp(0.0, 1.0);
            }
            InterventionKind::CloudSeeding { magnitude } => {
                if let Some(region) = &intervention.target_region {
                    self.apply_to_region(region, |tile| {
                        tile.moisture = (tile.moisture + magnitude * 0.3).clamp(0.0, 1.0);
                    });
                }
            }
            InterventionKind::NutrientBloom { magnitude } => {
                if let Some(region) = &intervention.target_region {
                    self.apply_to_region(region, |tile| {
                        tile.nutrients = (tile.nutrients + magnitude).clamp(0.0, 1.0);
                    });
                }
            }
            InterventionKind::IceMeltPulse { magnitude } => {
                self.params.hydrology.ice_fraction =
                    (self.params.hydrology.ice_fraction - magnitude * 0.1).clamp(0.0, 1.0);
            }
        }
        Ok(())
    }

    fn apply_to_region<F>(&mut self, region: &RegionTarget, mut f: F)
    where
        F: FnMut(&mut Tile),
    {
        let r = region.radius as isize;
        for dy in -r..=r {
            for dx in -r..=r {
                let x = (region.x as isize + dx) as usize;
                let y = (region.y as isize + dy) as usize;
                if x < self.grid.width && y < self.grid.height {
                    let tile = self.grid.get_mut(x, y);
                    f(tile);
                }
            }
        }
    }
}
```

**Step 4: Run tests**

Run: `cd sim-core && cargo test --test intervention_test`
Expected: All 4 tests PASS.

**Step 5: Commit**

```bash
git add sim-core/
git commit -m "feat: intervention system — CO2, O2, cloud seeding, nutrient bloom, ice melt"
```

---

## Task 10: Level Spec & Objective Evaluator

**Files:**
- Create: `sim-core/src/level.rs`
- Create: `sim-core/levels/level_01_first_breath.json`
- Test: `sim-core/tests/level_test.rs`

**Step 1: Write the failing test**

Create `sim-core/tests/level_test.rs`:

```rust
use planet_architect_sim::level::*;
use planet_architect_sim::types::*;

#[test]
fn test_load_level_spec() {
    let json = r#"{
        "id": "level_01",
        "name": "First Breath",
        "pack": "FREE",
        "description": "Establish microbial life for 10M years",
        "starting_seed": 42,
        "starting_params": null,
        "allowed_interventions": ["AdjustCO2", "AdjustO2", "NutrientBloom"],
        "energy_budget": 100.0,
        "objective": {
            "type": "MicrobialStability",
            "min_biomass": 1000.0,
            "required_duration_steps": 10000000
        },
        "fail_conditions": ["Extinction"]
    }"#;

    let spec: LevelSpec = serde_json::from_str(json).unwrap();
    assert_eq!(spec.id, "level_01");
    assert_eq!(spec.name, "First Breath");
    assert_eq!(spec.pack, Pack::Free);
}

#[test]
fn test_objective_not_met_initially() {
    let objective = Objective::MicrobialStability {
        min_biomass: 1000.0,
        required_duration_steps: 100,
    };
    let mut eval = ObjectiveEvaluator::new(objective);

    // No biomass at start
    let status = eval.evaluate(0.0, 0, 0);
    assert_eq!(status, ObjectiveStatus::InProgress);
}

#[test]
fn test_objective_met_after_sustained_biomass() {
    let objective = Objective::MicrobialStability {
        min_biomass: 100.0,
        required_duration_steps: 10,
    };
    let mut eval = ObjectiveEvaluator::new(objective);

    // Sustain biomass for 10 steps
    for step in 0..10 {
        let status = eval.evaluate(500.0, 1, step);
        if step < 9 {
            assert_eq!(status, ObjectiveStatus::InProgress);
        }
    }
    let final_status = eval.evaluate(500.0, 1, 10);
    assert_eq!(final_status, ObjectiveStatus::Complete);
}

#[test]
fn test_objective_fails_on_extinction() {
    let objective = Objective::MicrobialStability {
        min_biomass: 100.0,
        required_duration_steps: 100,
    };
    let mut eval = ObjectiveEvaluator::new(objective);

    eval.evaluate(500.0, 1, 0); // progress
    let status = eval.evaluate(0.0, 0, 1); // extinction
    assert_eq!(status, ObjectiveStatus::Failed);
}
```

**Step 2: Run test to verify it fails**

Run: `cd sim-core && cargo test --test level_test`
Expected: FAIL — `level` module empty.

**Step 3: Implement level.rs**

```rust
use serde::{Deserialize, Serialize};
use crate::types::PlanetParams;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Pack {
    #[serde(rename = "FREE")]
    Free,
    #[serde(rename = "PACK_CORE")]
    Core,
    #[serde(rename = "PACK_ADV")]
    Advanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelSpec {
    pub id: String,
    pub name: String,
    pub pack: Pack,
    pub description: String,
    pub starting_seed: u64,
    pub starting_params: Option<PlanetParams>,
    pub allowed_interventions: Vec<String>,
    pub energy_budget: f32,
    pub objective: Objective,
    pub fail_conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Objective {
    MicrobialStability {
        min_biomass: f64,
        required_duration_steps: u64,
    },
    EcosystemStability {
        min_trophic_levels: u32,
        required_duration_steps: u64,
    },
    BiodiversityStability {
        min_species: u32,
        max_climate_variance: f32,
        required_duration_steps: u64,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectiveStatus {
    InProgress,
    Complete,
    Failed,
}

pub struct ObjectiveEvaluator {
    objective: Objective,
    sustained_steps: u64,
}

impl ObjectiveEvaluator {
    pub fn new(objective: Objective) -> Self {
        Self {
            objective,
            sustained_steps: 0,
        }
    }

    /// Evaluate objective progress given current simulation state.
    /// Returns the status after this evaluation.
    pub fn evaluate(
        &mut self,
        total_biomass: f64,
        biodiversity: u32,
        _current_step: u64,
    ) -> ObjectiveStatus {
        match &self.objective {
            Objective::MicrobialStability { min_biomass, required_duration_steps } => {
                if total_biomass <= 0.0 && self.sustained_steps > 0 {
                    return ObjectiveStatus::Failed; // extinction after progress
                }
                if total_biomass >= *min_biomass {
                    self.sustained_steps += 1;
                    if self.sustained_steps >= *required_duration_steps {
                        return ObjectiveStatus::Complete;
                    }
                } else {
                    self.sustained_steps = 0; // reset if below threshold
                }
                ObjectiveStatus::InProgress
            }
            Objective::EcosystemStability { min_trophic_levels, required_duration_steps } => {
                // Simplified: treat biodiversity as proxy for trophic levels
                if biodiversity >= *min_trophic_levels {
                    self.sustained_steps += 1;
                    if self.sustained_steps >= *required_duration_steps {
                        return ObjectiveStatus::Complete;
                    }
                } else {
                    self.sustained_steps = 0;
                }
                ObjectiveStatus::InProgress
            }
            Objective::BiodiversityStability { min_species, required_duration_steps, .. } => {
                if biodiversity >= *min_species {
                    self.sustained_steps += 1;
                    if self.sustained_steps >= *required_duration_steps {
                        return ObjectiveStatus::Complete;
                    }
                } else {
                    self.sustained_steps = 0;
                }
                ObjectiveStatus::InProgress
            }
        }
    }

    pub fn sustained_steps(&self) -> u64 {
        self.sustained_steps
    }
}
```

**Step 4: Run tests**

Run: `cd sim-core && cargo test --test level_test`
Expected: All 4 tests PASS.

**Step 5: Create Level 1 JSON spec**

Create `sim-core/levels/` directory and `level_01_first_breath.json`:

```json
{
    "id": "level_01",
    "name": "First Breath",
    "pack": "FREE",
    "description": "A barren rocky planet with a thin CO2 atmosphere, frozen oceans, and a weak magnetic field. Establish stable microbial life for 10 million years.",
    "starting_seed": 7749,
    "starting_params": {
        "gravity": 9.2,
        "rotation_rate": 1.1,
        "axial_tilt": 18.0,
        "core_heat": 0.2,
        "magnetic_field": 0.2,
        "atmosphere": {
            "pressure": 0.4,
            "o2": 0.01,
            "co2": 0.15,
            "toxicity": 0.1
        },
        "hydrology": {
            "ocean_coverage": 0.3,
            "salinity": 0.04,
            "current_strength": 0.2,
            "ice_fraction": 0.6
        }
    },
    "allowed_interventions": ["AdjustCO2", "AdjustO2", "NutrientBloom", "IceMeltPulse"],
    "energy_budget": 50.0,
    "objective": {
        "type": "MicrobialStability",
        "min_biomass": 5000.0,
        "required_duration_steps": 10000000
    },
    "fail_conditions": ["Extinction"]
}
```

**Step 6: Commit**

```bash
mkdir -p sim-core/levels
git add sim-core/
git commit -m "feat: level spec format, objective evaluator, and Level 1 JSON"
```

---

## Task 11: Snapshot & Save/Load

**Files:**
- Modify: `sim-core/src/snapshot.rs`
- Modify: `sim-core/src/sim.rs`
- Test: `sim-core/tests/saveload_test.rs`

**Step 1: Write the failing test**

Create `sim-core/tests/saveload_test.rs`:

```rust
use planet_architect_sim::sim::Simulation;
use planet_architect_sim::types::*;

#[test]
fn test_save_and_load_produces_identical_state() {
    let mut sim = Simulation::new(42, PlanetParams::default());

    let producer = Species {
        id: 0,
        name: "Algae".to_string(),
        traits: SpeciesTraits {
            temp_optimal: 15.0,
            temp_range: 50.0,
            o2_need: 0.0,
            toxin_resistance: 0.1,
            trophic_level: TrophicLevel::Producer,
            reproduction_rate: 0.05,
            dispersal: 0.3,
            mutation_rate: 0.01,
        },
    };
    sim.add_species(producer, 100.0);
    sim.step(500);

    // Save
    let bytes = sim.save_state().expect("save should work");

    // Load into new simulation
    let loaded = Simulation::load_state(&bytes).expect("load should work");

    assert_eq!(sim.current_step(), loaded.current_step());

    // Continue both from same point
    let snap1 = sim.snapshot();
    let snap2 = loaded.snapshot();

    for i in 0..snap1.grid.tiles.len() {
        assert_eq!(
            snap1.grid.tiles[i].temperature,
            snap2.grid.tiles[i].temperature,
            "Tile {} temperature mismatch after load", i
        );
    }
}

#[test]
fn test_loaded_sim_continues_deterministically() {
    let mut sim = Simulation::new(42, PlanetParams::default());
    sim.step(100);

    let bytes = sim.save_state().unwrap();
    let mut loaded = Simulation::load_state(&bytes).unwrap();

    // Continue both for 100 more steps
    sim.step(100);
    loaded.step(100);

    let snap1 = sim.snapshot();
    let snap2 = loaded.snapshot();

    assert_eq!(snap1.current_step, snap2.current_step);
    for i in 0..snap1.grid.tiles.len() {
        assert_eq!(snap1.grid.tiles[i].temperature, snap2.grid.tiles[i].temperature);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd sim-core && cargo test --test saveload_test`
Expected: FAIL — `save_state`/`load_state` not found.

**Step 3: Implement save/load in sim.rs**

Add a serializable state struct and save/load methods:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct SimState {
    seed: u64,
    step_count: u64,
    params: PlanetParams,
    grid: WorldGrid,
    species: Vec<Species>,
    events: Vec<SimEvent>,
    next_species_id: u32,
    rng_state: Vec<u8>, // serialized ChaCha8Rng state
}

impl Simulation {
    pub fn save_state(&self) -> Result<Vec<u8>, bincode::Error> {
        let rng_bytes = bincode::serialize(&self.rng)?;
        let state = SimState {
            seed: self.seed,
            step_count: self.step_count,
            params: self.params.clone(),
            grid: self.grid.clone(),
            species: self.species.clone(),
            events: self.events.clone(),
            next_species_id: self.next_species_id,
            rng_state: rng_bytes,
        };
        bincode::serialize(&state)
    }

    pub fn load_state(bytes: &[u8]) -> Result<Self, bincode::Error> {
        let state: SimState = bincode::deserialize(bytes)?;
        let rng: ChaCha8Rng = bincode::deserialize(&state.rng_state)?;
        Ok(Self {
            seed: state.seed,
            rng,
            step_count: state.step_count,
            params: state.params,
            grid: state.grid,
            species: state.species,
            events: state.events,
            next_species_id: state.next_species_id,
        })
    }
}
```

Note: `ChaCha8Rng` implements `Serialize`/`Deserialize` via `rand_chacha` with `serde` feature. Update `Cargo.toml`:

```toml
rand_chacha = { version = "0.3", features = ["serde1"] }
```

**Step 4: Run tests**

Run: `cd sim-core && cargo test --test saveload_test`
Expected: Both tests PASS.

**Step 5: Commit**

```bash
git add sim-core/
git commit -m "feat: save/load with deterministic RNG state preservation"
```

---

## Task 12: Codex Trigger System

**Files:**
- Create: `sim-core/src/codex.rs`
- Test: `sim-core/tests/codex_test.rs`

**Step 1: Write the failing test**

Create `sim-core/tests/codex_test.rs`:

```rust
use planet_architect_sim::codex::*;

#[test]
fn test_codex_entry_deserialization() {
    let json = r#"{
        "id": "body_plan_001",
        "category": "BodyPlan",
        "name": "Single-Cell Photosynthesis",
        "unlock_trigger": { "type": "TraitStabilized", "trait_name": "photosynthesis", "min_duration": 1000 },
        "requirements_text": "Sustain a photosynthetic species for 1000 years",
        "facts_text": "The first organisms to harvest starlight.",
        "flavor_text": "A tiny cell turns toward a distant sun — and everything changes.",
        "related_entry_ids": ["species_001"],
        "icon_asset_id": "icon_photosynthesis"
    }"#;

    let entry: CodexEntry = serde_json::from_str(json).unwrap();
    assert_eq!(entry.id, "body_plan_001");
    assert_eq!(entry.category, CodexCategory::BodyPlan);
}

#[test]
fn test_codex_tracker_unlocks_on_condition() {
    let entry = CodexEntry {
        id: "species_first_life".to_string(),
        category: CodexCategory::Species,
        name: "First Life".to_string(),
        unlock_trigger: UnlockTrigger::SpeciesAppeared,
        requirements_text: "Any species appears".to_string(),
        facts_text: "Life finds a way.".to_string(),
        flavor_text: "From chemistry to biology, in a single step.".to_string(),
        related_entry_ids: vec![],
        icon_asset_id: "icon_first_life".to_string(),
    };

    let mut tracker = CodexTracker::new(vec![entry]);
    assert!(tracker.unlocked_ids().is_empty());

    // Signal that a species appeared
    let unlocks = tracker.check_species_appeared();
    assert_eq!(unlocks.len(), 1);
    assert_eq!(unlocks[0], "species_first_life");

    // Should not unlock twice
    let unlocks2 = tracker.check_species_appeared();
    assert!(unlocks2.is_empty());
}
```

**Step 2: Run test to verify it fails**

Run: `cd sim-core && cargo test --test codex_test`
Expected: FAIL — `codex` module empty.

**Step 3: Implement codex.rs**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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

pub struct CodexTracker {
    entries: Vec<CodexEntry>,
    unlocked: HashSet<String>,
}

impl CodexTracker {
    pub fn new(entries: Vec<CodexEntry>) -> Self {
        Self {
            entries,
            unlocked: HashSet::new(),
        }
    }

    pub fn unlocked_ids(&self) -> Vec<String> {
        self.unlocked.iter().cloned().collect()
    }

    pub fn check_species_appeared(&mut self) -> Vec<String> {
        self.check_trigger(|t| matches!(t, UnlockTrigger::SpeciesAppeared))
    }

    pub fn check_speciation(&mut self) -> Vec<String> {
        self.check_trigger(|t| matches!(t, UnlockTrigger::SpeciationEvent))
    }

    pub fn check_mass_extinction(&mut self) -> Vec<String> {
        self.check_trigger(|t| matches!(t, UnlockTrigger::MassExtinction))
    }

    pub fn check_biodiversity(&mut self, count: u32) -> Vec<String> {
        self.check_trigger(|t| {
            matches!(t, UnlockTrigger::BiodiversityThreshold { min_species } if count >= *min_species)
        })
    }

    fn check_trigger<F>(&mut self, predicate: F) -> Vec<String>
    where
        F: Fn(&UnlockTrigger) -> bool,
    {
        let mut newly_unlocked = Vec::new();
        for entry in &self.entries {
            if !self.unlocked.contains(&entry.id) && predicate(&entry.unlock_trigger) {
                newly_unlocked.push(entry.id.clone());
            }
        }
        for id in &newly_unlocked {
            self.unlocked.insert(id.clone());
        }
        newly_unlocked
    }
}
```

**Step 4: Run tests**

Run: `cd sim-core && cargo test --test codex_test`
Expected: Both tests PASS.

**Step 5: Commit**

```bash
git add sim-core/
git commit -m "feat: codex system with entry definitions, unlock triggers, and tracker"
```

---

## Task 13: Full Integration Test — Level 1 Headless Run

**Files:**
- Test: `sim-core/tests/level1_headless_test.rs`

**Step 1: Write the integration test**

Create `sim-core/tests/level1_headless_test.rs`:

```rust
use planet_architect_sim::sim::Simulation;
use planet_architect_sim::level::*;
use planet_architect_sim::biosphere;
use planet_architect_sim::types::*;

#[test]
fn test_level1_can_be_loaded_and_started() {
    let json = include_str!("../levels/level_01_first_breath.json");
    let spec: LevelSpec = serde_json::from_str(json).unwrap();

    let params = spec.starting_params.unwrap_or_default();
    let sim = Simulation::new(spec.starting_seed, params);

    assert_eq!(sim.current_step(), 0);
}

#[test]
fn test_level1_determinism_across_runs() {
    let json = include_str!("../levels/level_01_first_breath.json");
    let spec: LevelSpec = serde_json::from_str(json).unwrap();
    let params = spec.starting_params.unwrap_or_default();

    let mut sim1 = Simulation::new(spec.starting_seed, params.clone());
    let mut sim2 = Simulation::new(spec.starting_seed, params);

    // Seed identical species
    let microbe = Species {
        id: 0,
        name: "Thermophile".to_string(),
        traits: SpeciesTraits {
            temp_optimal: 5.0,
            temp_range: 60.0,
            o2_need: 0.0,
            toxin_resistance: 0.3,
            trophic_level: TrophicLevel::Producer,
            reproduction_rate: 0.04,
            dispersal: 0.2,
            mutation_rate: 0.005,
        },
    };

    sim1.add_species(microbe.clone(), 50.0);
    sim2.add_species(microbe, 50.0);

    sim1.step(5000);
    sim2.step(5000);

    let pop1 = biosphere::global_population(sim1.grid(), 0);
    let pop2 = biosphere::global_population(sim2.grid(), 0);

    assert_eq!(pop1, pop2, "Same seed + same species should give identical populations");
    assert_eq!(sim1.current_step(), sim2.current_step());
}

#[test]
fn test_level1_microbes_can_survive() {
    let json = include_str!("../levels/level_01_first_breath.json");
    let spec: LevelSpec = serde_json::from_str(json).unwrap();
    let mut params = spec.starting_params.unwrap_or_default();

    // Player interventions to make planet habitable:
    // Increase CO2 for greenhouse warming, reduce ice
    params.atmosphere.co2 = 0.05;
    params.hydrology.ice_fraction = 0.2;
    params.core_heat = 0.4;

    let mut sim = Simulation::new(spec.starting_seed, params);

    let microbe = Species {
        id: 0,
        name: "Extremophile".to_string(),
        traits: SpeciesTraits {
            temp_optimal: 0.0,
            temp_range: 60.0,
            o2_need: 0.0,
            toxin_resistance: 0.3,
            trophic_level: TrophicLevel::Producer,
            reproduction_rate: 0.04,
            dispersal: 0.3,
            mutation_rate: 0.005,
        },
    };
    sim.add_species(microbe, 100.0);

    sim.step(10_000);

    let pop = biosphere::global_population(sim.grid(), 0);
    assert!(pop > 0.0, "Microbes should survive with player intervention: pop={}", pop);

    let snap = sim.snapshot();
    assert!(snap.biodiversity_count >= 1);
}

#[test]
fn test_level1_objective_evaluator_integration() {
    let json = include_str!("../levels/level_01_first_breath.json");
    let spec: LevelSpec = serde_json::from_str(json).unwrap();

    let mut eval = ObjectiveEvaluator::new(spec.objective.clone());

    // Simulate objective tracking with fake data
    let status = eval.evaluate(0.0, 0, 0);
    assert_eq!(status, ObjectiveStatus::InProgress);

    // Sustain above threshold
    for step in 1..=20 {
        eval.evaluate(10000.0, 3, step);
    }
    assert!(eval.sustained_steps() >= 20);
}
```

**Step 2: Run tests**

Run: `cd sim-core && cargo test --test level1_headless_test`
Expected: All 4 tests PASS.

**Step 3: Run ALL tests**

Run: `cd sim-core && cargo test`
Expected: All tests across all test files PASS.

**Step 4: Commit**

```bash
git add sim-core/
git commit -m "test: Level 1 headless integration tests — determinism, survival, objectives"
```

---

## Task 14: Swift Bridge Types & App Shell

**Files:**
- Create: `ios/PlanetArchitect/Core/SimulationBridge/SimTypes.swift`
- Create: `ios/PlanetArchitect/PlanetArchitectApp.swift`
- Create: `ios/PlanetArchitect/Features/Campaign/LevelSelectView.swift`
- Create: `ios/PlanetArchitect/Features/Planet/PlanetView.swift`

**Step 1: Create Swift data models mirroring Rust types**

Create `ios/PlanetArchitect/Core/SimulationBridge/SimTypes.swift`:

```swift
import Foundation

// MARK: - Planet Parameters

struct PlanetParams: Codable {
    var gravity: Float = 9.8
    var rotationRate: Float = 1.0
    var axialTilt: Float = 23.4
    var coreHeat: Float = 0.4
    var magneticField: Float = 0.6
    var atmosphere: AtmosphereState = .init()
    var hydrology: HydroState = .init()
}

struct AtmosphereState: Codable {
    var pressure: Float = 1.0
    var o2: Float = 0.21
    var co2: Float = 0.0004
    var toxicity: Float = 0.0
}

struct HydroState: Codable {
    var oceanCoverage: Float = 0.7
    var salinity: Float = 0.035
    var currentStrength: Float = 0.5
    var iceFraction: Float = 0.1
}

// MARK: - Time

enum TimeSpeed: String, Codable {
    case observe   // 1x
    case adapt     // 100x
    case epoch     // 10_000x
    case eon       // 1_000_000x

    var stepsPerBatch: UInt64 {
        switch self {
        case .observe: return 1
        case .adapt: return 100
        case .epoch: return 10_000
        case .eon: return 1_000_000
        }
    }
}

// MARK: - Species

enum TrophicLevel: String, Codable {
    case producer
    case consumer
    case predator
}

struct SpeciesTraits: Codable {
    var tempOptimal: Float
    var tempRange: Float
    var o2Need: Float
    var toxinResistance: Float
    var trophicLevel: TrophicLevel
    var reproductionRate: Float
    var dispersal: Float
    var mutationRate: Float
}

struct Species: Codable, Identifiable {
    let id: UInt32
    var name: String
    var traits: SpeciesTraits
}

// MARK: - Interventions

enum InterventionKind: Codable {
    case adjustCO2(delta: Float)
    case adjustO2(delta: Float)
    case cloudSeeding(magnitude: Float)
    case nutrientBloom(magnitude: Float)
    case iceMeltPulse(magnitude: Float)
}

struct RegionTarget: Codable {
    let x: Int
    let y: Int
    let radius: Int
}

struct Intervention: Codable {
    let kind: InterventionKind
    let targetRegion: RegionTarget?
    let step: UInt64
}

// MARK: - Level Spec

enum Pack: String, Codable {
    case free = "FREE"
    case core = "PACK_CORE"
    case advanced = "PACK_ADV"
}

struct LevelSpec: Codable, Identifiable {
    let id: String
    let name: String
    let pack: Pack
    let description: String
    let startingSeed: UInt64
}

// MARK: - Codex

enum CodexCategory: String, Codable {
    case species
    case bodyPlan
    case biome
    case planetarySystem
    case evolutionaryEvent
    case failureMode
    case rarePhenomenon
    case historicWorld
}

struct CodexEntry: Codable, Identifiable {
    let id: String
    let category: CodexCategory
    let name: String
    let requirementsText: String
    let factsText: String
    let flavorText: String
    let relatedEntryIds: [String]
    let iconAssetId: String
}

// MARK: - Snapshot (received from sim-core)

struct SimSnapshot {
    let currentStep: UInt64
    let biodiversityCount: UInt32
    let species: [Species]
    // Tile data will be passed as flat arrays for Metal rendering
}
```

**Step 2: Create app entry point**

Create `ios/PlanetArchitect/PlanetArchitectApp.swift`:

```swift
import SwiftUI

@main
struct PlanetArchitectApp: App {
    var body: some Scene {
        WindowGroup {
            LevelSelectView()
        }
    }
}
```

**Step 3: Create Level Select placeholder**

Create `ios/PlanetArchitect/Features/Campaign/LevelSelectView.swift`:

```swift
import SwiftUI

struct LevelSelectView: View {
    let levels: [(id: Int, name: String, pack: Pack)] = [
        (1, "First Breath", .free),
        (2, "Shallow Seas", .free),
        (3, "Fragile Balance", .free),
        (4, "High Gravity Hell", .core),
        (5, "Toxic Skies", .core),
        (6, "Rogue Moon", .core),
        (7, "Crimson Star", .core),
        (8, "Desert Bloom", .core),
        (9, "Frozen Heart", .core),
        (10, "The Long Night", .core),
    ]

    var body: some View {
        NavigationStack {
            List {
                Section("Training") {
                    ForEach(levels.filter { $0.pack == .free }, id: \.id) { level in
                        NavigationLink {
                            PlanetView(levelId: level.id)
                        } label: {
                            LevelRow(number: level.id, name: level.name, locked: false)
                        }
                    }
                }
                Section("Core Challenge Pack") {
                    ForEach(levels.filter { $0.pack == .core }, id: \.id) { level in
                        LevelRow(number: level.id, name: level.name, locked: true)
                    }
                }
            }
            .navigationTitle("Planet Architect")
        }
    }
}

struct LevelRow: View {
    let number: Int
    let name: String
    let locked: Bool

    var body: some View {
        HStack {
            Text("\(number)")
                .font(.headline)
                .frame(width: 30)
            Text(name)
            Spacer()
            if locked {
                Image(systemName: "lock.fill")
                    .foregroundColor(.secondary)
            }
        }
    }
}
```

**Step 4: Create Planet View placeholder**

Create `ios/PlanetArchitect/Features/Planet/PlanetView.swift`:

```swift
import SwiftUI

struct PlanetView: View {
    let levelId: Int
    @State private var timeSpeed: TimeSpeed = .observe
    @State private var currentStep: UInt64 = 0
    @State private var isPaused: Bool = true

    var body: some View {
        VStack {
            // Planet visualization placeholder
            ZStack {
                Circle()
                    .fill(
                        RadialGradient(
                            colors: [.blue, .green, .brown],
                            center: .center,
                            startRadius: 50,
                            endRadius: 150
                        )
                    )
                    .frame(width: 300, height: 300)

                Text("Level \(levelId)")
                    .font(.title2)
                    .foregroundColor(.white)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)

            // Time controls
            HStack(spacing: 20) {
                Button(isPaused ? "Play" : "Pause") {
                    isPaused.toggle()
                }

                Picker("Speed", selection: $timeSpeed) {
                    Text("1x").tag(TimeSpeed.observe)
                    Text("100x").tag(TimeSpeed.adapt)
                    Text("10K").tag(TimeSpeed.epoch)
                    Text("1M").tag(TimeSpeed.eon)
                }
                .pickerStyle(.segmented)
            }
            .padding()

            Text("Step: \(currentStep)")
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .navigationTitle("Planet View")
        .navigationBarTitleDisplayMode(.inline)
    }
}
```

**Step 5: Commit**

```bash
git add ios/
git commit -m "feat: Swift data models mirroring Rust types + SwiftUI app shell with level select and planet view"
```

---

## Task 15: Final — Full Test Suite, CHANGELOG, Push

**Files:**
- Modify: `docs/CHANGELOG.md`

**Step 1: Run complete test suite**

Run: `cd sim-core && cargo test`
Expected: All tests pass.

Run: `cd sim-core && cargo clippy -- -W warnings`
Expected: No errors (warnings acceptable for now).

**Step 2: Update CHANGELOG**

```markdown
# Changelog

## 0.2.0 - 2026-02-24
- Project scaffold with full design documents
- Rust sim-core crate initialized
- Core data structures: PlanetParams, WorldGrid, Tile, Species, Interventions
- Deterministic tick loop with ChaCha8 seeded RNG
- Climate model: temperature, greenhouse, albedo, seasons, nutrients
- Biosphere model: suitability, logistic growth, carrying capacity, predation
- Mutation and speciation engine
- Intervention system: CO2, O2, cloud seeding, nutrient bloom, ice melt
- Level spec JSON format with objective evaluator
- Save/load with deterministic RNG state preservation
- Codex system with entry definitions and unlock triggers
- Level 1 ("First Breath") JSON spec + headless integration tests
- Swift bridge types mirroring Rust structures
- SwiftUI app shell: LevelSelectView + PlanetView placeholders

## 0.1.0 - 2026-02-24
- Initial project setup
- Added README, .gitignore, and docs structure
```

**Step 3: Commit**

```bash
git add docs/CHANGELOG.md
git commit -m "docs: update changelog for v0.2.0 — sim-core + app shell"
```

**Step 4: Push to GitHub**

```bash
git push origin main
```

---

## Summary: What This Plan Builds

| Component | Status After Plan |
|-----------|------------------|
| Repo scaffold + design docs | Complete |
| Rust sim-core crate | Complete — compiles, tests pass |
| Climate model | Temperature, greenhouse, albedo, seasons, nutrients |
| Biosphere model | Producers, consumers, predators, logistic growth, predation |
| Speciation | Epoch-based mutation + species splitting |
| Interventions | CO2, O2, cloud seeding, nutrient bloom, ice melt |
| Level system | JSON spec format + objective evaluator + Level 1 authored |
| Save/load | Deterministic state serialization via bincode |
| Codex | Entry format + unlock trigger tracker |
| Swift types | Bridge-ready data models |
| SwiftUI shell | Level select + planet view placeholders |
| Test coverage | Determinism, climate, biosphere, integration, save/load, codex |

## What Comes Next (Sprint 2+)

- **Sprint 2:** iOS ↔ Sim FFI bridge + Metal visualization + timeline scrubber
- **Sprint 3:** Level 1 fully playable with tutorial overlays
- **Sprint 4:** Food web depth + Level 2
- **Sprint 5:** Climate stability + Level 3 + Codex UI
- **Sprint 6:** StoreKit2 + paywall
- **Sprint 7:** Polish + TestFlight

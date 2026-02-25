# Sprint 2: iOS FFI Bridge & Visualization

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Connect the Rust sim-core to iOS via FFI, build a live heatmap visualization, and make Level 1 playable in the simulator.

**Architecture:** Rust sim-core is compiled as a static library (`.a`) for iOS. A thin C header exposes `extern "C"` functions using opaque pointers for the simulation handle, flat `f32` arrays for tile data (efficient for rendering), and JSON strings for complex types (species, interventions). Swift wraps the C API in a type-safe `SimulationEngine` class. An `@Observable` ViewModel drives SwiftUI views. A Canvas-based grid renderer draws 64x32 colored tiles for heatmap overlays.

**Tech Stack:** Rust FFI (`extern "C"`, `staticlib`), C header (manual), XcodeGen, Swift 6 / SwiftUI / `@Observable`, Canvas rendering

---

## Task 1: Rust FFI Module

**Files:**
- Create: `sim-core/src/ffi.rs`
- Modify: `sim-core/src/lib.rs` (add `pub mod ffi;`)
- Test: `sim-core/tests/ffi_test.rs`

**Step 1: Write the failing test**

Create `sim-core/tests/ffi_test.rs`:

```rust
use std::ffi::CString;
use std::ptr;

// We test via the Rust API directly since FFI functions
// are extern "C" — we can call them from Rust tests too.
use planet_architect_sim::ffi::*;

#[test]
fn test_create_and_destroy() {
    let handle = pa_sim_create(42, ptr::null());
    assert!(!handle.is_null());
    pa_sim_destroy(handle);
}

#[test]
fn test_create_with_params_json() {
    let json = CString::new(r#"{"gravity":9.8,"rotation_rate":1.0,"axial_tilt":23.4,"core_heat":0.4,"magnetic_field":0.6,"atmosphere":{"pressure":1.0,"o2":0.21,"co2":0.0004,"toxicity":0.0},"hydrology":{"ocean_coverage":0.7,"salinity":0.035,"current_strength":0.5,"ice_fraction":0.1}}"#).unwrap();
    let handle = pa_sim_create(42, json.as_ptr());
    assert!(!handle.is_null());
    pa_sim_destroy(handle);
}

#[test]
fn test_step_advances_time() {
    let handle = pa_sim_create(42, ptr::null());
    assert_eq!(pa_sim_current_step(handle), 0);
    pa_sim_step(handle, 100);
    assert_eq!(pa_sim_current_step(handle), 100);
    pa_sim_destroy(handle);
}

#[test]
fn test_snapshot_returns_valid_data() {
    let handle = pa_sim_create(42, ptr::null());
    pa_sim_step(handle, 10);
    pa_sim_snapshot_update(handle);

    assert_eq!(pa_sim_snapshot_width(handle), 64);
    assert_eq!(pa_sim_snapshot_height(handle), 32);

    let temps = pa_sim_snapshot_temperatures(handle);
    assert!(!temps.is_null());

    let nutrients = pa_sim_snapshot_nutrients(handle);
    assert!(!nutrients.is_null());

    let population = pa_sim_snapshot_population(handle);
    assert!(!population.is_null());

    let ocean = pa_sim_snapshot_ocean_mask(handle);
    assert!(!ocean.is_null());

    // Read a temperature value — should be in bounds
    let temp_val = unsafe { *temps };
    assert!(temp_val >= -80.0 && temp_val <= 80.0, "temp={}", temp_val);

    pa_sim_destroy(handle);
}

#[test]
fn test_add_species_and_check_biodiversity() {
    let handle = pa_sim_create(42, ptr::null());
    // Warm up nutrients
    pa_sim_step(handle, 500);

    let species_json = CString::new(r#"{"id":0,"name":"Algae","traits":{"temp_optimal":15.0,"temp_range":50.0,"o2_need":0.0,"toxin_resistance":0.1,"trophic_level":"Producer","reproduction_rate":0.05,"dispersal":0.3,"mutation_rate":0.01}}"#).unwrap();
    pa_sim_add_species_json(handle, species_json.as_ptr(), 100.0);

    pa_sim_step(handle, 100);
    pa_sim_snapshot_update(handle);

    let bio = pa_sim_snapshot_biodiversity(handle);
    assert!(bio >= 1, "Should have at least 1 species, got {}", bio);

    let species_ptr = pa_sim_snapshot_species_json(handle);
    assert!(!species_ptr.is_null());
    let species_str = unsafe { std::ffi::CStr::from_ptr(species_ptr) }.to_str().unwrap();
    assert!(species_str.contains("Algae"), "Species JSON should contain Algae: {}", species_str);

    pa_sim_destroy(handle);
}

#[test]
fn test_apply_intervention() {
    let handle = pa_sim_create(42, ptr::null());

    let intervention_json = CString::new(r#"{"kind":{"AdjustCO2":{"delta":0.01}},"target_region":null,"step":0}"#).unwrap();
    let result = pa_sim_apply_intervention_json(handle, intervention_json.as_ptr());
    assert_eq!(result, 0, "Intervention should succeed");

    pa_sim_destroy(handle);
}

#[test]
fn test_save_and_load() {
    let handle = pa_sim_create(42, ptr::null());
    pa_sim_step(handle, 500);

    let mut save_len: usize = 0;
    let save_ptr = pa_sim_save_state(handle, &mut save_len);
    assert!(!save_ptr.is_null());
    assert!(save_len > 0);

    let step_before = pa_sim_current_step(handle);
    pa_sim_destroy(handle);

    // Load
    let loaded = pa_sim_load_state(save_ptr, save_len);
    assert!(!loaded.is_null());
    assert_eq!(pa_sim_current_step(loaded), step_before);

    // Free saved bytes and loaded handle
    pa_free_bytes(save_ptr, save_len);
    pa_sim_destroy(loaded);
}

#[test]
fn test_null_handle_safety() {
    // All functions should handle null gracefully
    pa_sim_step(ptr::null_mut(), 100);
    pa_sim_snapshot_update(ptr::null_mut());
    assert_eq!(pa_sim_current_step(ptr::null_mut()), 0);
    assert_eq!(pa_sim_snapshot_width(ptr::null_mut()), 0);
    assert!(pa_sim_snapshot_temperatures(ptr::null_mut()).is_null());
    pa_sim_destroy(ptr::null_mut());
}
```

**Step 2: Run test to verify it fails**

Run: `cd /Users/jonathanguttman/Documents/World-Builder/sim-core && cargo test --test ffi_test`
Expected: FAIL — module `ffi` not found.

**Step 3: Implement ffi.rs**

```rust
use std::ffi::{c_char, CStr, CString};
use std::ptr;

use crate::biosphere;
use crate::sim::Simulation;
use crate::types::*;

/// Opaque handle wrapping a Simulation with cached flat arrays for FFI
pub struct SimHandle {
    sim: Simulation,
    temp_cache: Vec<f32>,
    nutrient_cache: Vec<f32>,
    moisture_cache: Vec<f32>,
    population_cache: Vec<f32>,
    ocean_cache: Vec<u8>,
    species_json_cache: CString,
}

impl SimHandle {
    fn new(sim: Simulation) -> Self {
        let tile_count = sim.grid().width * sim.grid().height;
        let mut handle = Self {
            sim,
            temp_cache: vec![0.0; tile_count],
            nutrient_cache: vec![0.0; tile_count],
            moisture_cache: vec![0.0; tile_count],
            population_cache: vec![0.0; tile_count],
            ocean_cache: vec![0; tile_count],
            species_json_cache: CString::new("[]").unwrap(),
        };
        handle.update_cache();
        handle
    }

    fn update_cache(&mut self) {
        let grid = self.sim.grid();
        for (i, tile) in grid.tiles.iter().enumerate() {
            self.temp_cache[i] = tile.temperature;
            self.nutrient_cache[i] = tile.nutrients;
            self.moisture_cache[i] = tile.moisture;
            self.population_cache[i] = tile.populations.values().sum::<f64>() as f32;
            self.ocean_cache[i] = if tile.is_ocean { 1 } else { 0 };
        }

        let species_data: Vec<serde_json::Value> = self.sim.species().iter().map(|s| {
            serde_json::json!({
                "id": s.id,
                "name": &s.name,
                "trophicLevel": format!("{:?}", s.traits.trophic_level),
                "globalPopulation": biosphere::global_population(self.sim.grid(), s.id),
            })
        }).collect();
        let json = serde_json::to_string(&species_data).unwrap_or_else(|_| "[]".to_string());
        self.species_json_cache = CString::new(json).unwrap_or_else(|_| CString::new("[]").unwrap());
    }
}

// --- Lifecycle ---

#[no_mangle]
pub extern "C" fn pa_sim_create(seed: u64, params_json: *const c_char) -> *mut SimHandle {
    let params = if params_json.is_null() {
        PlanetParams::default()
    } else {
        let c_str = unsafe { CStr::from_ptr(params_json) };
        match c_str.to_str() {
            Ok(s) => serde_json::from_str(s).unwrap_or_default(),
            Err(_) => PlanetParams::default(),
        }
    };

    let sim = Simulation::new(seed, params);
    Box::into_raw(Box::new(SimHandle::new(sim)))
}

#[no_mangle]
pub extern "C" fn pa_sim_destroy(handle: *mut SimHandle) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle)); }
    }
}

// --- Simulation ---

#[no_mangle]
pub extern "C" fn pa_sim_step(handle: *mut SimHandle, steps: u64) {
    if handle.is_null() { return; }
    let h = unsafe { &mut *handle };
    h.sim.step(steps);
}

#[no_mangle]
pub extern "C" fn pa_sim_current_step(handle: *mut SimHandle) -> u64 {
    if handle.is_null() { return 0; }
    let h = unsafe { &*handle };
    h.sim.current_step()
}

// --- Snapshot ---

#[no_mangle]
pub extern "C" fn pa_sim_snapshot_update(handle: *mut SimHandle) {
    if handle.is_null() { return; }
    let h = unsafe { &mut *handle };
    h.update_cache();
}

#[no_mangle]
pub extern "C" fn pa_sim_snapshot_width(handle: *mut SimHandle) -> u32 {
    if handle.is_null() { return 0; }
    let h = unsafe { &*handle };
    h.sim.grid().width as u32
}

#[no_mangle]
pub extern "C" fn pa_sim_snapshot_height(handle: *mut SimHandle) -> u32 {
    if handle.is_null() { return 0; }
    let h = unsafe { &*handle };
    h.sim.grid().height as u32
}

#[no_mangle]
pub extern "C" fn pa_sim_snapshot_temperatures(handle: *mut SimHandle) -> *const f32 {
    if handle.is_null() { return ptr::null(); }
    let h = unsafe { &*handle };
    h.temp_cache.as_ptr()
}

#[no_mangle]
pub extern "C" fn pa_sim_snapshot_nutrients(handle: *mut SimHandle) -> *const f32 {
    if handle.is_null() { return ptr::null(); }
    let h = unsafe { &*handle };
    h.nutrient_cache.as_ptr()
}

#[no_mangle]
pub extern "C" fn pa_sim_snapshot_moisture(handle: *mut SimHandle) -> *const f32 {
    if handle.is_null() { return ptr::null(); }
    let h = unsafe { &*handle };
    h.moisture_cache.as_ptr()
}

#[no_mangle]
pub extern "C" fn pa_sim_snapshot_population(handle: *mut SimHandle) -> *const f32 {
    if handle.is_null() { return ptr::null(); }
    let h = unsafe { &*handle };
    h.population_cache.as_ptr()
}

#[no_mangle]
pub extern "C" fn pa_sim_snapshot_ocean_mask(handle: *mut SimHandle) -> *const u8 {
    if handle.is_null() { return ptr::null(); }
    let h = unsafe { &*handle };
    h.ocean_cache.as_ptr()
}

#[no_mangle]
pub extern "C" fn pa_sim_snapshot_biodiversity(handle: *mut SimHandle) -> u32 {
    if handle.is_null() { return 0; }
    let h = unsafe { &*handle };
    biosphere::biodiversity_count(h.sim.grid(), h.sim.species())
}

#[no_mangle]
pub extern "C" fn pa_sim_snapshot_species_json(handle: *mut SimHandle) -> *const c_char {
    if handle.is_null() { return ptr::null(); }
    let h = unsafe { &*handle };
    h.species_json_cache.as_ptr()
}

// --- Species ---

#[no_mangle]
pub extern "C" fn pa_sim_add_species_json(handle: *mut SimHandle, json: *const c_char, initial_pop: f64) {
    if handle.is_null() || json.is_null() { return; }
    let h = unsafe { &mut *handle };
    let c_str = unsafe { CStr::from_ptr(json) };
    if let Ok(json_str) = c_str.to_str() {
        if let Ok(species) = serde_json::from_str::<Species>(json_str) {
            h.sim.add_species(species, initial_pop);
        }
    }
}

// --- Interventions ---

#[no_mangle]
pub extern "C" fn pa_sim_apply_intervention_json(handle: *mut SimHandle, json: *const c_char) -> i32 {
    if handle.is_null() || json.is_null() { return -1; }
    let h = unsafe { &mut *handle };
    let c_str = unsafe { CStr::from_ptr(json) };
    if let Ok(json_str) = c_str.to_str() {
        if let Ok(intervention) = serde_json::from_str::<Intervention>(json_str) {
            match h.sim.apply_intervention(intervention) {
                Ok(()) => return 0,
                Err(_) => return -1,
            }
        }
    }
    -1
}

// --- Save/Load ---

#[no_mangle]
pub extern "C" fn pa_sim_save_state(handle: *mut SimHandle, out_len: *mut usize) -> *const u8 {
    if handle.is_null() || out_len.is_null() { return ptr::null(); }
    let h = unsafe { &*handle };
    match h.sim.save_state() {
        Ok(bytes) => {
            let len = bytes.len();
            let ptr = bytes.as_ptr();
            unsafe { *out_len = len; }
            std::mem::forget(bytes);
            ptr
        }
        Err(_) => {
            unsafe { *out_len = 0; }
            ptr::null()
        }
    }
}

#[no_mangle]
pub extern "C" fn pa_sim_load_state(data: *const u8, len: usize) -> *mut SimHandle {
    if data.is_null() || len == 0 { return ptr::null_mut(); }
    let bytes = unsafe { std::slice::from_raw_parts(data, len) };
    match Simulation::load_state(bytes) {
        Ok(sim) => Box::into_raw(Box::new(SimHandle::new(sim))),
        Err(_) => ptr::null_mut(),
    }
}

// --- Memory Management ---

#[no_mangle]
pub extern "C" fn pa_free_bytes(data: *const u8, len: usize) {
    if !data.is_null() && len > 0 {
        unsafe {
            let _ = Vec::from_raw_parts(data as *mut u8, len, len);
        }
    }
}
```

**Step 4: Add module to lib.rs**

Add `pub mod ffi;` to `sim-core/src/lib.rs`.

**Step 5: Run tests**

Run: `cd /Users/jonathanguttman/Documents/World-Builder/sim-core && cargo test --test ffi_test`
Expected: All 8 tests PASS.

**Step 6: Run full test suite**

Run: `cd /Users/jonathanguttman/Documents/World-Builder/sim-core && cargo test`
Expected: All tests PASS (39 existing + 8 new = 47).

**Step 7: Commit**

```bash
git add sim-core/
git commit -m "feat: FFI module with C-compatible API for iOS bridge"
```

---

## Task 2: C Header & iOS Build Script

**Files:**
- Create: `sim-core/include/planet_architect.h`
- Create: `sim-core/build-ios.sh`

**Step 1: Create the C header**

Create `sim-core/include/planet_architect.h` (manual, not cbindgen — our API surface is small and hand-written headers are more predictable):

```c
#ifndef PLANET_ARCHITECT_H
#define PLANET_ARCHITECT_H

#include <stdint.h>
#include <stddef.h>

// Opaque handle to the simulation
typedef void* PASimHandle;

// --- Lifecycle ---
PASimHandle pa_sim_create(uint64_t seed, const char* params_json);
void pa_sim_destroy(PASimHandle handle);

// --- Simulation ---
void pa_sim_step(PASimHandle handle, uint64_t steps);
uint64_t pa_sim_current_step(PASimHandle handle);

// --- Snapshot ---
// Call pa_sim_snapshot_update first, then read cached arrays.
// Pointers are valid until the next pa_sim_snapshot_update or pa_sim_step call.
void pa_sim_snapshot_update(PASimHandle handle);
uint32_t pa_sim_snapshot_width(PASimHandle handle);
uint32_t pa_sim_snapshot_height(PASimHandle handle);
const float* pa_sim_snapshot_temperatures(PASimHandle handle);
const float* pa_sim_snapshot_nutrients(PASimHandle handle);
const float* pa_sim_snapshot_moisture(PASimHandle handle);
const float* pa_sim_snapshot_population(PASimHandle handle);
const uint8_t* pa_sim_snapshot_ocean_mask(PASimHandle handle);
uint32_t pa_sim_snapshot_biodiversity(PASimHandle handle);
const char* pa_sim_snapshot_species_json(PASimHandle handle);

// --- Species ---
void pa_sim_add_species_json(PASimHandle handle, const char* json, double initial_pop);

// --- Interventions ---
// Returns 0 on success, -1 on failure.
int32_t pa_sim_apply_intervention_json(PASimHandle handle, const char* json);

// --- Save/Load ---
// pa_sim_save_state returns a byte buffer. Caller must free with pa_free_bytes.
const uint8_t* pa_sim_save_state(PASimHandle handle, size_t* out_len);
PASimHandle pa_sim_load_state(const uint8_t* data, size_t len);

// --- Memory Management ---
void pa_free_bytes(const uint8_t* data, size_t len);

#endif
```

**Step 2: Create the iOS build script**

Create `sim-core/build-ios.sh`:

```bash
#!/bin/bash
set -e

cd "$(dirname "$0")"

echo "=== Building Planet Architect sim-core for iOS ==="

echo ""
echo "[1/2] Building for iOS Simulator (arm64)..."
cargo build --release --target aarch64-apple-ios-sim

echo ""
echo "[2/2] Building for iOS Device (arm64)..."
cargo build --release --target aarch64-apple-ios

echo ""
echo "=== Build complete ==="
echo "  Simulator: target/aarch64-apple-ios-sim/release/libplanet_architect_sim.a"
echo "  Device:    target/aarch64-apple-ios/release/libplanet_architect_sim.a"
```

**Step 3: Make build script executable and run it**

```bash
chmod +x sim-core/build-ios.sh
cd /Users/jonathanguttman/Documents/World-Builder/sim-core && ./build-ios.sh
```

Expected: Both `.a` files produced. The build may take 1-3 minutes.

**Step 4: Verify the libraries exist**

```bash
ls -la sim-core/target/aarch64-apple-ios-sim/release/libplanet_architect_sim.a
ls -la sim-core/target/aarch64-apple-ios/release/libplanet_architect_sim.a
```

**Step 5: Commit**

```bash
git add sim-core/include/ sim-core/build-ios.sh
git commit -m "feat: C header and iOS cross-compilation build script"
```

---

## Task 3: XcodeGen Project Setup

**Files:**
- Create: `ios/project.yml`
- Create: `ios/PlanetArchitect/Core/SimulationBridge/PlanetArchitect-Bridging-Header.h`
- Generate: `ios/PlanetArchitect.xcodeproj`

**Step 1: Create the bridging header**

Create `ios/PlanetArchitect/Core/SimulationBridge/PlanetArchitect-Bridging-Header.h`:

```c
#ifndef PlanetArchitect_Bridging_Header_h
#define PlanetArchitect_Bridging_Header_h

#include "planet_architect.h"

#endif
```

**Step 2: Create XcodeGen project spec**

Create `ios/project.yml`:

```yaml
name: PlanetArchitect
options:
  bundleIdPrefix: com.planetarchitect
  deploymentTarget:
    iOS: "17.0"
  xcodeVersion: "16.0"

settings:
  base:
    SWIFT_VERSION: "6.0"

targets:
  PlanetArchitect:
    type: application
    platform: iOS
    sources:
      - path: PlanetArchitect
        excludes:
          - "**/.gitkeep"
    settings:
      base:
        PRODUCT_BUNDLE_IDENTIFIER: com.planetarchitect.app
        SWIFT_OBJC_BRIDGING_HEADER: PlanetArchitect/Core/SimulationBridge/PlanetArchitect-Bridging-Header.h
        HEADER_SEARCH_PATHS:
          - "$(PROJECT_DIR)/../sim-core/include"
        LIBRARY_SEARCH_PATHS:
          - "$(PROJECT_DIR)/../sim-core/target/aarch64-apple-ios-sim/release"
          - "$(PROJECT_DIR)/../sim-core/target/aarch64-apple-ios/release"
        OTHER_LDFLAGS:
          - "-lplanet_architect_sim"
        INFOPLIST_GENERATION_MODE: GeneratedFile
        MARKETING_VERSION: "0.1.0"
        CURRENT_PROJECT_VERSION: "1"
        SWIFT_STRICT_CONCURRENCY: complete
```

**Step 3: Generate the Xcode project**

```bash
cd /Users/jonathanguttman/Documents/World-Builder/ios && xcodegen generate
```

Expected: `PlanetArchitect.xcodeproj` created.

**Step 4: Add .xcodeproj to .gitignore (generated, not committed)**

Append to the root `.gitignore`:
```
# Xcode generated project (use xcodegen to regenerate)
*.xcodeproj/
```

**Step 5: Commit**

```bash
git add ios/project.yml ios/PlanetArchitect/Core/SimulationBridge/PlanetArchitect-Bridging-Header.h .gitignore
git commit -m "feat: XcodeGen project spec with Rust static library linking"
```

---

## Task 4: Swift SimulationEngine Wrapper

**Files:**
- Create: `ios/PlanetArchitect/Core/SimulationBridge/SimulationEngine.swift`

**Step 1: Implement the Swift wrapper**

```swift
import Foundation

/// Type-safe Swift wrapper around the Rust sim-core FFI.
/// Manages the opaque simulation handle lifecycle and provides
/// Swift-native access to simulation data.
final class SimulationEngine: @unchecked Sendable {
    private let handle: PASimHandle

    let gridWidth: Int
    let gridHeight: Int
    private let tileCount: Int

    init(seed: UInt64, paramsJSON: String? = nil) {
        if let json = paramsJSON {
            handle = json.withCString { pa_sim_create(seed, $0) }
        } else {
            handle = pa_sim_create(seed, nil)
        }
        gridWidth = Int(pa_sim_snapshot_width(handle))
        gridHeight = Int(pa_sim_snapshot_height(handle))
        tileCount = gridWidth * gridHeight
    }

    private init(handle: PASimHandle, width: Int, height: Int) {
        self.handle = handle
        self.gridWidth = width
        self.gridHeight = height
        self.tileCount = width * height
    }

    deinit {
        pa_sim_destroy(handle)
    }

    // MARK: - Simulation Control

    func step(_ steps: UInt64) {
        pa_sim_step(handle, steps)
    }

    var currentStep: UInt64 {
        pa_sim_current_step(handle)
    }

    // MARK: - Snapshot

    func updateSnapshot() {
        pa_sim_snapshot_update(handle)
    }

    var temperatures: [Float] {
        guard let ptr = pa_sim_snapshot_temperatures(handle) else { return [] }
        return Array(UnsafeBufferPointer(start: ptr, count: tileCount))
    }

    var nutrients: [Float] {
        guard let ptr = pa_sim_snapshot_nutrients(handle) else { return [] }
        return Array(UnsafeBufferPointer(start: ptr, count: tileCount))
    }

    var moisture: [Float] {
        guard let ptr = pa_sim_snapshot_moisture(handle) else { return [] }
        return Array(UnsafeBufferPointer(start: ptr, count: tileCount))
    }

    var populationDensity: [Float] {
        guard let ptr = pa_sim_snapshot_population(handle) else { return [] }
        return Array(UnsafeBufferPointer(start: ptr, count: tileCount))
    }

    var oceanMask: [UInt8] {
        guard let ptr = pa_sim_snapshot_ocean_mask(handle) else { return [] }
        return Array(UnsafeBufferPointer(start: ptr, count: tileCount))
    }

    var biodiversityCount: UInt32 {
        pa_sim_snapshot_biodiversity(handle)
    }

    var speciesJSON: String {
        guard let ptr = pa_sim_snapshot_species_json(handle) else { return "[]" }
        return String(cString: ptr)
    }

    // MARK: - Species

    func addSpecies(json: String, initialPopulation: Double) {
        json.withCString { pa_sim_add_species_json(handle, $0, initialPopulation) }
    }

    // MARK: - Interventions

    func applyIntervention(json: String) -> Bool {
        json.withCString { pa_sim_apply_intervention_json(handle, $0) } == 0
    }

    // MARK: - Save/Load

    func saveState() -> Data? {
        var len: Int = 0
        guard let ptr = pa_sim_save_state(handle, &len), len > 0 else { return nil }
        let data = Data(bytes: ptr, count: len)
        pa_free_bytes(ptr, len)
        return data
    }

    static func loadState(from data: Data) -> SimulationEngine? {
        data.withUnsafeBytes { buffer -> SimulationEngine? in
            guard let ptr = buffer.baseAddress?.assumingMemoryBound(to: UInt8.self) else { return nil }
            guard let h = pa_sim_load_state(ptr, buffer.count) else { return nil }
            let width = Int(pa_sim_snapshot_width(h))
            let height = Int(pa_sim_snapshot_height(h))
            return SimulationEngine(handle: h, width: width, height: height)
        }
    }
}
```

**Step 2: Commit**

```bash
git add ios/PlanetArchitect/Core/SimulationBridge/SimulationEngine.swift
git commit -m "feat: Swift SimulationEngine wrapper for Rust FFI"
```

---

## Task 5: SimulationViewModel

**Files:**
- Create: `ios/PlanetArchitect/Features/Planet/SimulationViewModel.swift`

**Step 1: Implement the ViewModel**

```swift
import SwiftUI

enum OverlayMode: String, CaseIterable, Identifiable {
    case temperature = "Temp"
    case nutrients = "Nutrients"
    case moisture = "Moisture"
    case population = "Population"

    var id: String { rawValue }
}

@MainActor
@Observable
final class SimulationViewModel {
    // Engine
    private(set) var engine: SimulationEngine?

    // State
    private(set) var currentStep: UInt64 = 0
    private(set) var biodiversity: UInt32 = 0
    var isPaused: Bool = true
    var timeSpeed: TimeSpeed = .adapt
    var overlayMode: OverlayMode = .temperature

    // Grid data (updated from snapshot)
    private(set) var temperatures: [Float] = []
    private(set) var nutrients: [Float] = []
    private(set) var moisture: [Float] = []
    private(set) var populationDensity: [Float] = []
    private(set) var oceanMask: [UInt8] = []

    var gridWidth: Int { engine?.gridWidth ?? 64 }
    var gridHeight: Int { engine?.gridHeight ?? 32 }

    // Simulation loop
    private var simulationTask: Task<Void, Never>?

    var currentOverlayData: [Float] {
        switch overlayMode {
        case .temperature: return temperatures
        case .nutrients: return nutrients
        case .moisture: return moisture
        case .population: return populationDensity
        }
    }

    // MARK: - Lifecycle

    func startLevel(seed: UInt64, paramsJSON: String? = nil) {
        engine = SimulationEngine(seed: seed, paramsJSON: paramsJSON)

        // Warm up nutrients
        engine?.step(500)

        // Seed initial microbe for Level 1
        let microbeJSON = """
        {"id":0,"name":"Thermophile","traits":{"temp_optimal":5.0,"temp_range":60.0,"o2_need":0.0,"toxin_resistance":0.3,"trophic_level":"Producer","reproduction_rate":0.04,"dispersal":0.3,"mutation_rate":0.005}}
        """
        engine?.addSpecies(json: microbeJSON, initialPopulation: 100.0)
        refreshSnapshot()
    }

    // MARK: - Simulation Control

    func togglePause() {
        isPaused.toggle()
        if isPaused {
            simulationTask?.cancel()
            simulationTask = nil
        } else {
            startSimulationLoop()
        }
    }

    private func startSimulationLoop() {
        simulationTask = Task { [weak self] in
            while !Task.isCancelled {
                guard let self else { break }
                self.tick()
                try? await Task.sleep(for: .milliseconds(33))
            }
        }
    }

    private func tick() {
        guard let engine else { return }
        engine.step(timeSpeed.stepsPerBatch)
        refreshSnapshot()
    }

    func refreshSnapshot() {
        guard let engine else { return }
        engine.updateSnapshot()
        currentStep = engine.currentStep
        biodiversity = engine.biodiversityCount
        temperatures = engine.temperatures
        nutrients = engine.nutrients
        moisture = engine.moisture
        populationDensity = engine.populationDensity
        oceanMask = engine.oceanMask
    }

    // MARK: - Tile Inspector

    func tileInfo(x: Int, y: Int) -> TileInfo? {
        let index = y * gridWidth + x
        guard index < temperatures.count else { return nil }
        return TileInfo(
            x: x, y: y,
            temperature: temperatures[index],
            nutrients: nutrients[index],
            moisture: moisture[index],
            population: populationDensity[index],
            isOcean: oceanMask[index] == 1
        )
    }
}

struct TileInfo {
    let x: Int
    let y: Int
    let temperature: Float
    let nutrients: Float
    let moisture: Float
    let population: Float
    let isOcean: Bool
}
```

**Step 2: Commit**

```bash
git add ios/PlanetArchitect/Features/Planet/SimulationViewModel.swift
git commit -m "feat: SimulationViewModel with Observable state and simulation loop"
```

---

## Task 6: Grid Renderer

**Files:**
- Create: `ios/PlanetArchitect/UI/Components/GridRenderer.swift`

**Step 1: Implement Canvas-based heatmap**

```swift
import SwiftUI

struct GridRenderer: View {
    let data: [Float]
    let oceanMask: [UInt8]
    let width: Int
    let height: Int
    let overlayMode: OverlayMode
    var onTileTap: ((Int, Int) -> Void)?

    var body: some View {
        GeometryReader { geo in
            Canvas { context, size in
                guard !data.isEmpty, width > 0, height > 0 else { return }

                let tileW = size.width / CGFloat(width)
                let tileH = size.height / CGFloat(height)

                for y in 0..<height {
                    for x in 0..<width {
                        let index = y * width + x
                        guard index < data.count else { continue }

                        let isOcean = index < oceanMask.count && oceanMask[index] == 1
                        let color = tileColor(value: data[index], isOcean: isOcean)
                        let rect = CGRect(
                            x: CGFloat(x) * tileW,
                            y: CGFloat(y) * tileH,
                            width: tileW + 0.5,
                            height: tileH + 0.5
                        )
                        context.fill(Path(rect), with: .color(color))
                    }
                }
            }
            .contentShape(Rectangle())
            .onTapGesture { location in
                let tileW = geo.size.width / CGFloat(width)
                let tileH = geo.size.height / CGFloat(height)
                let x = Int(location.x / tileW)
                let y = Int(location.y / tileH)
                if x >= 0 && x < width && y >= 0 && y < height {
                    onTileTap?(x, y)
                }
            }
        }
    }

    private func tileColor(value: Float, isOcean: Bool) -> Color {
        switch overlayMode {
        case .temperature:
            return temperatureColor(value)
        case .nutrients:
            return scalarColor(value, low: Color(red: 0.05, green: 0.05, blue: 0.05),
                             high: Color(red: 0.2, green: 0.9, blue: 0.1))
        case .moisture:
            return scalarColor(value, low: Color(red: 0.3, green: 0.2, blue: 0.1),
                             high: Color(red: 0.1, green: 0.3, blue: 0.9))
        case .population:
            if value <= 0 {
                return isOcean ? Color(white: 0.12) : Color(white: 0.2)
            }
            let intensity = min(Double(value) / 5000.0, 1.0)
            return isOcean
                ? Color(red: 0, green: intensity * 0.8, blue: intensity)
                : Color(red: intensity * 0.2, green: intensity, blue: 0)
        }
    }

    private func temperatureColor(_ temp: Float) -> Color {
        let t = Double((temp + 80.0) / 160.0).clamped(to: 0...1)
        if t < 0.25 {
            let f = t / 0.25
            return Color(red: 0, green: 0, blue: 0.3 + f * 0.7)
        } else if t < 0.5 {
            let f = (t - 0.25) / 0.25
            return Color(red: 0, green: f, blue: 1.0 - f * 0.5)
        } else if t < 0.75 {
            let f = (t - 0.5) / 0.25
            return Color(red: f, green: 1.0 - f * 0.3, blue: 0)
        } else {
            let f = (t - 0.75) / 0.25
            return Color(red: 1.0, green: 0.7 - f * 0.7, blue: 0)
        }
    }

    private func scalarColor(_ val: Float, low: Color, high: Color) -> Color {
        let t = Double(val.clamped(to: 0...1))
        return blend(from: low, to: high, fraction: t)
    }

    private func blend(from: Color, to: Color, fraction: Double) -> Color {
        // Simple lerp between two colors
        let f = fraction.clamped(to: 0...1)
        let r1 = from.components.red, g1 = from.components.green, b1 = from.components.blue
        let r2 = to.components.red, g2 = to.components.green, b2 = to.components.blue
        return Color(
            red: r1 + (r2 - r1) * f,
            green: g1 + (g2 - g1) * f,
            blue: b1 + (b2 - b1) * f
        )
    }
}

// MARK: - Helpers

private extension Comparable {
    func clamped(to range: ClosedRange<Self>) -> Self {
        min(max(self, range.lowerBound), range.upperBound)
    }
}

private extension Color {
    struct Components {
        let red: Double, green: Double, blue: Double
    }
    var components: Components {
        var r: CGFloat = 0, g: CGFloat = 0, b: CGFloat = 0, a: CGFloat = 0
        UIColor(self).getRed(&r, green: &g, blue: &b, alpha: &a)
        return Components(red: Double(r), green: Double(g), blue: Double(b))
    }
}
```

**Step 2: Commit**

```bash
git add ios/PlanetArchitect/UI/Components/GridRenderer.swift
git commit -m "feat: Canvas-based grid renderer with temperature, nutrient, moisture, and population overlays"
```

---

## Task 7: Updated PlanetView + Tile Inspector

**Files:**
- Modify: `ios/PlanetArchitect/Features/Planet/PlanetView.swift`

**Step 1: Replace the placeholder PlanetView**

```swift
import SwiftUI

struct PlanetView: View {
    let levelId: Int
    let seed: UInt64
    let paramsJSON: String?

    @State private var viewModel = SimulationViewModel()
    @State private var selectedTile: TileInfo?
    @State private var showTileInspector = false

    init(levelId: Int, seed: UInt64 = 7749, paramsJSON: String? = nil) {
        self.levelId = levelId
        self.seed = seed
        self.paramsJSON = paramsJSON
    }

    var body: some View {
        VStack(spacing: 0) {
            // Status bar
            HStack {
                Label("\(viewModel.currentStep) yr", systemImage: "clock")
                Spacer()
                Label("\(viewModel.biodiversity) species", systemImage: "leaf")
            }
            .font(.caption)
            .padding(.horizontal)
            .padding(.vertical, 4)

            // Grid
            GridRenderer(
                data: viewModel.currentOverlayData,
                oceanMask: viewModel.oceanMask,
                width: viewModel.gridWidth,
                height: viewModel.gridHeight,
                overlayMode: viewModel.overlayMode,
                onTileTap: { x, y in
                    selectedTile = viewModel.tileInfo(x: x, y: y)
                    showTileInspector = true
                }
            )
            .aspectRatio(2.0, contentMode: .fit)
            .clipShape(RoundedRectangle(cornerRadius: 8))
            .padding(.horizontal)

            Spacer()

            // Overlay picker
            Picker("Overlay", selection: $viewModel.overlayMode) {
                ForEach(OverlayMode.allCases) { mode in
                    Text(mode.rawValue).tag(mode)
                }
            }
            .pickerStyle(.segmented)
            .padding(.horizontal)

            // Time controls
            HStack(spacing: 16) {
                Button {
                    viewModel.togglePause()
                } label: {
                    Image(systemName: viewModel.isPaused ? "play.fill" : "pause.fill")
                        .font(.title2)
                        .frame(width: 44, height: 44)
                }

                Picker("Speed", selection: $viewModel.timeSpeed) {
                    Text("1x").tag(TimeSpeed.observe)
                    Text("100x").tag(TimeSpeed.adapt)
                    Text("10K").tag(TimeSpeed.epoch)
                    Text("1M").tag(TimeSpeed.eon)
                }
                .pickerStyle(.segmented)
            }
            .padding()
        }
        .navigationTitle("Level \(levelId)")
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            viewModel.startLevel(seed: seed, paramsJSON: paramsJSON)
        }
        .sheet(isPresented: $showTileInspector) {
            if let tile = selectedTile {
                TileInspectorView(tile: tile)
                    .presentationDetents([.height(200)])
            }
        }
    }
}

struct TileInspectorView: View {
    let tile: TileInfo

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Tile (\(tile.x), \(tile.y))")
                .font(.headline)

            HStack {
                Text(tile.isOcean ? "Ocean" : "Land")
                    .foregroundStyle(tile.isOcean ? .blue : .brown)
                Spacer()
            }

            Grid(alignment: .leading) {
                GridRow {
                    Text("Temperature")
                    Text(String(format: "%.1f C", tile.temperature))
                        .foregroundStyle(.secondary)
                }
                GridRow {
                    Text("Nutrients")
                    Text(String(format: "%.3f", tile.nutrients))
                        .foregroundStyle(.secondary)
                }
                GridRow {
                    Text("Moisture")
                    Text(String(format: "%.3f", tile.moisture))
                        .foregroundStyle(.secondary)
                }
                GridRow {
                    Text("Population")
                    Text(String(format: "%.0f", tile.population))
                        .foregroundStyle(.secondary)
                }
            }
            .font(.body)
        }
        .padding()
    }
}
```

**Step 2: Update LevelSelectView to pass seed**

Modify `LevelSelectView.swift` — change the NavigationLink to pass seed:

```swift
NavigationLink {
    PlanetView(levelId: level.id, seed: 7749)
} label: {
    LevelRow(number: level.id, name: level.name, locked: false)
}
```

**Step 3: Commit**

```bash
git add ios/PlanetArchitect/Features/Planet/PlanetView.swift ios/PlanetArchitect/Features/Campaign/LevelSelectView.swift
git commit -m "feat: live PlanetView with grid renderer, overlay picker, time controls, and tile inspector"
```

---

## Task 8: Build & Run Verification

**Files:**
- None new — this task verifies the build chain works

**Step 1: Rebuild Rust library for iOS simulator**

```bash
cd /Users/jonathanguttman/Documents/World-Builder/sim-core && ./build-ios.sh
```

**Step 2: Regenerate Xcode project**

```bash
cd /Users/jonathanguttman/Documents/World-Builder/ios && xcodegen generate
```

**Step 3: Build the app for simulator**

```bash
cd /Users/jonathanguttman/Documents/World-Builder/ios && xcodebuild build \
    -project PlanetArchitect.xcodeproj \
    -scheme PlanetArchitect \
    -destination 'platform=iOS Simulator,name=iPhone 16' \
    -configuration Debug \
    CODE_SIGNING_ALLOWED=NO \
    2>&1 | tail -20
```

Expected: BUILD SUCCEEDED.

If there are compile errors, fix them. Common issues:
- Bridging header path wrong → fix in project.yml
- Library not found → verify build-ios.sh output path matches LIBRARY_SEARCH_PATHS
- Swift concurrency errors → add `@MainActor` or `nonisolated` as needed
- Missing `Sendable` conformance → add `@unchecked Sendable` where appropriate

**Step 4: Run full Rust test suite to confirm nothing broke**

```bash
cd /Users/jonathanguttman/Documents/World-Builder/sim-core && cargo test
```

**Step 5: Commit any build fixes**

```bash
git add -A
git commit -m "fix: resolve build issues for iOS simulator target"
```

---

## Task 9: Final — CHANGELOG, Push

**Files:**
- Modify: `docs/CHANGELOG.md`

**Step 1: Update CHANGELOG**

```markdown
# Changelog

## 0.3.0 - 2026-02-24
### Sprint 2: iOS FFI Bridge & Visualization
- Rust FFI module with C-compatible API (18 extern functions)
- C header (`planet_architect.h`) for iOS bridging
- iOS cross-compilation build script (simulator + device)
- XcodeGen project spec with static library linking
- Swift `SimulationEngine` wrapper with type-safe FFI access
- `SimulationViewModel` with `@Observable` state management and simulation loop
- Canvas-based `GridRenderer` with temperature, nutrient, moisture, and population overlays
- Live `PlanetView` with overlay picker, time controls, and step/biodiversity display
- Tile inspector (tap to see tile details)

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
```

**Step 2: Commit and push**

```bash
git add docs/CHANGELOG.md
git commit -m "docs: changelog for v0.3.0 — iOS FFI bridge and visualization"
git push origin main
```

---

## Summary: What This Plan Builds

| Component | Description |
|-----------|-------------|
| `sim-core/src/ffi.rs` | 18 `extern "C"` functions with SimHandle wrapper and cached flat arrays |
| `sim-core/include/planet_architect.h` | C header for iOS bridging |
| `sim-core/build-ios.sh` | Cross-compilation for iOS simulator + device |
| `ios/project.yml` | XcodeGen spec linking Rust static library |
| `SimulationEngine.swift` | Type-safe Swift wrapper around FFI |
| `SimulationViewModel.swift` | `@Observable` ViewModel driving the UI |
| `GridRenderer.swift` | Canvas heatmap with 4 overlay modes |
| `PlanetView.swift` | Full planet view with controls and tile inspector |

**End state:** The app builds for iOS Simulator and shows a live 64x32 heatmap of Level 1 with play/pause, speed control, overlay switching, and tile inspection.

## What Comes Next (Sprint 3+)
- Level 1 tutorial overlays and objective tracking UI
- Level completion screen with codex unlocks
- Intervention tray (player applies CO2/O2/nutrient changes)
- Level 2 and 3 implementation

// FFI functions are extern "C" and use raw pointers by design.
// Each function performs null checks before dereferencing.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

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
    objective_result_cache: CString,
    codex_entries_cache: CString,
    codex_unlocked_cache: CString,
    codex_new_cache: CString,
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
            objective_result_cache: CString::new("{}").unwrap(),
            codex_entries_cache: CString::new("[]").unwrap(),
            codex_unlocked_cache: CString::new("[]").unwrap(),
            codex_new_cache: CString::new("[]").unwrap(),
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

// --- Objective Evaluation ---

#[no_mangle]
pub extern "C" fn pa_sim_snapshot_total_biomass(handle: *mut SimHandle) -> f64 {
    if handle.is_null() { return 0.0; }
    let h = unsafe { &*handle };
    h.sim.grid().tiles.iter()
        .flat_map(|t| t.populations.values())
        .sum()
}

#[no_mangle]
pub extern "C" fn pa_sim_evaluate_objective(
    handle: *mut SimHandle,
    objective_json: *const c_char,
) -> *const c_char {
    if handle.is_null() || objective_json.is_null() { return ptr::null(); }
    let h = unsafe { &mut *handle };

    let c_str = unsafe { CStr::from_ptr(objective_json) };
    let json_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null(),
    };

    let objective: crate::level::Objective = match serde_json::from_str(json_str) {
        Ok(o) => o,
        Err(_) => return ptr::null(),
    };

    let total_biomass: f64 = h.sim.grid().tiles.iter()
        .flat_map(|t| t.populations.values())
        .sum();
    let biodiversity = crate::biosphere::biodiversity_count(h.sim.grid(), h.sim.species());
    let trophic_levels = crate::biosphere::trophic_level_count(h.sim.grid(), h.sim.species());

    let condition_met = match &objective {
        crate::level::Objective::MicrobialStability { min_biomass, .. } => {
            total_biomass >= *min_biomass
        }
        crate::level::Objective::EcosystemStability { min_trophic_levels, .. } => {
            trophic_levels >= *min_trophic_levels
        }
        crate::level::Objective::BiodiversityStability { min_species, .. } => {
            biodiversity >= *min_species
        }
    };

    let extinct = total_biomass <= 0.0 && !h.sim.species().is_empty();

    let result = serde_json::json!({
        "condition_met": condition_met,
        "total_biomass": total_biomass,
        "biodiversity": biodiversity,
        "trophic_levels": trophic_levels,
        "extinct": extinct,
    });

    let json = serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string());
    h.objective_result_cache = CString::new(json).unwrap_or_else(|_| CString::new("{}").unwrap());
    h.objective_result_cache.as_ptr()
}

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

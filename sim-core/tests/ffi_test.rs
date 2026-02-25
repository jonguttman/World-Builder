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
    assert!((-80.0..=80.0).contains(&temp_val), "temp={}", temp_val);

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

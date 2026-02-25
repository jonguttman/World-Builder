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

// --- Objective Evaluation ---
double pa_sim_snapshot_total_biomass(PASimHandle handle);
const char* pa_sim_evaluate_objective(PASimHandle handle, const char* objective_json);

// --- Codex ---
// Returns JSON array of all codex entry definitions.
const char* pa_sim_codex_all_entries_json(PASimHandle handle);
// Returns JSON array of unlocked entry ID strings.
const char* pa_sim_codex_unlocked_json(PASimHandle handle);
// Returns JSON array of entry IDs unlocked since last call. Clears the buffer.
const char* pa_sim_codex_new_unlocks_json(PASimHandle handle);

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

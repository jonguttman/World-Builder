use rand_chacha::ChaCha8Rng;
use rand::Rng;

use crate::types::*;

/// Initialize grid tiles from planet params and seed
pub fn init_grid(grid: &mut WorldGrid, params: &PlanetParams, rng: &mut ChaCha8Rng) {
    for y in 0..grid.height {
        let lat = grid.latitude(y);
        for x in 0..grid.width {
            let tile = grid.get_mut(x, y);
            tile.elevation = rng.gen_range(-1.0..1.0);
            tile.is_ocean = tile.elevation < 0.0 && (rng.gen::<f32>() < params.hydrology.ocean_coverage);
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
            tile.temperature = compute_temperature(lat, params, season_phase, tile.elevation);
        }
    }
}

fn base_temperature(lat: f32, params: &PlanetParams) -> f32 {
    let insolation = (lat.to_radians().cos()).max(0.0);
    let base = -20.0 + 50.0 * insolation;
    let greenhouse = 33.0 * (1.0 + params.atmosphere.co2).ln();
    let core = params.core_heat * 5.0;
    base + greenhouse + core
}

fn compute_temperature(lat: f32, params: &PlanetParams, season_phase: f32, elevation: f32) -> f32 {
    let base = base_temperature(lat, params);
    let seasonal = params.axial_tilt.to_radians().sin()
        * season_phase.sin()
        * lat.to_radians().sin()
        * 15.0;
    let albedo_cooling = params.hydrology.ice_fraction * 10.0;
    // Lapse rate: ~6.5 C per 1000m; elevation normalized to [-1, 1] maps to ~[-2000m, 8000m]
    let altitude_m = (elevation + 1.0) * 5000.0 - 2000.0;
    let lapse = if altitude_m > 0.0 { altitude_m * 0.0065 } else { 0.0 };
    let temp = base + seasonal - albedo_cooling - lapse;
    temp.clamp(-80.0, 80.0)
}

fn radiation_level(lat: f32, params: &PlanetParams) -> f32 {
    let base_radiation = (lat.to_radians().cos()).max(0.0);
    let shielding = params.magnetic_field;
    (base_radiation * (1.0 - shielding * 0.8)).clamp(0.0, 1.0)
}

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

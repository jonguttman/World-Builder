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
    extreme.atmosphere.co2 = 1.0;
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

    // Average temperature of equatorial band (rows 14-18) vs polar band (rows 0-3)
    let grid = &snap.grid;
    let mut equator_sum: f32 = 0.0;
    for y in 14..=18 {
        for x in 0..64 {
            equator_sum += grid.get(x, y).temperature;
        }
    }
    let equator_avg = equator_sum / (5.0 * 64.0);

    let mut pole_sum: f32 = 0.0;
    for y in 0..=3 {
        for x in 0..64 {
            pole_sum += grid.get(x, y).temperature;
        }
    }
    let pole_avg = pole_sum / (4.0 * 64.0);

    assert!(equator_avg > pole_avg,
        "Equator avg ({}) should be warmer than pole avg ({})", equator_avg, pole_avg);
}

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

fn avg_temperature(grid: &WorldGrid) -> f32 {
    let sum: f32 = grid.tiles.iter().map(|t| t.temperature).sum();
    sum / grid.tiles.len() as f32
}

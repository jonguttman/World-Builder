use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- Planet Parameters (player-controlled) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetParams {
    pub gravity: f32,
    pub rotation_rate: f32,
    pub axial_tilt: f32,
    pub core_heat: f32,
    pub magnetic_field: f32,
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
    pub pressure: f32,
    pub o2: f32,
    pub co2: f32,
    pub toxicity: f32,
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
    pub ocean_coverage: f32,
    pub salinity: f32,
    pub current_strength: f32,
    pub ice_fraction: f32,
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
    pub populations: HashMap<u32, f64>,
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
    pub dispersal: f32,
    pub mutation_rate: f32,
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
    pub step: u64,
}

// --- Simulation Time ---

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TimeSpeed {
    Observe,
    Adapt,
    Epoch,
    Eon,
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

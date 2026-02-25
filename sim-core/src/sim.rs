use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::types::*;
use crate::climate;
use crate::biosphere;
use crate::snapshot::SimSnapshot;

#[derive(Serialize, Deserialize)]
struct SimState {
    seed: u64,
    step_count: u64,
    params: PlanetParams,
    grid: WorldGrid,
    species: Vec<Species>,
    events: Vec<SimEvent>,
    next_species_id: u32,
    rng_state: Vec<u8>,
}

const SPECIATION_EPOCH: u64 = 1000;

#[derive(Debug)]
pub enum InterventionError {
    InvalidRegion,
}

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
                    &self.params,
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
            for y in 0..self.grid.height {
                for x in 0..self.grid.width {
                    let tile = self.grid.get_mut(x, y);
                    // Seed child on tiles with any existing population
                    let has_life = tile.populations.values().any(|&p| p > 10.0);
                    if has_life {
                        let suit = biosphere::suitability(&child.traits, tile, &self.params);
                        if suit > 0.2 {
                            tile.populations.insert(child.id, 10.0);
                        }
                    }
                }
            }
        }

        self.species.extend(new_species);
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

    pub fn species(&self) -> &[Species] {
        &self.species
    }

    pub fn grid(&self) -> &WorldGrid {
        &self.grid
    }

    pub fn params(&self) -> &PlanetParams {
        &self.params
    }

    pub fn events(&self) -> &[SimEvent] {
        &self.events
    }

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
            InterventionKind::AdjustCurrents { delta } => {
                self.params.hydrology.current_strength =
                    (self.params.hydrology.current_strength + delta).clamp(0.0, 1.0);
            }
            InterventionKind::AdjustSalinity { delta } => {
                self.params.hydrology.salinity =
                    (self.params.hydrology.salinity + delta).clamp(0.0, 1.0);
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

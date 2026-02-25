use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::types::*;
use crate::climate;
use crate::biosphere;
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

    fn tick(&mut self) {
        self.step_count += 1;
        climate::update(&mut self.grid, &self.params, self.step_count);
        climate::update_nutrients(&mut self.grid, &self.params);
        biosphere::update_grid(&mut self.grid, &self.species, &self.params.atmosphere);
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

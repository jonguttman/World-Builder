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

use serde::{Deserialize, Serialize};
use crate::types::PlanetParams;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Pack {
    #[serde(rename = "FREE")]
    Free,
    #[serde(rename = "PACK_CORE")]
    Core,
    #[serde(rename = "PACK_ADV")]
    Advanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelSpec {
    pub id: String,
    pub name: String,
    pub pack: Pack,
    pub description: String,
    pub starting_seed: u64,
    pub starting_params: Option<PlanetParams>,
    pub allowed_interventions: Vec<String>,
    pub energy_budget: f32,
    pub objective: Objective,
    pub fail_conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Objective {
    MicrobialStability {
        min_biomass: f64,
        required_duration_steps: u64,
    },
    EcosystemStability {
        min_trophic_levels: u32,
        required_duration_steps: u64,
    },
    BiodiversityStability {
        min_species: u32,
        max_climate_variance: f32,
        required_duration_steps: u64,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectiveStatus {
    InProgress,
    Complete,
    Failed,
}

pub struct ObjectiveEvaluator {
    objective: Objective,
    sustained_steps: u64,
}

impl ObjectiveEvaluator {
    pub fn new(objective: Objective) -> Self {
        Self {
            objective,
            sustained_steps: 0,
        }
    }

    pub fn evaluate(
        &mut self,
        total_biomass: f64,
        biodiversity: u32,
        _current_step: u64,
    ) -> ObjectiveStatus {
        match &self.objective {
            Objective::MicrobialStability { min_biomass, required_duration_steps } => {
                if total_biomass <= 0.0 && self.sustained_steps > 0 {
                    return ObjectiveStatus::Failed;
                }
                if total_biomass >= *min_biomass {
                    self.sustained_steps += 1;
                    if self.sustained_steps >= *required_duration_steps {
                        return ObjectiveStatus::Complete;
                    }
                } else {
                    self.sustained_steps = 0;
                }
                ObjectiveStatus::InProgress
            }
            Objective::EcosystemStability { min_trophic_levels, required_duration_steps } => {
                if biodiversity >= *min_trophic_levels {
                    self.sustained_steps += 1;
                    if self.sustained_steps >= *required_duration_steps {
                        return ObjectiveStatus::Complete;
                    }
                } else {
                    self.sustained_steps = 0;
                }
                ObjectiveStatus::InProgress
            }
            Objective::BiodiversityStability { min_species, required_duration_steps, .. } => {
                if biodiversity >= *min_species {
                    self.sustained_steps += 1;
                    if self.sustained_steps >= *required_duration_steps {
                        return ObjectiveStatus::Complete;
                    }
                } else {
                    self.sustained_steps = 0;
                }
                ObjectiveStatus::InProgress
            }
        }
    }

    pub fn sustained_steps(&self) -> u64 {
        self.sustained_steps
    }
}

use serde::{Deserialize, Serialize};

use super::spec_reqs::{FunctionalRequirement, GeneralGoal, NonFunctionalRequirement};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpecData {
    pub functional_requirements: Vec<FunctionalRequirement>,
    pub non_functional_requirements: Vec<NonFunctionalRequirement>,
    pub general_goals: Vec<GeneralGoal>,
    pub stakeholders: Vec<String>,
}

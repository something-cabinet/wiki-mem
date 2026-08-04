use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize)]
pub struct AcceptanceCriterionFm {
    pub text: String,
    #[serde(default)]
    pub checked: bool,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum AcceptanceCriterionRepr {
    Text(String),
    Full {
        text: String,
        #[serde(default)]
        checked: bool,
    },
}

impl<'de> Deserialize<'de> for AcceptanceCriterionFm {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match AcceptanceCriterionRepr::deserialize(deserializer)? {
            AcceptanceCriterionRepr::Text(text) => Ok(Self {
                text,
                checked: false,
            }),
            AcceptanceCriterionRepr::Full { text, checked } => Ok(Self { text, checked }),
        }
    }
}

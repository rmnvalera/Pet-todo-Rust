use serde::{Deserialize, Deserializer};
use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct HumanDuration(pub Duration);

impl HumanDuration {
    pub fn as_secs(&self) -> u64 {
        self.0.as_secs()
    }
}

impl<'de> Deserialize<'de> for HumanDuration {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        humantime::parse_duration(&s)
            .map(HumanDuration)
            .map_err(serde::de::Error::custom)
    }
}

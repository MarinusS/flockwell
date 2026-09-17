use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnimalId(Uuid);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LambingId(Uuid);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DispositionId(Uuid);

#[derive(Debug, PartialEq, Eq)]
pub enum Sex {
    Male,
    Female,
    Unknown,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LifeStage {
    Lamb,
    Sheep,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Animal {
    pub id: AnimalId,
    pub tag: Option<String>,
    pub tip_tag: Option<String>,
    pub uhf_tag: Option<String>,
    pub uhf_tag_visual: Option<String>,
    pub sex: Sex,
    pub life_stage_override: Option<LifeStage>,
    pub lambing_id: Option<LambingId>,
    pub disposition_id: Option<DispositionId>,
}

impl AnimalId {
    pub fn new(value: Uuid) -> Self {
        Self(value)
    }
}

impl std::str::FromStr for AnimalId {
    type Err = uuid::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(Uuid::parse_str(value)?))
    }
}

impl From<&str> for AnimalId {
    fn from(value: &str) -> Self {
        Self::new(Uuid::parse_str(value).expect("AnimalId must be a UUID"))
    }
}

impl std::fmt::Display for AnimalId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Animal {
    pub fn new(id: impl Into<AnimalId>, tag: Option<&str>) -> Self {
        Self {
            id: id.into(),
            tag: tag.map(str::to_owned),
            tip_tag: None,
            uhf_tag: None,
            uhf_tag_visual: None,
            sex: Sex::Unknown,
            life_stage_override: None,
            lambing_id: None,
            disposition_id: None,
        }
    }
}

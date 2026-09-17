use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnimalId(Uuid);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LambingId(Uuid);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DispositionId(Uuid);

#[derive(Debug)]
pub enum AnimalIdParseError {
    InvalidUuid(uuid::Error),
    NotUuidV7,
}

impl std::fmt::Display for AnimalIdParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUuid(error) => write!(formatter, "invalid UUID: {error}"),
            Self::NotUuidV7 => write!(formatter, "AnimalId must be a UUIDv7"),
        }
    }
}

impl std::error::Error for AnimalIdParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidUuid(error) => Some(error),
            Self::NotUuidV7 => None,
        }
    }
}

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
    pub fn from_uuid(value: Uuid) -> Result<Self, AnimalIdParseError> {
        if value.get_version_num() != 7 {
            return Err(AnimalIdParseError::NotUuidV7);
        }

        Ok(Self(value))
    }
}

impl std::str::FromStr for AnimalId {
    type Err = AnimalIdParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let uuid = Uuid::parse_str(value).map_err(AnimalIdParseError::InvalidUuid)?;
        Self::from_uuid(uuid)
    }
}

impl std::fmt::Display for AnimalId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Animal {
    pub fn new(id: AnimalId, tag: Option<&str>) -> Self {
        Self {
            id,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn animal_id_accepts_uuid_v7() {
        let value = "00000000-0000-7000-8000-000000000001";

        let id = AnimalId::from_str(value).expect("UUIDv7 should be accepted");

        assert_eq!(id.to_string(), value);
    }

    #[test]
    fn animal_id_rejects_other_uuid_versions() {
        let value = "00000000-0000-4000-8000-000000000001";

        assert!(matches!(
            AnimalId::from_str(value),
            Err(AnimalIdParseError::NotUuidV7)
        ));
    }

    #[test]
    fn animal_id_rejects_invalid_uuid() {
        assert!(matches!(
            AnimalId::from_str("not-a-uuid"),
            Err(AnimalIdParseError::InvalidUuid(_))
        ));
    }
}

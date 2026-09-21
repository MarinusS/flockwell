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
            Self::NotUuidV7 => write!(formatter, "ID must be a UUIDv7"),
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
    Adult,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Animal {
    pub id: AnimalId,
    pub tag: Option<String>,
    pub comment: Option<String>,
    pub tip_tag: Option<String>,
    pub uhf_tag: Option<String>,
    pub uhf_tag_visual: Option<String>,
    pub sex: Sex,
    pub life_stage_override: Option<LifeStage>,
    pub lambing_id: Option<LambingId>,
    pub disposition_id: Option<DispositionId>,
}

fn parse_uuid_v7(value: &str) -> Result<Uuid, AnimalIdParseError> {
    let uuid = Uuid::parse_str(value).map_err(AnimalIdParseError::InvalidUuid)?;

    if uuid.get_version_num() != 7 {
        return Err(AnimalIdParseError::NotUuidV7);
    }

    Ok(uuid)
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
        Ok(Self(parse_uuid_v7(value)?))
    }
}

impl std::fmt::Display for AnimalId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl LambingId {
    pub fn from_uuid(value: Uuid) -> Result<Self, AnimalIdParseError> {
        if value.get_version_num() != 7 {
            return Err(AnimalIdParseError::NotUuidV7);
        }

        Ok(Self(value))
    }
}

impl std::str::FromStr for LambingId {
    type Err = AnimalIdParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self(parse_uuid_v7(value)?))
    }
}

impl std::fmt::Display for LambingId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl DispositionId {
    pub fn from_uuid(value: Uuid) -> Result<Self, AnimalIdParseError> {
        if value.get_version_num() != 7 {
            return Err(AnimalIdParseError::NotUuidV7);
        }

        Ok(Self(value))
    }
}

impl std::str::FromStr for DispositionId {
    type Err = AnimalIdParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self(parse_uuid_v7(value)?))
    }
}

impl std::fmt::Display for DispositionId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Animal {
    pub fn new(id: AnimalId) -> Self {
        Self {
            id,
            tag: None,
            comment: None,
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

    #[test]
    fn related_ids_use_the_same_uuid_v7_validation() {
        let valid = "00000000-0000-7000-8000-000000000001";
        let invalid_version = "00000000-0000-4000-8000-000000000001";

        assert!(LambingId::from_str(valid).is_ok());
        assert!(DispositionId::from_str(valid).is_ok());
        assert!(matches!(
            LambingId::from_str(invalid_version),
            Err(AnimalIdParseError::NotUuidV7)
        ));
        assert!(matches!(
            DispositionId::from_str(invalid_version),
            Err(AnimalIdParseError::NotUuidV7)
        ));
    }

    #[test]
    fn animal_new_sets_default_state() {
        let id = AnimalId::from_str("00000000-0000-7000-8000-000000000001")
            .expect("test ID is a valid UUIDv7");

        let animal = Animal::new(id);

        assert_eq!(animal.id, id);
        assert_eq!(animal.tag, None);
        assert_eq!(animal.comment, None);
        assert_eq!(animal.tip_tag, None);
        assert_eq!(animal.uhf_tag, None);
        assert_eq!(animal.uhf_tag_visual, None);
        assert_eq!(animal.sex, Sex::Unknown);
        assert_eq!(animal.life_stage_override, None);
        assert_eq!(animal.lambing_id, None);
        assert_eq!(animal.disposition_id, None);
    }
}

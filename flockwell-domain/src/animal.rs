use crate::{Tag, TipTag, UhfTag, UhfTagVisual};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnimalId(Uuid);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LambingId(Uuid);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DispositionId(Uuid);

#[derive(Debug)]
pub enum UuidV7ParseError {
    InvalidUuid(uuid::Error),
    NotUuidV7,
}

impl std::fmt::Display for UuidV7ParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUuid(error) => write!(formatter, "invalid UUID: {error}"),
            Self::NotUuidV7 => write!(formatter, "ID must be a UUIDv7"),
        }
    }
}

impl std::error::Error for UuidV7ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidUuid(error) => Some(error),
            Self::NotUuidV7 => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sex {
    Male,
    Female,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifeStage {
    Lamb,
    Adult,
}

/// Individually valid animal data; collection uniqueness is checked separately.
///
/// ```
/// use flockwell_domain::{Animal, AnimalData};
/// let id = "00000000-0000-7000-8000-000000000001".parse()?;
/// let animal = Animal::from_data(id, AnimalData {
///     tag: Some(" 000000000000042 ".parse()?),
///     ..AnimalData::default()
/// });
/// assert_eq!(animal.tag().unwrap().as_str(), "000000000000042");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// Tag namespaces cannot be mixed:
/// ```compile_fail
/// use flockwell_domain::{AnimalData, UhfTag};
/// let uhf: UhfTag = "E200001".parse().unwrap();
/// let data = AnimalData { tag: Some(uhf), ..AnimalData::default() };
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct Animal {
    id: AnimalId,
    data: AnimalData,
}

/// Typed values used to construct an animal. Does not assert uniqueness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimalData {
    pub tag: Option<Tag>,
    pub comment: Option<String>,
    pub tip_tag: Option<TipTag>,
    pub uhf_tag: Option<UhfTag>,
    pub uhf_tag_visual: Option<UhfTagVisual>,
    pub sex: Sex,
    pub life_stage_override: Option<LifeStage>,
    pub lambing_id: Option<LambingId>,
    pub disposition_id: Option<DispositionId>,
}

fn parse_uuid_v7(value: &str) -> Result<Uuid, UuidV7ParseError> {
    let uuid = Uuid::parse_str(value).map_err(UuidV7ParseError::InvalidUuid)?;

    if uuid.get_version_num() != 7 {
        return Err(UuidV7ParseError::NotUuidV7);
    }

    Ok(uuid)
}

impl AnimalId {
    pub fn from_uuid(value: Uuid) -> Result<Self, UuidV7ParseError> {
        if value.get_version_num() != 7 {
            return Err(UuidV7ParseError::NotUuidV7);
        }

        Ok(Self(value))
    }
}

impl std::str::FromStr for AnimalId {
    type Err = UuidV7ParseError;

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
    pub fn from_uuid(value: Uuid) -> Result<Self, UuidV7ParseError> {
        if value.get_version_num() != 7 {
            return Err(UuidV7ParseError::NotUuidV7);
        }

        Ok(Self(value))
    }
}

impl std::str::FromStr for LambingId {
    type Err = UuidV7ParseError;

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
    pub fn from_uuid(value: Uuid) -> Result<Self, UuidV7ParseError> {
        if value.get_version_num() != 7 {
            return Err(UuidV7ParseError::NotUuidV7);
        }

        Ok(Self(value))
    }
}

impl std::str::FromStr for DispositionId {
    type Err = UuidV7ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self(parse_uuid_v7(value)?))
    }
}

impl std::fmt::Display for DispositionId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Default for AnimalData {
    fn default() -> Self {
        Self {
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

impl Animal {
    pub fn new(id: AnimalId) -> Self {
        Self::from_data(id, AnimalData::default())
    }

    pub fn from_data(id: AnimalId, data: AnimalData) -> Self {
        Self { id, data }
    }

    pub fn id(&self) -> AnimalId {
        self.id
    }

    pub fn data(&self) -> &AnimalData {
        &self.data
    }

    pub fn tag(&self) -> Option<&Tag> {
        self.data.tag.as_ref()
    }

    pub fn comment(&self) -> Option<&str> {
        self.data.comment.as_deref()
    }

    pub fn tip_tag(&self) -> Option<&TipTag> {
        self.data.tip_tag.as_ref()
    }

    pub fn uhf_tag(&self) -> Option<&UhfTag> {
        self.data.uhf_tag.as_ref()
    }

    pub fn uhf_tag_visual(&self) -> Option<&UhfTagVisual> {
        self.data.uhf_tag_visual.as_ref()
    }

    pub fn sex(&self) -> &Sex {
        &self.data.sex
    }

    pub fn life_stage_override(&self) -> Option<&LifeStage> {
        self.data.life_stage_override.as_ref()
    }

    pub fn lambing_id(&self) -> Option<&LambingId> {
        self.data.lambing_id.as_ref()
    }

    pub fn disposition_id(&self) -> Option<&DispositionId> {
        self.data.disposition_id.as_ref()
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
            Err(UuidV7ParseError::NotUuidV7)
        ));
    }

    #[test]
    fn animal_id_rejects_invalid_uuid() {
        assert!(matches!(
            AnimalId::from_str("not-a-uuid"),
            Err(UuidV7ParseError::InvalidUuid(_))
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
            Err(UuidV7ParseError::NotUuidV7)
        ));
        assert!(matches!(
            DispositionId::from_str(invalid_version),
            Err(UuidV7ParseError::NotUuidV7)
        ));
    }

    #[test]
    fn animal_new_sets_default_state() {
        let id = AnimalId::from_str("00000000-0000-7000-8000-000000000001")
            .expect("test ID is a valid UUIDv7");

        let animal = Animal::new(id);

        assert_eq!(animal.id, id);
        assert_eq!(animal.data.tag, None);
        assert_eq!(animal.data.comment, None);
        assert_eq!(animal.data.tip_tag, None);
        assert_eq!(animal.data.uhf_tag, None);
        assert_eq!(animal.data.uhf_tag_visual, None);
        assert_eq!(animal.data.sex, Sex::Unknown);
        assert_eq!(animal.data.life_stage_override, None);
        assert_eq!(animal.data.lambing_id, None);
        assert_eq!(animal.data.disposition_id, None);
    }
}

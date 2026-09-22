mod animal;
mod lambing;
mod audit;
mod registry;
mod tag;
mod uniqueness;

pub use animal::{
    Animal, AnimalData, AnimalId, DispositionId, LambingId, LifeStage, Sex, UuidV7ParseError,
};

pub use lambing::{Lambing};

pub use audit::{AuditIssue, AuditReport, RecordRef, audit_animals};
pub use registry::{
    AnimalRegistry, Change, CreateAnimal, CreateAnimalError, TagConflict, UpdateAnimal,
    UpdateAnimalError,
};
pub use tag::{Tag, TagParseError, TipTag, UhfTag, UhfTagVisual};

/// Compatibility alias for callers using the original ID error name.
pub type AnimalIdParseError = UuidV7ParseError;

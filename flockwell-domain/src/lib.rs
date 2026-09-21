mod animal;
mod audit;
mod tag;

pub use animal::{
    Animal, AnimalData, AnimalId, DispositionId, LambingId, LifeStage, Sex, UuidV7ParseError,
};
pub use audit::{AuditIssue, AuditReport, RecordRef, audit_animals};
pub use tag::{Tag, TagParseError, TipTag, UhfTag, UhfTagVisual};

/// Compatibility alias for callers using the original ID error name.
pub type AnimalIdParseError = UuidV7ParseError;

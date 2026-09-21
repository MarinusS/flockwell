use std::collections::BTreeMap;

use crate::{
    Animal, AnimalData, AnimalId, AuditReport, DispositionId, LambingId, LifeStage, Sex, Tag,
    TipTag, UhfTag, UhfTagVisual, audit_animals,
    uniqueness::{UniqueTag, unique_tags},
};

/// A patch for an optional field. `Keep` and `Clear` have different meanings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Change<T> {
    #[default]
    Keep,
    Set(T),
    Clear,
}

impl<T: Clone> Change<T> {
    fn apply_to(&self, value: &mut Option<T>) {
        match self {
            Self::Keep => {}
            Self::Set(new) => *value = Some(new.clone()),
            Self::Clear => *value = None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateAnimal {
    pub id: AnimalId,
    pub data: AnimalData,
}

/// Identifies an existing animal; its identity cannot be changed by a patch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateAnimal {
    pub id: AnimalId,
    pub tag: Change<Tag>,
    pub comment: Change<String>,
    pub tip_tag: Change<TipTag>,
    pub uhf_tag: Change<UhfTag>,
    pub uhf_tag_visual: Change<UhfTagVisual>,
    /// `None` keeps the current sex; `Some(Unknown)` explicitly resets it.
    pub sex: Option<Sex>,
    pub life_stage_override: Change<LifeStage>,
    pub lambing_id: Change<LambingId>,
    pub disposition_id: Change<DispositionId>,
}

impl UpdateAnimal {
    /// Starts a patch that leaves every field unchanged.
    pub fn new(id: AnimalId) -> Self {
        Self {
            id,
            tag: Change::Keep,
            comment: Change::Keep,
            tip_tag: Change::Keep,
            uhf_tag: Change::Keep,
            uhf_tag_visual: Change::Keep,
            sex: None,
            life_stage_override: Change::Keep,
            lambing_id: Change::Keep,
            disposition_id: Change::Keep,
        }
    }

    fn apply_to(&self, data: &mut AnimalData) {
        self.tag.apply_to(&mut data.tag);
        self.comment.apply_to(&mut data.comment);
        self.tip_tag.apply_to(&mut data.tip_tag);
        self.uhf_tag.apply_to(&mut data.uhf_tag);
        self.uhf_tag_visual.apply_to(&mut data.uhf_tag_visual);
        if let Some(sex) = &self.sex {
            data.sex = sex.clone();
        }
        self.life_stage_override
            .apply_to(&mut data.life_stage_override);
        self.lambing_id.apply_to(&mut data.lambing_id);
        self.disposition_id.apply_to(&mut data.disposition_id);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagConflict {
    TagAlreadyAssigned { tag: Tag, owner: AnimalId },
    UhfTagAlreadyAssigned { tag: UhfTag, owner: AnimalId },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateAnimalError {
    IdAlreadyExists(AnimalId),
    Conflict(TagConflict),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateAnimalError {
    AnimalNotFound(AnimalId),
    Conflict(TagConflict),
}

/// A collection with unique IDs, tags and UHF tags within its supplied scope.
/// Disposed animals participate. Tag namespaces are independent.
///
/// Preview validation only describes the current state. Mutations always
/// validate again. This in-memory collection cannot guarantee uniqueness
/// against data elsewhere or replace atomic enforcement in persistent storage.
///
/// ```
/// use flockwell_domain::{AnimalRegistry, AnimalData, CreateAnimal, UpdateAnimal, Change};
/// let id = "00000000-0000-7000-8000-000000000001".parse().unwrap();
/// let mut registry = AnimalRegistry::new();
/// registry.create(CreateAnimal {
///     id,
///     data: AnimalData { tag: Some("000000000000042".parse().unwrap()), ..AnimalData::default() },
/// }).unwrap();
/// let mut patch = UpdateAnimal::new(id);
/// patch.tag = Change::Clear;
/// assert!(registry.validate_update(&patch).is_empty());
/// registry.update(patch).unwrap();
/// assert!(registry.get(id).unwrap().tag().is_none());
/// ```
#[derive(Debug, Default, PartialEq, Eq)]
pub struct AnimalRegistry {
    animals: BTreeMap<AnimalId, Animal>,
    tag_owners: BTreeMap<UniqueTag, AnimalId>,
}

impl AnimalRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Audits the complete input before building maps, so duplicate IDs never
    /// overwrite records. Rejection includes every audit finding and position.
    pub fn try_from_animals(animals: Vec<Animal>) -> Result<Self, AuditReport> {
        let report = audit_animals(&animals);
        if report.has_errors() {
            return Err(report);
        }
        let mut registry = Self::new();
        for animal in animals {
            registry.insert_validated(animal);
        }
        Ok(registry)
    }

    pub fn get(&self, id: AnimalId) -> Option<&Animal> {
        self.animals.get(&id)
    }

    /// Read-only iteration in ascending animal ID order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &Animal> {
        self.animals.values()
    }

    pub fn len(&self) -> usize {
        self.animals.len()
    }

    pub fn is_empty(&self) -> bool {
        self.animals.is_empty()
    }

    /// Returns all conflicts in ID, tag, UHF order without changing state.
    pub fn validate_create(&self, command: &CreateAnimal) -> Vec<CreateAnimalError> {
        let mut errors = Vec::new();
        if self.animals.contains_key(&command.id) {
            errors.push(CreateAnimalError::IdAlreadyExists(command.id));
        }
        errors.extend(
            self.conflicts(&command.data, None)
                .into_iter()
                .map(CreateAnimalError::Conflict),
        );
        errors
    }

    /// A rejected command leaves both the animals and indexes unchanged.
    pub fn create(&mut self, command: CreateAnimal) -> Result<AnimalId, Vec<CreateAnimalError>> {
        let errors = self.validate_create(&command);
        if !errors.is_empty() {
            return Err(errors);
        }
        self.insert_validated(Animal::from_data(command.id, command.data));
        Ok(command.id)
    }

    /// Excludes only the existing target animal from tag conflicts.
    pub fn validate_update(&self, command: &UpdateAnimal) -> Vec<UpdateAnimalError> {
        self.prepare_update(command).err().unwrap_or_default()
    }

    /// Validates the complete proposed state before changing animals or indexes.
    pub fn update(&mut self, command: UpdateAnimal) -> Result<(), Vec<UpdateAnimalError>> {
        let data = self.prepare_update(&command)?;
        // prepare_update guarantees this target exists; no fallible domain work
        // remains once index changes begin.
        let old = self
            .animals
            .get(&command.id)
            .expect("validated target exists");
        for tag in unique_tags(old.data()) {
            self.tag_owners.remove(&tag);
        }
        self.insert_validated(Animal::from_data(command.id, data));
        Ok(())
    }

    fn prepare_update(&self, command: &UpdateAnimal) -> Result<AnimalData, Vec<UpdateAnimalError>> {
        let Some(animal) = self.get(command.id) else {
            return Err(vec![UpdateAnimalError::AnimalNotFound(command.id)]);
        };
        let mut data = animal.data().clone();
        command.apply_to(&mut data);
        let errors: Vec<_> = self
            .conflicts(&data, Some(command.id))
            .into_iter()
            .map(UpdateAnimalError::Conflict)
            .collect();
        if errors.is_empty() {
            Ok(data)
        } else {
            Err(errors)
        }
    }

    fn conflicts(&self, data: &AnimalData, exclude: Option<AnimalId>) -> Vec<TagConflict> {
        unique_tags(data)
            .filter_map(|tag| {
                let owner = *self.tag_owners.get(&tag)?;
                if Some(owner) == exclude {
                    return None;
                }
                Some(match tag {
                    UniqueTag::Tag(tag) => TagConflict::TagAlreadyAssigned { tag, owner },
                    UniqueTag::UhfTag(tag) => TagConflict::UhfTagAlreadyAssigned { tag, owner },
                })
            })
            .collect()
    }

    fn insert_validated(&mut self, animal: Animal) {
        for tag in unique_tags(animal.data()) {
            self.tag_owners.insert(tag, animal.id());
        }
        self.animals.insert(animal.id(), animal);
    }
}

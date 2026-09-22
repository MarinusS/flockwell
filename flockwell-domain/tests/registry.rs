use flockwell_domain::{
    Animal, AnimalData, AnimalId, AnimalRegistry, AuditIssue, Change, CreateAnimal,
    CreateAnimalError, LifeStage, Sex, TagConflict, UpdateAnimal, UpdateAnimalError, audit_animals,
};

fn id(n: u8) -> AnimalId {
    format!("00000000-0000-7000-8000-{n:012}").parse().unwrap()
}

fn command(n: u8, tag: Option<&str>, uhf: Option<&str>) -> CreateAnimal {
    CreateAnimal {
        id: id(n),
        data: AnimalData {
            tag: tag.map(|s| s.parse().unwrap()),
            uhf_tag: uhf.map(|s| s.parse().unwrap()),
            ..AnimalData::default()
        },
    }
}

fn animal(command: CreateAnimal) -> Animal {
    Animal::from_data(command.id, command.data)
}

fn populated() -> AnimalRegistry {
    AnimalRegistry::try_from_animals(vec![
        animal(command(1, Some("000000000000001"), Some("A"))),
        animal(command(2, Some("000000000000002"), Some("B"))),
    ])
    .unwrap()
}

fn assert_audit_passes(registry: &AnimalRegistry) {
    let animals: Vec<_> = registry
        .iter()
        .map(|a| Animal::from_data(a.id().clone(), a.data().clone()))
        .collect();
    assert!(!audit_animals(&animals).has_errors());
}

#[test]
fn loading_checks_all_records_before_ids_can_be_overwritten() {
    let report = AnimalRegistry::try_from_animals(vec![
        animal(command(1, Some("000000000000001"), Some("A"))),
        animal(command(1, Some("000000000000001"), Some("A"))),
        animal(command(2, Some("000000000000001"), Some("A"))),
    ])
    .unwrap_err();
    assert_eq!(report.issues().len(), 3);
    assert!(matches!(report.issues()[0], AuditIssue::DuplicateId { .. }));
    assert_eq!(
        report.issues()[0]
            .records()
            .iter()
            .map(|r| r.index)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(report.issues()[1].records().len(), 3);
    assert_eq!(report.issues()[2].records().len(), 3);
}

#[test]
fn empty_loading_and_creation_support_missing_tags() {
    let mut registry = AnimalRegistry::try_from_animals(vec![]).unwrap();
    assert!(registry.is_empty());
    assert_eq!(registry.get(id(1)), None);
    assert_eq!(registry.create(command(2, None, None)), Ok(id(2)));
    assert_eq!(registry.create(command(1, None, None)), Ok(id(1)));
    assert_eq!(registry.len(), 2);
    assert_eq!(
        registry.iter().map(|animal| animal.id().clone()).collect::<Vec<_>>(),
        [id(1).clone(), id(2).clone()]
    );
    assert_audit_passes(&registry);
}

#[test]
fn create_collects_every_conflict_and_never_excludes_matching_ids() {
    let mut registry = populated();
    let command = command(1, Some("000000000000001"), Some("B"));
    let expected = vec![
        CreateAnimalError::IdAlreadyExists(id(1)),
        CreateAnimalError::Conflict(TagConflict::TagAlreadyAssigned {
            tag: "000000000000001".parse().unwrap(),
            owner: id(1),
        }),
        CreateAnimalError::Conflict(TagConflict::UhfTagAlreadyAssigned {
            tag: "B".parse().unwrap(),
            owner: id(2),
        }),
    ];
    assert_eq!(registry.validate_create(&command), expected);
    assert_eq!(registry, populated());
    assert_eq!(registry.create(command), Err(expected));
    assert_eq!(registry, populated());
}

#[test]
fn unchanged_values_pass_and_rejected_updates_change_nothing() {
    let mut registry = populated();
    let mut patch = UpdateAnimal::new(id(1));
    assert_eq!(registry.update(patch.clone()), Ok(()));
    patch.tag = Change::Set("000000000000001".parse().unwrap());
    patch.uhf_tag = Change::Set("A".parse().unwrap());
    assert!(registry.validate_update(&patch).is_empty());
    assert_eq!(registry.update(patch.clone()), Ok(()));
    assert_eq!(registry, populated());
    patch.tag = Change::Set("000000000000002".parse().unwrap());
    patch.uhf_tag = Change::Set("B".parse().unwrap());
    patch.comment = Change::Set("must not be applied".into());
    let expected = vec![
        UpdateAnimalError::Conflict(TagConflict::TagAlreadyAssigned {
            tag: "000000000000002".parse().unwrap(),
            owner: id(2),
        }),
        UpdateAnimalError::Conflict(TagConflict::UhfTagAlreadyAssigned {
            tag: "B".parse().unwrap(),
            owner: id(2),
        }),
    ];
    assert_eq!(registry.validate_update(&patch), expected);
    assert_eq!(registry, populated());
    assert_eq!(registry.update(patch), Err(expected));
    assert_eq!(registry, populated());
}

#[test]
fn a_free_tag_is_not_reserved_when_another_field_conflicts() {
    let mut registry = populated();
    let candidate = command(3, Some("000000000000003"), Some("B"));
    assert!(registry.create(candidate).is_err());
    let mut patch = UpdateAnimal::new(id(1));
    patch.tag = Change::Set("000000000000003".parse().unwrap());
    patch.uhf_tag = Change::Set("B".parse().unwrap());
    assert!(registry.update(patch).is_err());
    assert_eq!(registry, populated());
    registry
        .create(command(3, Some("000000000000003"), Some("C")))
        .unwrap();
    assert_audit_passes(&registry);
}

#[test]
fn missing_update_target_is_typed_and_cannot_insert() {
    let mut registry = populated();
    let patch = UpdateAnimal::new(id(9));
    let expected = vec![UpdateAnimalError::AnimalNotFound(id(9))];
    assert_eq!(registry.validate_update(&patch), expected);
    assert_eq!(registry.update(patch), Err(expected));
    assert_eq!(registry, populated());
}

#[test]
fn replacing_and_clearing_release_both_tag_indexes() {
    let mut registry = populated();
    let mut patch = UpdateAnimal::new(id(1));
    patch.tag = Change::Set("000000000000003".parse().unwrap());
    patch.uhf_tag = Change::Set("C".parse().unwrap());
    registry.update(patch).unwrap();
    registry
        .create(command(3, Some("000000000000001"), Some("A")))
        .unwrap();
    assert_eq!(
        registry
            .validate_create(&command(4, Some("000000000000003"), Some("C")))
            .len(),
        2
    );
    let mut patch = UpdateAnimal::new(id(1));
    patch.tag = Change::Clear;
    patch.uhf_tag = Change::Clear;
    registry.update(patch).unwrap();
    registry
        .create(command(4, Some("000000000000003"), Some("C")))
        .unwrap();
    assert_eq!(registry.get(id(1)).unwrap().tag(), None);
    assert_eq!(registry.get(id(1)).unwrap().uhf_tag(), None);
    assert_audit_passes(&registry);
}

#[test]
fn mutations_revalidate_after_a_successful_preview() {
    let mut registry = populated();
    let candidate = command(3, Some("000000000000003"), Some("C"));
    let mut patch = UpdateAnimal::new(id(1));
    patch.tag = Change::Set("000000000000003".parse().unwrap());
    patch.uhf_tag = Change::Set("C".parse().unwrap());
    assert!(registry.validate_create(&candidate).is_empty());
    assert!(registry.validate_update(&patch).is_empty());
    registry
        .create(command(4, Some("000000000000003"), Some("C")))
        .unwrap();
    assert_eq!(registry.create(candidate).unwrap_err().len(), 2);
    assert_eq!(registry.update(patch).unwrap_err().len(), 2);
    assert_eq!(
        registry.get(id(1)).unwrap().tag().unwrap().as_str(),
        "000000000000001"
    );
    assert_audit_passes(&registry);
}

#[test]
fn every_non_unique_field_supports_patch_semantics() {
    let mut registry = populated();
    let mut patch = UpdateAnimal::new(id(1));
    patch.comment = Change::Set("hello".into());
    patch.tip_tag = Change::Set("TIP".parse().unwrap());
    patch.uhf_tag_visual = Change::Set("VISUAL".parse().unwrap());
    patch.sex = Some(Sex::Female);
    patch.life_stage_override = Change::Set(LifeStage::Adult);
    patch.lambing_id = Change::Set(id(8).to_string().parse().unwrap());
    patch.disposition_id = Change::Set(id(9).to_string().parse().unwrap());
    registry.update(patch).unwrap();
    let expected = AnimalData {
        tag: Some("000000000000001".parse().unwrap()),
        uhf_tag: Some("A".parse().unwrap()),
        comment: Some("hello".into()),
        tip_tag: Some("TIP".parse().unwrap()),
        uhf_tag_visual: Some("VISUAL".parse().unwrap()),
        sex: Sex::Female,
        life_stage_override: Some(LifeStage::Adult),
        lambing_id: Some(id(8).to_string().parse().unwrap()),
        disposition_id: Some(id(9).to_string().parse().unwrap()),
    };
    registry.update(UpdateAnimal::new(id(1))).unwrap();
    assert_eq!(registry.get(id(1)).unwrap().data(), &expected);
    // Disposition does not free identifiers.
    assert_eq!(
        registry
            .validate_create(&command(3, Some("000000000000001"), Some("A")))
            .len(),
        2
    );
    let mut patch = UpdateAnimal::new(id(1));
    patch.comment = Change::Clear;
    patch.tip_tag = Change::Clear;
    patch.uhf_tag_visual = Change::Clear;
    patch.sex = Some(Sex::Unknown);
    patch.life_stage_override = Change::Clear;
    patch.lambing_id = Change::Clear;
    patch.disposition_id = Change::Clear;
    registry.update(patch).unwrap();
    assert_eq!(registry, populated());
}

#[test]
fn audit_and_registry_agree_on_normalization_and_namespaces() {
    let mut registry = AnimalRegistry::new();
    registry
        .create(command(1, Some(" 000000000000001 "), Some("ABC")))
        .unwrap();
    registry
        .create(command(2, Some("000000000000002"), Some("000000000000001")))
        .unwrap();
    registry
        .create(command(3, Some("000000000000003"), Some("DEF")))
        .unwrap();
    registry
        .create(command(4, Some("000000000000004"), None))
        .unwrap();
    assert_eq!(
        registry
            .validate_create(&command(5, Some("000000000000001"), Some(" aBc ")))
            .len(),
        2
    );
    assert_audit_passes(&registry);
}

#[test]
fn duplicate_tip_and_visual_tags_are_allowed_on_load_create_and_update() {
    let mut first = command(1, None, None);
    first.data.tip_tag = Some("tip".parse().unwrap());
    first.data.uhf_tag_visual = Some("visual".parse().unwrap());
    let mut second = command(2, None, None);
    second.data.tip_tag = Some("TIP".parse().unwrap());
    second.data.uhf_tag_visual = Some("VISUAL".parse().unwrap());
    let mut registry =
        AnimalRegistry::try_from_animals(vec![animal(first), animal(second)]).unwrap();
    let third = CreateAnimal {
        id: id(3),
        data: registry.get(id(1)).unwrap().data().clone(),
    };
    registry.create(third).unwrap();
    let mut patch = UpdateAnimal::new(id(3));
    patch.tip_tag = Change::Set("TiP".parse().unwrap());
    patch.uhf_tag_visual = Change::Set("ViSuAl".parse().unwrap());
    registry.update(patch).unwrap();
    assert_audit_passes(&registry);
}

#[test]
fn uhf_update_is_case_insensitive_and_keeps_indexes_consistent() {
    let mut registry = populated();
    let mut patch = UpdateAnimal::new(id(1));
    patch.uhf_tag = Change::Set("a".parse().unwrap());
    registry.update(patch).unwrap();
    let mut conflict = UpdateAnimal::new(id(2));
    conflict.uhf_tag = Change::Set("a".parse().unwrap());
    assert_eq!(
        registry.update(conflict),
        Err(vec![UpdateAnimalError::Conflict(
            TagConflict::UhfTagAlreadyAssigned {
                tag: "A".parse().unwrap(),
                owner: id(1)
            }
        )])
    );
    assert_eq!(registry, populated());
    let mut clear = UpdateAnimal::new(id(1));
    clear.uhf_tag = Change::Clear;
    registry.update(clear).unwrap();
    registry.create(command(3, None, Some("a"))).unwrap();
    assert_audit_passes(&registry);
}

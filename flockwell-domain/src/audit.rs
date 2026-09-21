//! Pure collection checks. Uniqueness applies to all supplied animals, per field.
use crate::{Animal, AnimalId, Tag, UhfTag};
use std::collections::BTreeMap;

/// Position in the supplied slice, independent of potentially duplicated IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordRef {
    pub index: usize,
    pub animal_id: AnimalId,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AuditIssue {
    DuplicateId {
        id: AnimalId,
        records: Vec<RecordRef>,
    },
    DuplicateTag {
        tag: Tag,
        records: Vec<RecordRef>,
    },
    DuplicateUhfTag {
        tag: UhfTag,
        records: Vec<RecordRef>,
    },
}

impl AuditIssue {
    pub fn records(&self) -> &[RecordRef] {
        match self {
            Self::DuplicateId { records, .. }
            | Self::DuplicateTag { records, .. }
            | Self::DuplicateUhfTag { records, .. } => records,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct AuditReport {
    issues: Vec<AuditIssue>,
}

impl AuditReport {
    pub fn issues(&self) -> &[AuditIssue] {
        &self.issues
    }
    pub fn has_errors(&self) -> bool {
        !self.issues.is_empty()
    }
}

fn duplicates<K: Ord>(
    animals: &[Animal],
    key: impl Fn(&Animal) -> Option<K>,
) -> Vec<(K, Vec<RecordRef>)> {
    let mut groups = BTreeMap::<K, Vec<RecordRef>>::new();
    for (index, animal) in animals.iter().enumerate() {
        if let Some(key) = key(animal) {
            groups.entry(key).or_default().push(RecordRef {
                index,
                animal_id: animal.id(),
            });
        }
    }
    groups
        .into_iter()
        .filter_map(|(key, mut records)| {
            if records.len() < 2 {
                return None;
            }
            records.sort_by_key(|record| (record.animal_id, record.index));
            Some((key, records))
        })
        .collect()
}

/// Findings are ordered by field (ID, tag, UHF), then value; records by ID and index.
/// Missing tags are ignored. All animals, including disposed animals, participate.
pub fn audit_animals(animals: &[Animal]) -> AuditReport {
    let mut issues = Vec::new();
    issues.extend(
        duplicates(animals, |a| Some(a.id()))
            .into_iter()
            .map(|(id, records)| AuditIssue::DuplicateId { id, records }),
    );
    issues.extend(
        duplicates(animals, |a| a.tag().cloned())
            .into_iter()
            .map(|(tag, records)| AuditIssue::DuplicateTag { tag, records }),
    );
    issues.extend(
        duplicates(animals, |a| a.uhf_tag().cloned())
            .into_iter()
            .map(|(tag, records)| AuditIssue::DuplicateUhfTag { tag, records }),
    );
    AuditReport { issues }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AnimalData;

    fn animal(id: u8, tag: Option<&str>, uhf: Option<&str>) -> Animal {
        Animal::from_data(
            format!("00000000-0000-7000-8000-{id:012}").parse().unwrap(),
            AnimalData {
                tag: tag.map(|s| s.parse().unwrap()),
                uhf_tag: uhf.map(|s| s.parse().unwrap()),
                ..AnimalData::default()
            },
        )
    }

    #[test]
    fn empty_missing_and_unique_values_pass() {
        assert!(!audit_animals(&[]).has_errors());
        assert!(
            !audit_animals(&[
                animal(1, None, None),
                animal(2, None, None),
                animal(3, Some("001"), Some("ABC")),
                animal(4, Some("002"), Some("DEF")),
            ])
            .has_errors()
        );
    }

    #[test]
    fn duplicate_ids_keep_distinct_source_positions() {
        let animals = [animal(1, Some("a"), None), animal(1, Some("b"), None)];
        let report = audit_animals(&animals);
        assert_eq!(
            report.issues(),
            &[AuditIssue::DuplicateId {
                id: animals[0].id(),
                records: vec![
                    RecordRef {
                        index: 0,
                        animal_id: animals[0].id()
                    },
                    RecordRef {
                        index: 1,
                        animal_id: animals[1].id()
                    }
                ],
            }]
        );
    }

    #[test]
    fn tags_and_uhf_report_all_owners_in_stable_order() {
        let animals = [
            animal(3, Some("002"), Some("X")),
            animal(2, Some("001"), Some("X")),
            animal(1, Some("001"), Some("X")),
            animal(4, Some("002"), None),
        ];
        let report = audit_animals(&animals);
        assert_eq!(report.issues().len(), 3);
        assert!(
            matches!(&report.issues()[0], AuditIssue::DuplicateTag { tag, .. } if tag.as_str() == "001")
        );
        assert!(
            matches!(&report.issues()[1], AuditIssue::DuplicateTag { tag, .. } if tag.as_str() == "002")
        );
        assert!(
            matches!(&report.issues()[2], AuditIssue::DuplicateUhfTag { tag, .. } if tag.as_str() == "X")
        );
        assert_eq!(
            report.issues()[0]
                .records()
                .iter()
                .map(|r| r.index)
                .collect::<Vec<_>>(),
            [2, 1]
        );
        assert_eq!(report.issues()[2].records().len(), 3);
        let mut reversed = animals;
        reversed.reverse();
        let reordered = audit_animals(&reversed);
        for (a, b) in report.issues().iter().zip(reordered.issues()) {
            assert_eq!(
                a.records().iter().map(|r| r.animal_id).collect::<Vec<_>>(),
                b.records().iter().map(|r| r.animal_id).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn normalization_is_shared_and_namespaces_are_separate() {
        assert!(
            audit_animals(&[animal(1, Some(" 001 "), None), animal(2, Some("001"), None)])
                .has_errors()
        );
        assert!(
            !audit_animals(&[
                animal(1, Some("001"), Some("ABC")),
                animal(2, Some("ABC"), Some("001")),
                animal(3, Some("abc"), Some("abc")),
                animal(4, Some("1"), None)
            ])
            .has_errors()
        );
    }

    #[test]
    fn reports_all_conflict_kinds_together() {
        let report = audit_animals(&[
            animal(1, Some("a"), Some("b")),
            animal(1, Some("a"), Some("b")),
        ]);
        assert_eq!(report.issues().len(), 3);
        assert!(report.has_errors());
        assert!(matches!(report.issues()[0], AuditIssue::DuplicateId { .. }));
    }
}

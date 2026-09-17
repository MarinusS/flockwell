use std::collections::HashMap;

use flockwell_domain::Animal;
use flockwell_domain::AnimalId;

#[derive(Debug, PartialEq, Eq)]
struct DuplicateTag {
    tag: String,
    animal_ids: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
struct DuplicateId {
    id: String,
    count: usize,
}

fn find_duplicate_tags(animals: &[Animal]) -> Vec<DuplicateTag> {
    let mut ids_by_tag: HashMap<&str, Vec<AnimalId>> = HashMap::new();

    for animal in animals {
        if let Some(tag) = animal.tag.as_deref() {
            ids_by_tag.entry(tag).or_default().push(animal.id);
        }
    }

    ids_by_tag.retain(|_, ids| ids.len() > 1);

    for ids in ids_by_tag.values_mut() {
        ids.sort_unstable_by_key(|id| id.to_string());
    }

    let mut duplicates: Vec<DuplicateTag> = ids_by_tag
        .into_iter()
        .map(|(tag, ids)| DuplicateTag {
            tag: tag.to_string(),
            animal_ids: ids.into_iter().map(|id| id.to_string()).collect(),
        })
        .collect();

    duplicates.sort_by(|a, b| a.tag.cmp(&b.tag));

    duplicates
}

fn find_duplicate_ids(animals: &[Animal]) -> Vec<DuplicateId> {
    let mut counts_by_id: HashMap<AnimalId, usize> = HashMap::new();

    for animal in animals {
        *counts_by_id.entry(animal.id).or_default() += 1;
    }

    counts_by_id.retain(|_, count| *count > 1);

    let mut duplicates: Vec<DuplicateId> = counts_by_id
        .into_iter()
        .map(|(id, count)| DuplicateId {
            id: id.to_string(),
            count,
        })
        .collect();

    duplicates.sort_by(|a, b| a.id.cmp(&b.id));

    duplicates
}

#[derive(Debug, PartialEq, Eq)]
pub struct AuditReport {
    duplicate_ids: Vec<DuplicateId>,
    duplicate_tags: Vec<DuplicateTag>,
}

impl AuditReport {
    pub fn has_errors(&self) -> bool {
        !self.duplicate_ids.is_empty() || !self.duplicate_tags.is_empty()
    }
}

pub fn audit_animals(animals: &[Animal]) -> AuditReport {
    AuditReport {
        duplicate_ids: find_duplicate_ids(animals),
        duplicate_tags: find_duplicate_tags(animals),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn id(value: &str) -> AnimalId {
        let value = match value {
            "a" => "00000000-0000-0000-0000-000000000001",
            "b" => "00000000-0000-0000-0000-000000000002",
            "c" => "00000000-0000-0000-0000-000000000003",
            "animal-1" => "00000000-0000-0000-0000-000000000011",
            "animal-2" => "00000000-0000-0000-0000-000000000012",
            "animal-3" => "00000000-0000-0000-0000-000000000013",
            "animal-4" => "00000000-0000-0000-0000-000000000014",
            "animal-5" => "00000000-0000-0000-0000-000000000015",
            _ => value,
        };
        value.parse().expect("test IDs are valid UUIDs")
    }

    fn animal(value: &str, tag: Option<&str>) -> Animal {
        Animal::new(id(value), tag)
    }

    #[test]
    fn duplicate_tags_exclude_unique_and_missing_tags() {
        let animals = vec![
            animal("animal-1", Some("00042")),
            animal("animal-2", Some("00042")),
            animal("animal-3", Some("00099")),
            animal("animal-4", None),
            animal("animal-5", None),
        ];

        let expected = vec![DuplicateTag {
            tag: "00042".to_string(),
            animal_ids: vec![id("animal-1").to_string(), id("animal-2").to_string()],
        }];

        assert_eq!(find_duplicate_tags(&animals), expected);
    }

    #[test]
    fn empty_collection_has_no_duplicate_tags() {
        assert!(find_duplicate_tags(&[]).is_empty());
    }

    #[test]
    fn missing_tags_are_not_duplicates() {
        let animals = vec![animal("animal-1", None), animal("animal-2", None)];

        assert!(find_duplicate_tags(&animals).is_empty());
    }

    #[test]
    fn duplicate_tags_include_all_affected_animals() {
        let animals = vec![
            animal("animal-1", Some("00042")),
            animal("animal-2", Some("00042")),
            animal("animal-3", Some("00042")),
        ];

        let expected = vec![DuplicateTag {
            tag: "00042".to_string(),
            animal_ids: vec![
                id("animal-1").to_string(),
                id("animal-2").to_string(),
                id("animal-3").to_string(),
            ],
        }];

        assert_eq!(find_duplicate_tags(&animals), expected);
    }

    #[test]
    fn duplicate_tags_do_not_depend_on_input_order() {
        let mut animals = vec![
            animal("animal-3", Some("00042")),
            animal("animal-1", Some("00042")),
            animal("animal-2", Some("00042")),
        ];

        let expected = vec![DuplicateTag {
            tag: "00042".to_string(),
            animal_ids: vec![
                id("animal-1").to_string(),
                id("animal-2").to_string(),
                id("animal-3").to_string(),
            ],
        }];

        assert_eq!(find_duplicate_tags(&animals), expected);

        animals.reverse();

        assert_eq!(find_duplicate_tags(&animals), expected);
    }

    #[test]
    fn duplicate_tags_are_sorted_by_tag_and_animal_id() {
        let animals = vec![
            animal("animal-4", Some("00099")),
            animal("animal-2", Some("00042")),
            animal("animal-3", Some("00099")),
            animal("animal-1", Some("00042")),
        ];

        let expected = vec![
            DuplicateTag {
                tag: "00042".to_string(),
                animal_ids: vec![id("animal-1").to_string(), id("animal-2").to_string()],
            },
            DuplicateTag {
                tag: "00099".to_string(),
                animal_ids: vec![id("animal-3").to_string(), id("animal-4").to_string()],
            },
        ];

        assert_eq!(find_duplicate_tags(&animals), expected);
    }

    #[test]
    fn duplicate_ids_are_detected_even_when_tags_differ() {
        let animals = vec![
            animal("animal-1", Some("00042")),
            animal("animal-2", Some("00042")),
            animal("animal-2", Some("00099")),
            animal("animal-3", None),
            animal("animal-4", None),
        ];

        let expected = vec![DuplicateId {
            id: id("animal-2").to_string(),
            count: 2,
        }];

        assert_eq!(find_duplicate_ids(&animals), expected);
    }

    #[test]
    fn duplicate_ids_are_counted_and_sorted() {
        let animals = vec![
            animal("b", Some("00042")),
            animal("a", Some("00042")),
            animal("b", None),
            animal("c", None),
            animal("a", Some("00099")),
            animal("b", Some("00100")),
        ];

        let expected = vec![
            DuplicateId {
                id: id("a").to_string(),
                count: 2,
            },
            DuplicateId {
                id: id("b").to_string(),
                count: 3,
            },
        ];

        assert_eq!(find_duplicate_ids(&animals), expected);
    }
    #[test]
    fn report_without_findings_has_no_errors() {
        let report = AuditReport {
            duplicate_ids: vec![],
            duplicate_tags: vec![],
        };

        assert!(!report.has_errors());
    }

    #[test]
    fn report_with_only_duplicate_ids_has_errors() {
        let report = AuditReport {
            duplicate_ids: vec![DuplicateId {
                id: id("animal-1").to_string(),
                count: 2,
            }],
            duplicate_tags: vec![],
        };

        assert!(report.has_errors());
    }

    #[test]
    fn report_with_only_duplicate_tags_has_errors() {
        let report = AuditReport {
            duplicate_ids: vec![],
            duplicate_tags: vec![DuplicateTag {
                tag: "00042".to_string(),
                animal_ids: vec![id("animal-1").to_string(), id("animal-2").to_string()],
            }],
        };

        assert!(report.has_errors());
    }

    #[test]
    fn report_with_both_types_of_findings_has_errors() {
        let report = AuditReport {
            duplicate_ids: vec![DuplicateId {
                id: id("animal-1").to_string(),
                count: 2,
            }],
            duplicate_tags: vec![DuplicateTag {
                tag: "00042".to_string(),
                animal_ids: vec![id("animal-1").to_string(), id("animal-2").to_string()],
            }],
        };

        assert!(report.has_errors());
    }
}

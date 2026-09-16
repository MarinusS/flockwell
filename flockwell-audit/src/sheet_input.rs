use flockwell_audit::Animal;

const ANIMAL_ID_HEADER: &str = "uuid_v7";
const ANIMAL_TAG_HEADER: &str = "tag";

#[derive(Debug, PartialEq, Eq)]
enum HeaderError {
    MissingColumn {
        column_name: String,
    },
    DuplicateColumn {
        column_name: String,
        indices: Vec<usize>,
    },
}

#[derive(Debug, PartialEq, Eq)]
enum RowError {
    MissingAnimalId,
}

#[derive(Debug, PartialEq, Eq)]
struct LocatedRowError {
    row_number: usize,
    error: RowError,
}

#[derive(Debug, PartialEq, Eq)]
struct AnimalColumns {
    id: usize,
    tag: usize,
}

#[derive(Debug, PartialEq, Eq)]
struct ParsedAnimals {
    animals: Vec<Animal>,
    row_errors: Vec<LocatedRowError>,
}

fn require_single_column(column_name: &str, indices: Vec<usize>) -> Result<usize, HeaderError> {
    match indices.as_slice() {
        [] => Err(HeaderError::MissingColumn {
            column_name: column_name.to_string(),
        }),
        [index] => Ok(*index),
        _ => Err(HeaderError::DuplicateColumn {
            column_name: column_name.to_string(),
            indices,
        }),
    }
}

fn find_animal_columns(headers: &[&str]) -> Result<AnimalColumns, Vec<HeaderError>> {
    let mut id_indices = Vec::new();
    let mut tag_indices = Vec::new();

    for (index, header) in headers.iter().enumerate() {
        match header.trim().to_ascii_lowercase().as_str() {
            ANIMAL_ID_HEADER => id_indices.push(index),
            ANIMAL_TAG_HEADER => tag_indices.push(index),
            _ => {}
        }
    }

    let id = require_single_column(ANIMAL_ID_HEADER, id_indices);
    let tag = require_single_column(ANIMAL_TAG_HEADER, tag_indices);

    match (id, tag) {
        (Ok(id), Ok(tag)) => Ok(AnimalColumns { id, tag }),
        (id, tag) => {
            let errors = [id, tag].into_iter().filter_map(Result::err).collect();

            Err(errors)
        }
    }
}

fn parse_animal_row(row: &[&str], columns: &AnimalColumns) -> Result<Animal, RowError> {
    let id = row
        .get(columns.id)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .ok_or(RowError::MissingAnimalId)?;

    let tag = row
        .get(columns.tag)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty());

    Ok(Animal::new(id, tag))
}

fn parse_animal_rows(rows: &[&[&str]]) -> Result<ParsedAnimals, Vec<HeaderError>> {
    let headers = rows.first().copied().unwrap_or(&[]);
    let columns = find_animal_columns(headers)?;

    let mut animals = Vec::new();
    let mut errors = Vec::new();

    for (index, row) in rows.iter().enumerate().skip(1) {
        let row_number = index + 1;

        match parse_animal_row(row, &columns) {
            Ok(animal) => animals.push(animal),
            Err(error) => errors.push(LocatedRowError { row_number, error }),
        }
    }

    Ok(ParsedAnimals {
        animals,
        row_errors: errors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_columns_regardless_of_order() {
        let headers = ["Tag", "Comment", "UUID_v7"];

        assert_eq!(
            find_animal_columns(&headers),
            Ok(AnimalColumns { id: 2, tag: 0 }),
        );
    }

    #[test]
    fn ignores_header_case_and_surrounding_whitespace() {
        let headers = [" UUID_V7 ", "\tTAG "];

        assert_eq!(
            find_animal_columns(&headers),
            Ok(AnimalColumns { id: 0, tag: 1 }),
        );
    }

    #[test]
    fn empty_headers_report_both_missing_columns() {
        assert_eq!(
            find_animal_columns(&[]),
            Err(vec![
                HeaderError::MissingColumn {
                    column_name: ANIMAL_ID_HEADER.to_string(),
                },
                HeaderError::MissingColumn {
                    column_name: ANIMAL_TAG_HEADER.to_string(),
                },
            ]),
        );
    }

    #[test]
    fn reports_missing_id_column() {
        assert_eq!(
            find_animal_columns(&["Tag"]),
            Err(vec![HeaderError::MissingColumn {
                column_name: ANIMAL_ID_HEADER.to_string(),
            }]),
        );
    }

    #[test]
    fn reports_missing_tag_column() {
        assert_eq!(
            find_animal_columns(&["UUID_v7"]),
            Err(vec![HeaderError::MissingColumn {
                column_name: ANIMAL_TAG_HEADER.to_string(),
            }]),
        );
    }

    #[test]
    fn reports_all_duplicate_tag_indices() {
        let headers = ["UUID_v7", "Tag", "TAG", " tag "];

        assert_eq!(
            find_animal_columns(&headers),
            Err(vec![HeaderError::DuplicateColumn {
                column_name: ANIMAL_TAG_HEADER.to_string(),
                indices: vec![1, 2, 3],
            }]),
        );
    }

    #[test]
    fn reports_all_duplicate_id_indices() {
        let headers = ["UUID_v7", "Tag", "uuid_v7", " UUID_V7 "];

        assert_eq!(
            find_animal_columns(&headers),
            Err(vec![HeaderError::DuplicateColumn {
                column_name: ANIMAL_ID_HEADER.to_string(),
                indices: vec![0, 2, 3],
            }]),
        );
    }

    #[test]
    fn reports_missing_and_duplicate_columns_together() {
        let headers = ["Tag", "TAG", "tag"];

        assert_eq!(
            find_animal_columns(&headers),
            Err(vec![
                HeaderError::MissingColumn {
                    column_name: ANIMAL_ID_HEADER.to_string(),
                },
                HeaderError::DuplicateColumn {
                    column_name: ANIMAL_TAG_HEADER.to_string(),
                    indices: vec![0, 1, 2],
                },
            ]),
        );
    }

    #[test]
    fn reports_both_duplicate_columns_in_id_then_tag_order() {
        let headers = ["Tag", "UUID_v7", "TAG", "uuid_v7"];

        assert_eq!(
            find_animal_columns(&headers),
            Err(vec![
                HeaderError::DuplicateColumn {
                    column_name: ANIMAL_ID_HEADER.to_string(),
                    indices: vec![1, 3],
                },
                HeaderError::DuplicateColumn {
                    column_name: ANIMAL_TAG_HEADER.to_string(),
                    indices: vec![0, 2],
                },
            ]),
        );
    }

    #[test]
    fn ignores_unrelated_headers_even_when_duplicated() {
        let headers = ["Comment", "UUID_v7", "Comment", "Tag", ""];

        assert_eq!(
            find_animal_columns(&headers),
            Ok(AnimalColumns { id: 1, tag: 3 }),
        );
    }

    #[test]
    fn trims_animal_id_and_tag() {
        let columns = AnimalColumns { id: 0, tag: 1 };
        let row = [" animal-1 ", "\t00042 "];

        assert_eq!(
            parse_animal_row(&row, &columns),
            Ok(Animal::new("animal-1", Some("00042"))),
        );
    }

    #[test]
    fn preserves_leading_zeros_in_tag() {
        let columns = AnimalColumns { id: 0, tag: 1 };
        let row = ["animal-1", "00042"];

        assert_eq!(
            parse_animal_row(&row, &columns),
            Ok(Animal::new("animal-1", Some("00042"))),
        );
    }

    #[test]
    fn blank_tags_become_none() {
        let columns = AnimalColumns { id: 0, tag: 1 };

        for tag in ["", " ", "\t\n"] {
            let row = ["animal-1", tag];

            assert_eq!(
                parse_animal_row(&row, &columns),
                Ok(Animal::new("animal-1", None)),
                "Failed for tag {tag:?}",
            );
        }
    }

    #[test]
    fn missing_tag_cell_becomes_none() {
        let columns = AnimalColumns { id: 0, tag: 1 };
        let row = ["animal-1"];

        assert_eq!(
            parse_animal_row(&row, &columns),
            Ok(Animal::new("animal-1", None)),
        );
    }

    #[test]
    fn blank_animal_ids_are_rejected() {
        let columns = AnimalColumns { id: 0, tag: 1 };

        for id in ["", " ", "\t\n"] {
            let row = [id, "00042"];

            assert_eq!(
                parse_animal_row(&row, &columns),
                Err(RowError::MissingAnimalId),
                "Failed for ID {id:?}",
            );
        }
    }

    #[test]
    fn missing_animal_id_cell_is_rejected() {
        let columns = AnimalColumns { id: 1, tag: 0 };
        let row = ["00042"];

        assert_eq!(
            parse_animal_row(&row, &columns),
            Err(RowError::MissingAnimalId),
        );
    }

    #[test]
    fn empty_row_is_rejected() {
        let columns = AnimalColumns { id: 0, tag: 1 };

        assert_eq!(
            parse_animal_row(&[], &columns),
            Err(RowError::MissingAnimalId),
        );
    }

    #[test]
    fn parses_using_supplied_column_positions() {
        let columns = AnimalColumns { id: 2, tag: 0 };
        let row = ["00042", "Some comment", "animal-1"];

        assert_eq!(
            parse_animal_row(&row, &columns),
            Ok(Animal::new("animal-1", Some("00042"))),
        );
    }
    #[test]
    fn empty_sheet_reports_missing_headers() {
        assert_eq!(
            parse_animal_rows(&[]),
            Err(vec![
                HeaderError::MissingColumn {
                    column_name: ANIMAL_ID_HEADER.to_string(),
                },
                HeaderError::MissingColumn {
                    column_name: ANIMAL_TAG_HEADER.to_string(),
                },
            ]),
        );
    }

    #[test]
    fn headers_only_produce_no_animals_or_row_errors() {
        let rows: &[&[&str]] = &[&["UUID_v7", "Tag"]];

        assert_eq!(
            parse_animal_rows(rows),
            Ok(ParsedAnimals {
                animals: vec![],
                row_errors: vec![],
            })
        );
    }

    #[test]
    fn parses_data_rows_without_treating_headers_as_an_animal() {
        let rows: &[&[&str]] = &[
            &["UUID_v7", "Tag"],
            &["animal-1", "00042"],
            &["animal-2", "00099"],
        ];

        let expected_animals = vec![
            Animal::new("animal-1", Some("00042")),
            Animal::new("animal-2", Some("00099")),
        ];

        assert_eq!(
            parse_animal_rows(rows),
            Ok(ParsedAnimals {
                animals: expected_animals,
                row_errors: vec![],
            }),
        );
    }

    #[test]
    fn retains_valid_animals_and_reports_invalid_row_location() {
        let rows: &[&[&str]] = &[
            &["UUID_v7", "Tag"],
            &["animal-1", "00042"],
            &["", "00099"],
            &["animal-2", "00100"],
        ];

        let expected_animals = vec![
            Animal::new("animal-1", Some("00042")),
            Animal::new("animal-2", Some("00100")),
        ];

        let expected_errors = vec![LocatedRowError {
            row_number: 3,
            error: RowError::MissingAnimalId,
        }];

        assert_eq!(
            parse_animal_rows(rows),
            Ok(ParsedAnimals {
                animals: expected_animals,
                row_errors: expected_errors,
            }),
        );
    }

    #[test]
    fn collects_multiple_row_errors_without_stopping_early() {
        let rows: &[&[&str]] = &[
            &["UUID_v7", "Tag"],
            &["", "00042"],
            &["animal-1", "00099"],
            &["   ", "00100"],
        ];

        let expected_animals = vec![Animal::new("animal-1", Some("00099"))];

        let expected_errors = vec![
            LocatedRowError {
                row_number: 2,
                error: RowError::MissingAnimalId,
            },
            LocatedRowError {
                row_number: 4,
                error: RowError::MissingAnimalId,
            },
        ];

        assert_eq!(
            parse_animal_rows(rows),
            Ok(ParsedAnimals {
                animals: expected_animals,
                row_errors: expected_errors,
            }),
        );
    }

    #[test]
    fn invalid_headers_prevent_row_parsing() {
        let rows: &[&[&str]] = &[&["UUID_v7", "Tag", "TAG"], &["animal-1", "00042", "00099"]];

        assert_eq!(
            parse_animal_rows(rows),
            Err(vec![HeaderError::DuplicateColumn {
                column_name: ANIMAL_TAG_HEADER.to_string(),
                indices: vec![1, 2],
            }]),
        );
    }
}

use std::collections::HashMap;
use std::str::FromStr;

use flockwell_domain::{Animal, AnimalId, DispositionId, LambingId, LifeStage, Sex};

const ANIMAL_ID_HEADER: &str = "uuid_v7";
const ANIMAL_TAG_HEADER: &str = "tag";
const ANIMAL_TIP_TAG_HEADER: &str = "tip tag";
const ANIMAL_UHF_TAG_HEADER: &str = "uhf tag";
const ANIMAL_UHF_TAG_VISUAL_HEADER: &str = "uhf tag visual";
const ANIMAL_SEX_HEADER: &str = "sex";
const ANIMAL_LIFE_STAGE_OVERRIDE_HEADER: &str = "life stage override";
const ANIMAL_LAMBING_ID_HEADER: &str = "lambing_id";
const ANIMAL_DISPOSITION_ID_HEADER: &str = "disposition_id";

const ANIMAL_HEADERS: [&str; 9] = [
    ANIMAL_ID_HEADER,
    ANIMAL_TAG_HEADER,
    ANIMAL_TIP_TAG_HEADER,
    ANIMAL_UHF_TAG_HEADER,
    ANIMAL_UHF_TAG_VISUAL_HEADER,
    ANIMAL_SEX_HEADER,
    ANIMAL_LIFE_STAGE_OVERRIDE_HEADER,
    ANIMAL_LAMBING_ID_HEADER,
    ANIMAL_DISPOSITION_ID_HEADER,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderError {
    MissingColumn {
        column_name: String,
    },
    DuplicateColumn {
        column_name: String,
        indices: Vec<usize>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum RowError {
    MissingRequiredValue {
        field: &'static str,
    },
    InvalidValue {
        field: &'static str,
        value: String,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct LocatedRowError {
    row_number: usize,
    error: RowError,
}

#[derive(Debug, PartialEq, Eq)]
struct AnimalColumns {
    id: usize,
    tag: usize,
    tip_tag: usize,
    uhf_tag: usize,
    uhf_tag_visual: usize,
    sex: usize,
    life_stage_override: usize,
    lambing_id: usize,
    disposition_id: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParsedAnimals {
    pub(crate) animals: Vec<Animal>,
    pub(crate) row_errors: Vec<LocatedRowError>,
}

fn index_headers(headers: &[String]) -> HashMap<String, Vec<usize>> {
    let mut indices_by_header = HashMap::new();

    for (index, header) in headers.iter().enumerate() {
        indices_by_header
            .entry(header.trim().to_ascii_lowercase())
            .or_insert_with(Vec::new)
            .push(index);
    }

    indices_by_header
}

fn require_single_column(
    headers: &HashMap<String, Vec<usize>>,
    column_name: &str,
) -> Result<usize, HeaderError> {
    match headers.get(column_name).map(Vec::as_slice) {
        None | Some([]) => Err(HeaderError::MissingColumn {
            column_name: column_name.to_owned(),
        }),
        Some([index]) => Ok(*index),
        Some(indices) => Err(HeaderError::DuplicateColumn {
            column_name: column_name.to_owned(),
            indices: indices.to_vec(),
        }),
    }
}

fn find_animal_columns(headers: &[String]) -> Result<AnimalColumns, Vec<HeaderError>> {
    let headers = index_headers(headers);
    let results = [
        require_single_column(&headers, ANIMAL_ID_HEADER),
        require_single_column(&headers, ANIMAL_TAG_HEADER),
        require_single_column(&headers, ANIMAL_TIP_TAG_HEADER),
        require_single_column(&headers, ANIMAL_UHF_TAG_HEADER),
        require_single_column(&headers, ANIMAL_UHF_TAG_VISUAL_HEADER),
        require_single_column(&headers, ANIMAL_SEX_HEADER),
        require_single_column(&headers, ANIMAL_LIFE_STAGE_OVERRIDE_HEADER),
        require_single_column(&headers, ANIMAL_LAMBING_ID_HEADER),
        require_single_column(&headers, ANIMAL_DISPOSITION_ID_HEADER),
    ];

    let errors: Vec<HeaderError> = results
        .iter()
        .filter_map(|result| result.as_ref().err().cloned())
        .collect();

    if !errors.is_empty() {
        return Err(errors);
    }

    let [
        id,
        tag,
        tip_tag,
        uhf_tag,
        uhf_tag_visual,
        sex,
        life_stage_override,
        lambing_id,
        disposition_id,
    ] = results.map(|result| result.expect("column results were checked above"));

    Ok(AnimalColumns {
        id,
        tag,
        tip_tag,
        uhf_tag,
        uhf_tag_visual,
        sex,
        life_stage_override,
        lambing_id,
        disposition_id,
    })
}

fn cell(row: &[String], column: usize) -> Option<&str> {
    row.get(column)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
}

fn parse_required<T>(
    row: &[String],
    column: usize,
    field: &'static str,
) -> Result<T, RowError>
where
    T: FromStr,
{
    let value = cell(row, column).ok_or(RowError::MissingRequiredValue { field })?;

    value.parse().map_err(|_| RowError::InvalidValue {
        field,
        value: value.to_owned(),
    })
}

fn parse_optional<T>(
    row: &[String],
    column: usize,
    field: &'static str,
) -> Result<Option<T>, RowError>
where
    T: FromStr,
{
    let Some(value) = cell(row, column) else {
        return Ok(None);
    };

    value
        .parse()
        .map(Some)
        .map_err(|_| RowError::InvalidValue {
            field,
            value: value.to_owned(),
        })
}

fn parse_sex(value: Option<&str>) -> Result<Sex, RowError> {
    let Some(value) = value else {
        return Ok(Sex::Unknown);
    };

    match value.to_ascii_lowercase().as_str() {
        "m" | "male" => Ok(Sex::Male),
        "f" | "female" => Ok(Sex::Female),
        "u" | "unk" | "unknown" => Ok(Sex::Unknown),
        _ => Err(RowError::InvalidValue {
            field: ANIMAL_SEX_HEADER,
            value: value.to_owned(),
        }),
    }
}

fn parse_life_stage(value: Option<&str>) -> Result<Option<LifeStage>, RowError> {
    let Some(value) = value else {
        return Ok(None);
    };

    match value.to_ascii_lowercase().as_str() {
        "lamb" => Ok(Some(LifeStage::Lamb)),
        "sheep" => Ok(Some(LifeStage::Sheep)),
        _ => Err(RowError::InvalidValue {
            field: ANIMAL_LIFE_STAGE_OVERRIDE_HEADER,
            value: value.to_owned(),
        }),
    }
}

fn parse_animal_row(row: &[String], columns: &AnimalColumns) -> Result<Animal, RowError> {
    let id = parse_required::<AnimalId>(row, columns.id, ANIMAL_ID_HEADER)?;
    let tag = cell(row, columns.tag).map(str::to_owned);
    let tip_tag = cell(row, columns.tip_tag).map(str::to_owned);
    let uhf_tag = cell(row, columns.uhf_tag).map(str::to_owned);
    let uhf_tag_visual = cell(row, columns.uhf_tag_visual).map(str::to_owned);
    let sex = parse_sex(cell(row, columns.sex))?;
    let life_stage_override = parse_life_stage(cell(row, columns.life_stage_override))?;
    let lambing_id = parse_optional::<LambingId>(row, columns.lambing_id, ANIMAL_LAMBING_ID_HEADER)?;
    let disposition_id = parse_optional::<DispositionId>(
        row,
        columns.disposition_id,
        ANIMAL_DISPOSITION_ID_HEADER,
    )?;

    Ok(Animal {
        id,
        tag,
        tip_tag,
        uhf_tag,
        uhf_tag_visual,
        sex,
        life_stage_override,
        lambing_id,
        disposition_id,
    })
}

pub fn parse_animal_rows(rows: &[Vec<String>]) -> Result<ParsedAnimals, Vec<HeaderError>> {
    let headers = rows.first().map(Vec::as_slice).unwrap_or(&[]);
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

    const ANIMAL_1: &str = "00000000-0000-7000-8000-000000000011";
    const ANIMAL_2: &str = "00000000-0000-7000-8000-000000000012";
    const LAMBING_1: &str = "00000000-0000-7000-8000-000000000021";
    const DISPOSITION_1: &str = "00000000-0000-7000-8000-000000000031";

    fn cells(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    fn headers() -> Vec<String> {
        cells(&ANIMAL_HEADERS)
    }

    fn empty_animal(id: &str) -> Animal {
        Animal::new(id.parse().expect("test animal ID is a valid UUIDv7"))
    }

    fn complete_animal() -> Animal {
        let mut animal = empty_animal(ANIMAL_1);
        animal.tag = Some("00042".to_owned());
        animal.tip_tag = Some("TIP-42".to_owned());
        animal.uhf_tag = Some("E2000017221101441890ABCD".to_owned());
        animal.uhf_tag_visual = Some("UHF-42".to_owned());
        animal.sex = Sex::Female;
        animal.life_stage_override = Some(LifeStage::Lamb);
        animal.lambing_id = Some(LAMBING_1.parse().expect("valid lambing UUIDv7"));
        animal.disposition_id = Some(
            DISPOSITION_1
                .parse()
                .expect("valid disposition UUIDv7"),
        );
        animal
    }

    #[test]
    fn finds_all_columns_regardless_of_order_case_and_whitespace() {
        let headers = cells(&[
            " Disposition_ID ",
            "TAG",
            "Sex",
            "UUID_V7",
            "uhf tag visual",
            "LAMBING_ID",
            "Life Stage Override",
            "tip tag",
            "UHF TAG",
        ]);

        assert_eq!(
            find_animal_columns(&headers),
            Ok(AnimalColumns {
                id: 3,
                tag: 1,
                tip_tag: 7,
                uhf_tag: 8,
                uhf_tag_visual: 4,
                sex: 2,
                life_stage_override: 6,
                lambing_id: 5,
                disposition_id: 0,
            }),
        );
    }

    #[test]
    fn empty_headers_report_every_required_column() {
        let expected = ANIMAL_HEADERS
            .iter()
            .map(|column_name| HeaderError::MissingColumn {
                column_name: (*column_name).to_owned(),
            })
            .collect::<Vec<_>>();

        assert_eq!(find_animal_columns(&[]), Err(expected));
    }

    #[test]
    fn duplicate_columns_report_all_indices() {
        let mut headers = headers();
        headers.push(" TAG ".to_owned());

        assert_eq!(
            find_animal_columns(&headers),
            Err(vec![HeaderError::DuplicateColumn {
                column_name: ANIMAL_TAG_HEADER.to_owned(),
                indices: vec![1, 9],
            }]),
        );
    }

    #[test]
    fn parses_every_persisted_animal_field() {
        let columns = find_animal_columns(&headers()).expect("headers are valid");
        let row = cells(&[
            ANIMAL_1,
            " 00042 ",
            "TIP-42",
            "E2000017221101441890ABCD",
            "UHF-42",
            "F",
            "Lamb",
            LAMBING_1,
            DISPOSITION_1,
        ]);

        assert_eq!(parse_animal_row(&row, &columns), Ok(complete_animal()));
    }

    #[test]
    fn blank_optional_values_use_empty_domain_state() {
        let columns = find_animal_columns(&headers()).expect("headers are valid");
        let row = cells(&[ANIMAL_1, "", " ", "", "", "", "", "", ""]);

        assert_eq!(parse_animal_row(&row, &columns), Ok(empty_animal(ANIMAL_1)));
    }

    #[test]
    fn sex_accepts_short_and_long_values_case_insensitively() {
        assert_eq!(parse_sex(Some("M")), Ok(Sex::Male));
        assert_eq!(parse_sex(Some("male")), Ok(Sex::Male));
        assert_eq!(parse_sex(Some("f")), Ok(Sex::Female));
        assert_eq!(parse_sex(Some("FEMALE")), Ok(Sex::Female));
        assert_eq!(parse_sex(None), Ok(Sex::Unknown));
        assert_eq!(parse_sex(Some("unknown")), Ok(Sex::Unknown));
    }

    #[test]
    fn invalid_sex_is_reported_with_field_and_value() {
        assert_eq!(
            parse_sex(Some("ram")),
            Err(RowError::InvalidValue {
                field: ANIMAL_SEX_HEADER,
                value: "ram".to_owned(),
            }),
        );
    }

    #[test]
    fn invalid_life_stage_is_reported_with_field_and_value() {
        assert_eq!(
            parse_life_stage(Some("adult")),
            Err(RowError::InvalidValue {
                field: ANIMAL_LIFE_STAGE_OVERRIDE_HEADER,
                value: "adult".to_owned(),
            }),
        );
    }

    #[test]
    fn blank_animal_id_is_reported_as_missing_required_value() {
        let columns = find_animal_columns(&headers()).expect("headers are valid");
        let row = cells(&["", "", "", "", "", "", "", "", ""]);

        assert_eq!(
            parse_animal_row(&row, &columns),
            Err(RowError::MissingRequiredValue {
                field: ANIMAL_ID_HEADER,
            }),
        );
    }

    #[test]
    fn invalid_uuid_versions_are_rejected_for_persisted_ids() {
        let columns = find_animal_columns(&headers()).expect("headers are valid");
        let non_v7 = "00000000-0000-4000-8000-000000000021";
        let row = cells(&[
            ANIMAL_1, "", "", "", "", "", "", non_v7, "",
        ]);

        assert_eq!(
            parse_animal_row(&row, &columns),
            Err(RowError::InvalidValue {
                field: ANIMAL_LAMBING_ID_HEADER,
                value: non_v7.to_owned(),
            }),
        );
    }

    #[test]
    fn parse_rows_keeps_valid_animals_and_reports_bad_row_location() {
        let rows = vec![
            headers(),
            cells(&[ANIMAL_1, "00042", "", "", "", "M", "", "", ""]),
            cells(&["", "00099", "", "", "", "F", "", "", ""]),
            cells(&[ANIMAL_2, "00100", "", "", "", "F", "", "", ""]),
        ];

        let mut first = empty_animal(ANIMAL_1);
        first.tag = Some("00042".to_owned());
        first.sex = Sex::Male;

        let mut second = empty_animal(ANIMAL_2);
        second.tag = Some("00100".to_owned());
        second.sex = Sex::Female;

        assert_eq!(
            parse_animal_rows(&rows),
            Ok(ParsedAnimals {
                animals: vec![first, second],
                row_errors: vec![LocatedRowError {
                    row_number: 3,
                    error: RowError::MissingRequiredValue {
                        field: ANIMAL_ID_HEADER,
                    },
                }],
            }),
        );
    }

    #[test]
    fn invalid_headers_prevent_row_parsing() {
        let mut invalid_headers = headers();
        invalid_headers.remove(5);
        let rows = vec![invalid_headers, cells(&[ANIMAL_1])];

        assert_eq!(
            parse_animal_rows(&rows),
            Err(vec![HeaderError::MissingColumn {
                column_name: ANIMAL_SEX_HEADER.to_owned(),
            }]),
        );
    }
}

use std::str::FromStr;

use flockwell_domain::{Animal, AnimalId, DispositionId, LambingId, LifeStage, Sex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimalColumn {
    Id,
    Tag,
    Comment,
    TipTag,
    UhfTag,
    UhfTagVisual,
    Sex,
    LifeStageOverride,
    LambingId,
    DispositionId,
}

impl AnimalColumn {
    pub const fn header(self) -> &'static str {
        match self {
            Self::Id => "uuid_v7",
            Self::Tag => "tag",
            Self::Comment => "comment",
            Self::TipTag => "tip tag",
            Self::UhfTag => "uhf tag",
            Self::UhfTagVisual => "uhf tag visual",
            Self::Sex => "sex",
            Self::LifeStageOverride => "life stage override",
            Self::LambingId => "lambing_id",
            Self::DispositionId => "disposition_id",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum HeaderError {
    MissingColumn {
        column: AnimalColumn,
    },
    DuplicateColumn {
        column: AnimalColumn,
        indices: Vec<usize>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum RowError {
    MissingRequiredValue {
        field: AnimalColumn,
    },
    InvalidValue {
        field: AnimalColumn,
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
    comment: usize,
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

fn require_single_column(headers: &[String], column: AnimalColumn) -> Result<usize, HeaderError> {
    let indices = headers
        .iter()
        .enumerate()
        .filter_map(|(index, header)| {
            header
                .trim()
                .eq_ignore_ascii_case(column.header())
                .then_some(index)
        })
        .collect::<Vec<_>>();

    match indices.as_slice() {
        [] => Err(HeaderError::MissingColumn { column }),
        [index] => Ok(*index),
        _ => Err(HeaderError::DuplicateColumn { column, indices }),
    }
}

fn find_animal_columns(headers: &[String]) -> Result<AnimalColumns, HeaderError> {
    Ok(AnimalColumns {
        id: require_single_column(headers, AnimalColumn::Id)?,
        tag: require_single_column(headers, AnimalColumn::Tag)?,
        comment: require_single_column(headers, AnimalColumn::Comment)?,
        tip_tag: require_single_column(headers, AnimalColumn::TipTag)?,
        uhf_tag: require_single_column(headers, AnimalColumn::UhfTag)?,
        uhf_tag_visual: require_single_column(headers, AnimalColumn::UhfTagVisual)?,
        sex: require_single_column(headers, AnimalColumn::Sex)?,
        life_stage_override: require_single_column(headers, AnimalColumn::LifeStageOverride)?,
        lambing_id: require_single_column(headers, AnimalColumn::LambingId)?,
        disposition_id: require_single_column(headers, AnimalColumn::DispositionId)?,
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
    field: AnimalColumn,
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
    field: AnimalColumn,
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
            field: AnimalColumn::Sex,
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
        "sheep" | "adult" => Ok(Some(LifeStage::Adult)),
        _ => Err(RowError::InvalidValue {
            field: AnimalColumn::LifeStageOverride,
            value: value.to_owned(),
        }),
    }
}

fn parse_animal_row(row: &[String], columns: &AnimalColumns) -> Result<Animal, RowError> {
    let id = parse_required::<AnimalId>(row, columns.id, AnimalColumn::Id)?;
    let tag = cell(row, columns.tag).map(str::to_owned);
    let comment = cell(row, columns.comment).map(str::to_owned);
    let tip_tag = cell(row, columns.tip_tag).map(str::to_owned);
    let uhf_tag = cell(row, columns.uhf_tag).map(str::to_owned);
    let uhf_tag_visual = cell(row, columns.uhf_tag_visual).map(str::to_owned);
    let sex = parse_sex(cell(row, columns.sex))?;
    let life_stage_override = parse_life_stage(cell(row, columns.life_stage_override))?;
    let lambing_id = parse_optional::<LambingId>(row, columns.lambing_id, AnimalColumn::LambingId)?;
    let disposition_id = parse_optional::<DispositionId>(
        row,
        columns.disposition_id,
        AnimalColumn::DispositionId,
    )?;

    Ok(Animal {
        id,
        tag,
        comment,
        tip_tag,
        uhf_tag,
        uhf_tag_visual,
        sex,
        life_stage_override,
        lambing_id,
        disposition_id,
    })
}

pub fn parse_animal_rows(rows: &[Vec<String>]) -> Result<ParsedAnimals, HeaderError> {
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
        [
            AnimalColumn::Id,
            AnimalColumn::Tag,
            AnimalColumn::TipTag,
            AnimalColumn::UhfTag,
            AnimalColumn::UhfTagVisual,
            AnimalColumn::Sex,
            AnimalColumn::LifeStageOverride,
            AnimalColumn::LambingId,
            AnimalColumn::DispositionId,
            AnimalColumn::Comment,
        ]
        .map(|column| column.header().to_owned())
        .to_vec()
    }

    fn empty_animal(id: &str) -> Animal {
        Animal::new(id.parse().expect("test animal ID is a valid UUIDv7"))
    }

    fn complete_animal() -> Animal {
        let mut animal = empty_animal(ANIMAL_1);
        animal.tag = Some("00042".to_owned());
        animal.comment = Some("Needs checking".to_owned());
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
            " Comment ",
        ]);

        assert_eq!(
            find_animal_columns(&headers),
            Ok(AnimalColumns {
                id: 3,
                tag: 1,
                comment: 9,
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
    fn missing_column_is_typed() {
        assert_eq!(
            find_animal_columns(&[]),
            Err(HeaderError::MissingColumn {
                column: AnimalColumn::Id,
            }),
        );
    }

    #[test]
    fn duplicate_columns_report_all_indices() {
        let mut headers = headers();
        headers.push(" TAG ".to_owned());

        assert_eq!(
            find_animal_columns(&headers),
            Err(HeaderError::DuplicateColumn {
                column: AnimalColumn::Tag,
                indices: vec![1, 10],
            }),
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
            " Needs checking ",
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
    fn invalid_sex_is_reported_with_typed_field() {
        assert_eq!(
            parse_sex(Some("ram")),
            Err(RowError::InvalidValue {
                field: AnimalColumn::Sex,
                value: "ram".to_owned(),
            }),
        );
    }

    #[test]
    fn invalid_life_stage_is_reported_with_typed_field() {
        assert_eq!(
            parse_life_stage(Some("invalid-stage")),
            Err(RowError::InvalidValue {
                field: AnimalColumn::LifeStageOverride,
                value: "invalid-stage".to_owned(),
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
                field: AnimalColumn::Id,
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
                field: AnimalColumn::LambingId,
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
                        field: AnimalColumn::Id,
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
            Err(HeaderError::MissingColumn {
                column: AnimalColumn::Sex,
            }),
        );
    }

    #[test]
    fn trims_id_and_tag_and_preserves_leading_zeros() {
        let columns = find_animal_columns(&headers()).expect("valid headers");
        let padded_id = format!(" {ANIMAL_1} ");
        let row = cells(&[&padded_id, " 00042 "]);
        let mut expected = empty_animal(ANIMAL_1);
        expected.tag = Some("00042".to_owned());

        assert_eq!(parse_animal_row(&row, &columns), Ok(expected));
    }

    #[test]
    fn malformed_id_preserves_value_and_row_location() {
        let rows = vec![headers(), cells(&["not-a-uuid"]), cells(&[ANIMAL_1])];

        assert_eq!(
            parse_animal_rows(&rows),
            Ok(ParsedAnimals {
                animals: vec![empty_animal(ANIMAL_1)],
                row_errors: vec![LocatedRowError {
                    row_number: 2,
                    error: RowError::InvalidValue {
                        field: AnimalColumn::Id,
                        value: "not-a-uuid".to_owned(),
                    },
                }],
            }),
        );
    }

    #[test]
    fn adult_stage_accepts_existing_sheet_label() {
        for value in ["Sheep", "Adult", "ADULT"] {
            assert_eq!(parse_life_stage(Some(value)), Ok(Some(LifeStage::Adult)));
        }
    }

    #[test]
    fn blank_comment_becomes_none() {
        let columns = find_animal_columns(&headers()).expect("valid headers");
        let mut row = cells(&[ANIMAL_1]);
        row.resize(headers().len(), String::new());
        row[columns.comment] = "   ".to_owned();

        assert_eq!(parse_animal_row(&row, &columns), Ok(empty_animal(ANIMAL_1)));
    }
}

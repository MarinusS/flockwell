use crate::animal::{AnimalId, LambingId};
use time::Date;

pub struct Lambing {
    id: LambingId,
    data: LambingData
}

pub struct LambingData {
    pub dam_id: Option<AnimalId>,
    pub date: Option<Date>,
    pub notes: Option<String>,
}

impl Default for LambingData {
    fn default() -> Self {
        Self {
            dam_id: None,
            date: None,
            notes: None,
        }
    }
}

impl Lambing {
    pub fn new(id: LambingId) -> Self {
        Self { id, data: LambingData::default() }
    }

    pub fn from_data(id: LambingId, data: LambingData) -> Self {
        Self { id, data }
    }

    pub fn id(&self) -> &LambingId {
        &self.id
    }

    pub fn dam_id(&self) -> Option<&AnimalId> {
        self.data.dam_id.as_ref()
    }

    pub fn date(&self) -> Option<&Date> {
        self.data.date.as_ref()
    }

    pub fn notes(&self) -> Option<&str> {
        self.data.notes.as_deref()
    }

    pub fn data(&self) -> &LambingData {
        &self.data
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Month;

    #[test]
    fn lambing_new_sets_default_data() {
        let id: LambingId = "00000000-0000-7000-8000-000000000001"
            .parse()
            .expect("test ID should be a valid UUIDv7");

        let lambing = Lambing::new(id);

        assert_eq!(lambing.id(), &id);
        assert_eq!(lambing.dam_id(), None);
        assert_eq!(lambing.date(), None);
        assert_eq!(lambing.notes(), None);
    }

    #[test]
    fn lambing_from_data_preserves_data() {
        let id: LambingId = "00000000-0000-7000-8000-000000000001"
            .parse()
            .expect("test ID should be a valid UUIDv7");

        let dam_id: AnimalId = "00000000-0000-7000-8000-000000000002"
            .parse()
            .expect("test dam ID should be a valid UUIDv7");

        let date = Date::from_calendar_date(
            2026,
            Month::April,
            4,
        )
        .expect("test date should be valid");

        let notes = "Twins, no assistance";

        let lambing = Lambing::from_data(
            id,
            LambingData {
                dam_id: Some(dam_id),
                date: Some(date),
                notes: Some(notes.to_string()),
            },
        );

        assert_eq!(lambing.id(), &id);
        assert_eq!(lambing.dam_id(), Some(&dam_id));
        assert_eq!(lambing.date(), Some(&date));
        assert_eq!(lambing.notes(), Some(notes));
    }

    #[test]
    fn lambing_allows_unknown_dam_and_date() {
        let id: LambingId = "00000000-0000-7000-8000-000000000001"
            .parse()
            .expect("test ID should be a valid UUIDv7");

        let lambing = Lambing::from_data(
            id,
            LambingData {
                dam_id: None,
                date: None,
                notes: Some("Historical record with incomplete data".to_string()),
            },
        );

        assert_eq!(lambing.dam_id(), None);
        assert_eq!(lambing.date(), None);
        assert_eq!(
            lambing.notes(),
            Some("Historical record with incomplete data")
        );
    }
}
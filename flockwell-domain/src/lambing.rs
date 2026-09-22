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

    pub fn notes(&self) -> Option<&String> {
        self.data.notes.as_ref()
    }

    pub fn data(&self) -> &LambingData {
        &self.data
    }

}
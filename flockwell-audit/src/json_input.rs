use std::error::Error;
use std::path::Path;

use flockwell_domain::{Animal, AnimalId, Sex};
use std::str::FromStr;

#[derive(serde::Deserialize)]
struct AnimalInput {
    id: String,
    tag: Option<String>,
}

pub(crate) fn load_animals(path: &Path) -> Result<Vec<Animal>, Box<dyn Error>> {
    let json = std::fs::read_to_string(path)?;
    let animal_inputs: Vec<AnimalInput> = serde_json::from_str(&json)?;

    let animals = animal_inputs
        .into_iter()
        .map(|input| {
            let id = AnimalId::from_str(&input.id)?;
            Ok(Animal {
                id,
                tag: input.tag,
                tip_tag: None,
                uhf_tag: None,
                uhf_tag_visual: None,
                sex: Sex::Unknown,
                life_stage_override: None,
                lambing_id: None,
                disposition_id: None,
            })
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

    Ok(animals)
}

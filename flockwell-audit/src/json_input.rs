use std::error::Error;
use std::path::Path;

use flockwell_domain::{Animal, AnimalId};
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
            let mut animal = Animal::new(id);
            animal.tag = input.tag;
            Ok(animal)
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

    Ok(animals)
}

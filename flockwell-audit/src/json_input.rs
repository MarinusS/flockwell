use std::error::Error;
use std::path::Path;

use flockwell_audit::Animal;

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
        .map(|input| Animal::new(&input.id, input.tag.as_deref()))
        .collect();

    Ok(animals)
}

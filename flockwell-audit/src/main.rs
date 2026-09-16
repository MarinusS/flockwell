use std::process::ExitCode;

use flockwell_audit::{Animal, audit_animals};

#[derive(serde::Deserialize)]
struct AnimalInput {
    id: String,
    tag: Option<String>,
}

fn main() -> std::process::ExitCode {
    let animals_json = match std::fs::read_to_string("animals.json") {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Could not read animals.json: {err}");
            return ExitCode::from(2);
        }
    };

    let animal_inputs = match serde_json::from_str::<Vec<AnimalInput>>(&animals_json) {
        Ok(animals) => animals,
        Err(err) => {
            eprintln!("Invalid json in animals.json: {err}");
            return ExitCode::from(2);
        }
    };

    println!("Read {} animals", animal_inputs.len());

    let animals: Vec<Animal> = animal_inputs
        .into_iter()
        .map(|animal_input| Animal::new(&animal_input.id, animal_input.tag.as_deref()))
        .collect();

    let audit = audit_animals(&animals);
    if audit.has_errors() {
        println!("{audit:#?}");
        ExitCode::from(1)
    } else {
        println!("Audit passed");
        ExitCode::SUCCESS
    }
}

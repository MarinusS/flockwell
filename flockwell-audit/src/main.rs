use std::path::PathBuf;
use std::process::ExitCode;

use flockwell_audit::{Animal, audit_animals};

#[derive(serde::Deserialize)]
struct AnimalInput {
    id: String,
    tag: Option<String>,
}

fn main() -> ExitCode {
    let input_path = match std::env::args_os().nth(1) {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!("Usage: flockwell-audit <animals.json>");
            return ExitCode::from(2);
        }
    };

    let animals_json = match std::fs::read_to_string(&input_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Could not read {}: {err}", input_path.display());
            return ExitCode::from(2);
        }
    };

    let animal_inputs = match serde_json::from_str::<Vec<AnimalInput>>(&animals_json) {
        Ok(animals) => animals,
        Err(err) => {
            eprintln!("Invalid JSON in {}: {err}", input_path.display());
            return ExitCode::from(2);
        }
    };

    println!("Read {} animals", animal_inputs.len());

    let animals: Vec<Animal> = animal_inputs
        .into_iter()
        .map(|input| Animal::new(&input.id, input.tag.as_deref()))
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

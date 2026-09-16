use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use flockwell_audit::{Animal, audit_animals};

#[derive(serde::Deserialize)]
struct AnimalInput {
    id: String,
    tag: Option<String>,
}

fn load_animals(path: &Path) -> Result<Vec<AnimalInput>, Box<dyn Error>> {
    let json = std::fs::read_to_string(path)?;
    let animals = serde_json::from_str(&json)?;

    Ok(animals)
}

fn main() -> ExitCode {
    let input_path = match std::env::args_os().nth(1) {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!("Usage: flockwell-audit <animals.json>");
            return ExitCode::from(2);
        }
    };

    let animal_inputs = match load_animals(&input_path) {
        Ok(animals) => animals,
        Err(err) => {
            eprintln!("Could not load {}: {err}", input_path.display());
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

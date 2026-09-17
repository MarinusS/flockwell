mod google_sheets;
mod json_input;
mod sheet_input;

use std::path::PathBuf;
use std::process::ExitCode;

use flockwell_audit::audit_animals;
use json_input::load_animals;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let sheets_client = match google_sheets::Client::new().await {
        Ok(client) => {
            println!("Google authentication succeeded");
            client
        }
        Err(err) => {
            eprintln!("Google authentication failed: {err}");
            return ExitCode::from(2);
        }
    };

    let animals_table = match sheets_client.fetch_animal_table().await {
        Ok(animals_table) => {
            println!("Succesfully fetched animals table");
            animals_table
        }
        Err(err) => {
            eprintln!("Failed to fetch animals table: {err}");
            return ExitCode::from(2);
        }
    };

    let animals = match sheet_input::parse_animal_rows(&animals_table) {
        Ok(animals) => {
            println!("Parsed animals: {animals:#?}");
            animals
        }
        Err(_err) => {
            println!("Could parse animals because of header errors: ");
            return ExitCode::from(2);
        }
    };

    let input_path = match std::env::args_os().nth(1) {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!("Usage: flockwell-audit <animals.json>");
            return ExitCode::from(2);
        }
    };

    let animals = match load_animals(&input_path) {
        Ok(animals) => animals,
        Err(err) => {
            eprintln!("Could not load {}: {err}", input_path.display());
            return ExitCode::from(2);
        }
    };

    println!("Read {} animals", animals.len());

    let audit = audit_animals(&animals);

    if audit.has_errors() {
        println!("{audit:#?}");
        ExitCode::from(1)
    } else {
        println!("Audit passed");
        ExitCode::SUCCESS
    }
}

mod google_sheets;
mod json_input;
mod sheet_input;

use std::path::PathBuf;
use std::process::ExitCode;

use flockwell_audit::audit_animals;
use json_input::load_animals;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let token = match google_sheets::authenticate().await {
        Ok(token) => {
            println!("Google authentication succeeded");
            token
        }
        Err(err) => {
            eprintln!("{err}");
            return ExitCode::from(2);
        }
    };

    match google_sheets::fetch_animal_headers(
        &token,
        "1i3cuiIWHw4vefJTLJ5Gq-3opdnsEXuu4J7pavTfBX3M",
    )
    .await
    {
        Ok(headers) => {
            println!("Animal headers: ");
            println!("{headers:#?}");
        }
        Err(err) => {
            eprintln!("{err}");
            return ExitCode::from(2);
        }
    }

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

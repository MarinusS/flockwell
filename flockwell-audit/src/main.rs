use flockwell_audit::{Animal, audit_animals};

fn main() {
    let animals = vec![
        Animal::new(
            "01a0a92d-bb64-72db-a508-760942ff20dd",
            Some("250029228126084"),
        ),
        Animal::new(
            "01a0a92e-9028-76af-8b41-d3a4e9d05a59",
            Some("250029228126084"),
        ),
        Animal::new(
            "01a0a940-f174-745c-a77c-0104bcbd769c",
            Some("250029228125072"),
        ),
        Animal::new("01a0a941-f926-774e-9d98-7876de45ae1c", None),
        Animal::new("01a0a941-f926-774e-9d98-7d2043c0632f", None),
    ];

    println!("{animals:#?}");
    println!();

    let audit = audit_animals(&animals);
    if audit.has_errors() {
        println!("{audit:#?}");
    } else {
        println!("Audit passed")
    }
}

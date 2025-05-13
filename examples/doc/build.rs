use microscpi_doc;

pub fn main() {
    let mut doc = microscpi_doc::Documentation::new();

    if let Err(error) = doc.parse_file("src/lib.rs") {
        println!("cargo::error=Failed to parse lib.rs: {}", error);
    };

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let out_file = std::path::Path::new(&out_dir).join("scpi_commands.json");
    doc.write_to_file(out_file).unwrap();
}

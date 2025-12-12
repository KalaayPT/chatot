use std::collections::hash_map;

pub struct Charmap {
    pub encode_map: std::collections::HashMap<String, u16>,
    pub decode_map: std::collections::HashMap<u16, String>,
}

struct CharmapEntry {
    pub character: String,
    pub code: u16,
    pub kind: String,
}

pub fn read_charmap(path: &std::path::PathBuf) -> Result<Charmap, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let mut encode_map = std::collections::HashMap::new();
    let mut decode_map = std::collections::HashMap::new();

    Ok(Charmap {
        encode_map,
        decode_map,
    })
}
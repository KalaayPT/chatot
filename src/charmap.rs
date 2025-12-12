use std::collections::HashMap;
use quick_xml::Reader;
use quick_xml::events::Event;

pub struct Charmap {
    pub encode_map: HashMap<String, u16>,
    pub decode_map: HashMap<u16, String>,
    pub command_map: HashMap<u16, String>,
}

pub fn read_charmap(path: &std::path::PathBuf) -> Result<Charmap, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let mut encode_map = HashMap::new();
    let mut decode_map = HashMap::new();
    let mut command_map = HashMap::new();

    let mut reader = Reader::from_str(&content);
    reader.config_mut().trim_text(true);
    
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                if e.name().as_ref() == b"entry" {
                    // Parse attributes
                    let mut code_str = String::new();
                    let mut kind = String::new();
                    
                    for attr in e.attributes() {
                        let attr = attr?;
                        match attr.key.as_ref() {
                            b"code" => {
                                code_str = String::from_utf8(attr.value.to_vec())?;
                            }
                            b"kind" => {
                                kind = String::from_utf8(attr.value.to_vec())?;
                            }
                            _ => {}
                        }
                    }
                    
                    // Parse the character content
                    let character = if let Ok(Event::Text(t)) = reader.read_event_into(&mut buf) {
                        String::from_utf8(t.to_vec())?
                    } else {
                        String::new()
                    };
                    
                    // Convert hex code to u16
                    let code = u16::from_str_radix(code_str.trim_start_matches("0x"), 16)?;

                    // Populate maps based on kind
                    match kind.as_str() {
                        "char" => {
                            encode_map.insert(character.clone(), code);
                            decode_map.insert(code, character);
                        },
                        "alias" => {
                            encode_map.insert(character.clone(), code);
                        },
                        "command" => {
                            // For commands, we only need the command_map
                            command_map.insert(code, character);
                        },
                        _ => {return Err("Unknown kind attribute in charmap entry".into());}
                    }


                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Error parsing XML at position {}: {:?}", reader.buffer_position(), e).into()),
            _ => {}
        }
        buf.clear();
    }

    Ok(Charmap {
        encode_map,
        decode_map,
        command_map,
    })
}
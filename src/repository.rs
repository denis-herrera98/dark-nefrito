// use serde::Deserialize;
use std::collections::HashMap;
use serde_json::Value;
use std::fs;
use std::io;
use std::path::Path;

pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String, io::Error> {
    fs::read_to_string(path)
}

pub fn string_to_hasmap(json_string: &str) -> HashMap<String, Value> {
    let parsed_json: Value = serde_json::from_str(json_string).expect("Error al analizar el JSON");
    let hashmap: HashMap<String, Value> = serde_json::from_value(parsed_json).expect("Error al convertir a HashMap");
    return hashmap
}

use crate::Configs;
use std::collections::HashMap;
use std::fs;
use std::io::Write;

static CONFIG_FILENAME: &str = "RUSTVAIL-autoload.cfg";

//parse extremely simple config file
pub fn parse(filename: &str) -> Configs {
    //read file
    let contents = match fs::read_to_string(filename) {
        Ok(contents) => contents,
        Err(err) => {
            println!("Failed to read config file: {:?}", err);
            return Configs::default();
        }
    };
    let mut hash_map = HashMap::new();

    //make a hashmap from key=value pairs
    for line in contents.lines() {
        if let Some((key, value)) = line.split_once('=') {
            hash_map.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    //there is a lot of duplicated code, probably can be cleaned up using macros but im not smart enough to do that
    let mut configs = Configs::default();

    if let Some(value) = hash_map.get("enabled") {
        if let Ok(value) = value.parse() {
            configs.enabled = value;
        }
    }

    if let Some(value) = hash_map.get("hints_enabled") {
        if let Ok(value) = value.parse() {
            configs.hints_enabled = value;
        }
    }

    if let Some(value) = hash_map.get("ip_address") {
        if let Ok(value) = value.parse() {
            configs.ip_address = value;
        }
    }

    if let Some(value) = hash_map.get("height") {
        if let Ok(value) = value.parse() {
            configs.height = value;
        }
    }

    if let Some(value) = hash_map.get("height_offset") {
        if let Ok(value) = value.parse() {
            configs.height_offset = value;
        }
    }

    if let Some(value) = hash_map.get("locked_to_headset") {
        if let Ok(value) = value.parse() {
            configs.locked_to_headset = value;
        }
    }

    if let Some(value) = hash_map.get("hip_enabled") {
        if let Ok(value) = value.parse() {
            configs.hip_enabled = value;
        }
    }

    if let Some(value) = hash_map.get("left_foot_enabled") {
        if let Ok(value) = value.parse() {
            configs.left_foot_enabled = value;
        }
    }

    if let Some(value) = hash_map.get("right_foot_enabled") {
        if let Ok(value) = value.parse() {
            configs.right_foot_enabled = value;
        }
    }

    return configs;
}

//TODO: error handling
pub fn save(filename: &str, thread_data: &Configs) {
    let mut file = fs::File::create(filename).expect("Failed to create config file");

    //make a hashmap of key=value pairs TODO: use thread_data struct to define keys
    let mut config = HashMap::new();
    config.insert("enabled".to_string(), thread_data.enabled.to_string());
    config.insert("height".to_string(), thread_data.height.to_string());
    config.insert(
        "height_offset".to_string(),
        thread_data.height_offset.to_string(),
    );
    config.insert(
        "hip_enabled".to_string(),
        thread_data.hip_enabled.to_string(),
    );
    config.insert(
        "left_foot_enabled".to_string(),
        thread_data.left_foot_enabled.to_string(),
    );
    config.insert(
        "right_foot_enabled".to_string(),
        thread_data.right_foot_enabled.to_string(),
    );
    config.insert(
        "locked_to_headset".to_string(),
        thread_data.locked_to_headset.to_string(),
    );
    config.insert("ip_address".to_string(), thread_data.ip_address.to_string());

    //write the hashmap to the file
    writeln!(
        file,
        "#RustVail server properties ^^\n#Delete this if you are having issues"
    )
    .expect("Failed to write to config file");
    for (key, value) in config {
        writeln!(file, "{}={}", key, value).expect("Failed to write to config file");
    }
}

pub fn delete_config() {
    delete_named_config(CONFIG_FILENAME);
}

pub fn delete_named_config(name: &str) {
    let result = std::fs::remove_file(name);
    match result {
        Ok(_) => {
            // file removed
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // ignore if file not found
        }
        Err(e) => {
            panic!("Failed to remove file: {:?}", e);
        }
    }
}



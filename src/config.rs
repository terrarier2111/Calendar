use std::{
    fs,
    hash::{Hash, Hasher},
};

use fnv::FnvHasher;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    calendars: Vec<Calendar>,
}

impl Config {
    pub fn load() -> Self {
        let mut path = dirs::config_dir().unwrap();
        path.push("HCal");
        let dir_path = path.clone();
        path.set_file_name("config.json");
        if !dir_path.exists() {
            fs::create_dir_all(dir_path).unwrap();
            fs::write(
                &path,
                serde_json::to_string_pretty(&Config { calendars: vec![] }).unwrap(),
            )
            .unwrap();
        }
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap()
    }

    pub fn save(&self) {
        let mut path = dirs::config_dir().unwrap();
        path.push("HCal");
        if !path.exists() {
            fs::create_dir_all(&path).unwrap();
        }
        path.set_file_name("config.json");
        fs::write(path, serde_json::to_string_pretty(self).unwrap()).unwrap();
    }
}

pub fn get_last_update(src: &str) -> Option<u128> {
    let mut path = dirs::config_dir().unwrap();
    path.push("HCal");
    if !path.exists() {
        return None;
    }
    path.set_file_name({
        let mut hasher = FnvHasher::default();
        src.hash(&mut hasher);
        hasher.finish().to_string()
    });
    if !path.exists() {
        return None;
    }
    let raw = fs::read_to_string(path).unwrap();
    Some(raw.parse::<u128>().unwrap())
}

pub fn store_last_update(src: &str, last_update: u128) {
    let mut path = dirs::config_dir().unwrap();
    path.push("HCal");
    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    path.set_file_name({
        let mut hasher = FnvHasher::default();
        src.hash(&mut hasher);
        hasher.finish().to_string()
    });
    fs::write(path, last_update.to_le_bytes()).unwrap();
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Calendar {
    src: CalendarSrc,
    name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum CalendarSrc {
    Web {
        src: String,
    },
    Local {
        events: Vec<CalEvent>,
        color: ConfigColor,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CalEvent {
    pub start: u64,
    pub finish: u64,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConfigColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

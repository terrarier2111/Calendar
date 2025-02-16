use std::{
    borrow::Cow,
    fs,
    hash::{Hash, Hasher},
};

use fnv::FnvHasher;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub calendars: Vec<Calendar>,
}

impl Config {
    pub fn load() -> Self {
        let mut path = dirs::config_dir().unwrap();
        path.push("HCal");
        let dir_path = path.clone();
        path.push("config.json");
        if !dir_path.exists() {
            fs::create_dir_all(dir_path).unwrap();
            fs::write(
                &path,
                serde_json::to_string_pretty(&Config {
                    calendars: vec![Calendar {
                        src: CalendarSrc::Web {
                            src: "".to_string(),
                        },
                        name: "test".to_string(),
                        color: ConfigColor {
                            r: 255,
                            g: 0,
                            b: 0,
                            a: 255,
                        },
                    }],
                })
                .unwrap(),
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
    pub name: String,
    pub color: ConfigColor,
}

impl Calendar {
    pub fn get_events(&self, rt: &Runtime) -> Cow<Vec<CalEvent>> {
        match &self.src {
            CalendarSrc::Web { src } => {
                println!("fetch events");
                Cow::Owned(
                    rt.block_on(web_ical::Calendar::new(src))
                        .unwrap()
                        .events
                        .into_iter()
                        .map(|event| CalEvent {
                            start: event
                                .dtstart
                                .as_ref()
                                .map(|dt| dt.timestamp_millis() as u64)
                                .unwrap_or(0),
                            finish: event
                                .dtend
                                .as_ref()
                                .map(|dt| dt.timestamp_millis() as u64)
                                .unwrap_or(0),
                            name: event.summary.unwrap_or_default(),
                            location: event.location.unwrap_or_default(),
                            repeat: event.repeat.map(|rep| Repeat {
                                freq: rep.freq,
                                until: rep.until.map(|val| val.timestamp_millis() as u64),
                            }),
                        })
                        .collect::<Vec<CalEvent>>(),
                )
            }
            CalendarSrc::Local { events } => Cow::Borrowed(events),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum CalendarSrc {
    Web { src: String },
    Local { events: Vec<CalEvent> },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CalEvent {
    pub start: u64,
    pub finish: u64,
    pub name: String,
    pub location: String,
    pub repeat: Option<Repeat>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Repeat {
    pub freq: String,
    pub until: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConfigColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

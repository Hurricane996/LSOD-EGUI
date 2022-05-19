use livesplit_core::{HotkeyConfig, Layout, Run, Segment};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt::Display,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use crate::utils::{load_layout_from_file, load_splits_from_file};

#[derive(Debug)]
struct NoConfigDirError;

impl Display for NoConfigDirError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Configuration Dir not found")
    }
}

impl Error for NoConfigDirError {}

#[derive(Serialize, Deserialize)]
pub struct Configuration {
    #[serde(default)]
    pub splits_path: Option<PathBuf>,
    #[serde(default)]
    pub layout_path: Option<PathBuf>,
    #[serde(default)]
    pub hotkey_config: HotkeyConfig,

    #[serde(default = "default_size")]
    pub size: (u32, u32),
}

fn default_size() -> (u32, u32) {
    (320, 240)
}
impl Default for Configuration {
    fn default() -> Self {
        Self {
            splits_path: None,
            layout_path: None,
            hotkey_config: Default::default(),
            size: default_size(),
        }
    }
}

impl Configuration {
    fn get() -> Option<Self> {
        let file = File::open(Configuration::get_path()?).ok()?;

        serde_json::de::from_reader(BufReader::new(file))
            .map_err(|e| println!("Failed to load config, got error: {e}"))
            .ok()
            .map(|mut config: Configuration| {
                config.validate();
                config
            })
    }

    pub fn get_path() -> Option<PathBuf> {
        let mut location = dirs::config_dir()?;
        location.push("lsod_config.json");
        Some(location)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(Configuration::get_path().ok_or(NoConfigDirError)?)?;

        serde_json::ser::to_writer(BufWriter::new(file), self).map_err(Into::into)
    }

    pub fn get_or_default() -> Self {
        Configuration::get().unwrap_or_default()
    }

    pub fn layout_or_default(&self) -> Layout {
        match &self.layout_path {
            Some(path) => load_layout_from_file(path).unwrap_or_else(|_| Layout::default_layout()),
            None => Layout::default_layout(),
        }
    }
    pub fn run_or_default(&self) -> Run {
        match &self.splits_path {
            Some(path) => load_splits_from_file(path).unwrap_or_else(|e| {
                eprintln!("Failed to load splits, got error {e}");
                Self::default_run()
            }),
            None => Self::default_run(),
        }
    }

    fn default_run() -> Run {
        let mut run = Run::new();

        run.set_game_name("untitled");
        run.set_category_name("untitled");
        run.push_segment(Segment::new("Time"));

        run
    }

    fn validate(&mut self) {
        if self.size.0 == 0 {
            self.size.0 = 1;
        }

        if self.size.1 == 0 {
            self.size.1 = 1;
        }
    }
}

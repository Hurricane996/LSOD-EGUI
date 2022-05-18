use anyhow::Result;
use livesplit_core::{
    layout::{self, LayoutSettings},
    run::parser::composite,
    Layout, Run,
};
use std::{
    fs::File,
    io::{BufReader, Seek, SeekFrom},
    path::PathBuf,
};

pub fn load_layout_from_file(path: &PathBuf) -> Result<Layout> {
    let mut file = BufReader::new(File::open(path)?);
    if let Ok(settings) = LayoutSettings::from_json(&mut file) {
        return Ok(Layout::from_settings(settings));
    }
    file.seek(SeekFrom::Start(0))?;
    layout::parser::parse(file).map_err(|e| e.into())
}

pub fn load_splits_from_file(path: &PathBuf) -> Result<Run> {
    let file = BufReader::new(File::open(&path)?);
    let mut run = composite::parse(file, Some(path.clone()), true)?.run;
    run.fix_splits();
    Ok(run)
}

use std::error::Error;
use std::fs::File;
use std::io::{Error as IOError, ErrorKind, Result as IOResult, Write};
use std::path::Path;
use std::{fs, io::BufReader};

use serde::de::Deserializer;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
struct StatField {
    stat: String,
    translation: String,
}

#[derive(Serialize, Debug)]
struct Stats {
    game_version: String,
    stats: Vec<StatField>,
}

impl<'de> Deserialize<'de> for StatField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            stat: String,
            translation: String,
        }

        let helper = Helper::deserialize(deserializer)?;

        let stat = translate_stat(helper.stat);
        let translation = helper.translation;

        Ok(StatField { stat, translation })
    }
}

fn main() {
    let dir_path = Path::new("./assets");
    let stats = get_stats(dir_path).expect("Failed to parse stat files from the directory");
    write_files(stats).expect("Failed to write stats to files");
}

fn get_stats(dir: &Path) -> IOResult<Vec<Stats>> {
    let dir = fs::read_dir(dir)?;

    let mut stats: Vec<Stats> = Vec::new();

    for entry in dir {
        let entry = entry?;
        let file = File::open(&entry.path())?;
        let file_name = entry.file_name();

        let Some(file_name) = file_name.to_str() else {
            return Err(IOError::new(ErrorKind::NotFound, "Cannot find file name"));
        };

        let game_version = &file_name[6..=11];

        let reader = BufReader::new(file);

        let parsed: Vec<StatField> = serde_json::from_reader(reader)?;

        stats.push(Stats {
            game_version: game_version.to_string(),
            stats: parsed,
        });
    }

    Ok(stats)
}

fn write_files(stats: Vec<Stats>) -> Result<(), Box<dyn Error>> {
    let outdir_path = Path::new("./out");

    if outdir_path.exists() {
        fs::remove_dir_all(outdir_path)?;
    }

    fs::create_dir(outdir_path)?;

    for stat in stats {
        let file_name = format!("scoreboards_{}.json", stat.game_version);
        let path = outdir_path.join(file_name);
        let mut file = File::create(&path)?;

        serde_json::to_writer_pretty(&mut file, &stat.stats)?;
        file.write_all(b"\n")?; // Add a newline at the end of the file
        file.flush()?; // Flush the file buffer to ensure it's written to disk
    }

    Ok(())
}

fn translate_stat(name: String) -> String {
    let parts: Vec<&str> = name.split(":").to_owned().collect();

    format!("{}-{}", shorten_scoreboard_type(parts[0]), &parts[1][10..])
}

fn shorten_scoreboard_type(s: &str) -> String {
    match s {
        "minecraft.mined" => "m",
        "minecraft.used" => "u",
        "minecraft.crafted" => "c",
        "minecraft.broken" => "b",
        "minecraft.picked_up" => "p",
        "minecraft.dropped" => "d",
        "minecraft.killed" => "k",
        "minecraft.killed_by" => "kb",
        "minecraft.custom" => "z",
        _ => panic!("Found unexpected stat: {}", s),
    }
    .to_string()
}

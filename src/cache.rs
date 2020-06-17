/*
 * Meteoschweiz: Render meteo data from meteoschweiz.admin.ch
 * Copyright (C) 2020  Tibor Schneider
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::forecast::Forecast;
use crate::Result;
use bincode;
use std::fs;
use std::path::PathBuf;

static LAST_JSON_FILENAME: &str = "last_forecast_url";
static LAST_FORECAST_DATA: &str = "last_forecast.bin";

pub fn check_cache(cache_folder: &str) -> Result<Option<String>> {
    let mut p = PathBuf::from(cache_folder);
    // check if path exists
    if !p.is_dir() {
        fs::create_dir_all(&p)?;
    }
    // check if last forecast exists
    p.push(LAST_JSON_FILENAME);
    if !p.is_file() {
        Ok(None)
    } else {
        Ok(Some(fs::read_to_string(p)?))
    }
}

pub fn clear_cache(cache_folder: &str) -> Result<()> {
    let p = PathBuf::from(cache_folder);
    // remove all .pdf files in this directory
    for file in fs::read_dir(&p)? {
        let file = file?;
        let path = file.path();
        if path.is_file() {
            if path.extension() == Some(std::ffi::OsStr::new("pdf")) {
                fs::remove_file(path)?;
            }
        }
    }

    Ok(())
}

pub fn set_current_json_url(cache_folder: &str, json_url: &str) -> Result<()> {
    let mut p = PathBuf::from(cache_folder);
    p.push(LAST_JSON_FILENAME);
    fs::write(p, json_url)?;
    Ok(())
}

pub fn cache_forecast(cache_folder: &str, forecast: &Forecast) -> Result<()> {
    let mut p = PathBuf::from(cache_folder);
    p.push(LAST_FORECAST_DATA);
    fs::write(p, bincode::serialize(forecast)?)?;
    Ok(())
}

pub fn get_cached_forecast(cache_folder: &str) -> Result<Forecast> {
    let mut p = PathBuf::from(cache_folder);
    p.push(LAST_FORECAST_DATA);
    Ok(bincode::deserialize(&fs::read(p)?)?)
}

pub fn is_pdf_cached(cache_folder: &str, day_idx: usize, show_long: bool) -> bool {
    let mut p = PathBuf::from(cache_folder);
    if show_long {
        p.push("long.pdf");
    } else {
        p.push(format!("{}.pdf", day_idx));
    }
    p.is_file()
}

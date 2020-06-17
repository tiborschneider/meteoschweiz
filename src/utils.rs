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

use crate::Result;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

static NUM_ICONS: usize = 35;
static ICON_URL_BASE: &str = "https://www.meteoschweiz.admin.ch/etc/designs/meteoswiss/assets/images/icons/meteo/weather-symbols/";

pub fn fetch_icons(icon_folder: &str) -> Result<()> {
    // export the old current path
    let old_path = env::current_dir()?;
    // check if icon folder already exists
    let mut p = PathBuf::from(icon_folder);
    // if it is not a directory, recursively generate it
    if !p.is_dir() {
        fs::create_dir_all(&p)?;
    }
    // change directory
    env::set_current_dir(&p)?;

    // loop over all
    for i in 0..NUM_ICONS {
        let icon_idx = i + 1;
        p.push(format!("{}.pdf", icon_idx));
        if !p.is_file() {
            println!("Downloading and converting: {:?}", p);
            // downoad the image
            Command::new("wget")
                .arg(format!("{}{}.svg", ICON_URL_BASE, icon_idx))
                .output()?;

            // convert the image to a pdf
            Command::new("inkscape")
                .arg(format!("{}.svg", icon_idx))
                .arg(format!("--export-filename={}.pdf", icon_idx))
                .arg("--export-type=pdf")
                .output()?;
        }
        p.pop();
    }

    // change directory back
    env::set_current_dir(old_path)?;

    Ok(())
}

pub fn generate_template(template_file: &str, template_long_file: &str) -> Result<()> {
    let p_short = PathBuf::from(template_file);
    // check if the template file already exists
    if !p_short.is_file() {
        // check if parent dir exists
        if let Some(parent) = p_short.parent() {
            if !parent.is_dir() {
                fs::create_dir_all(parent)?;
            }
        }
        // generate a new file with the desired content
        let template_file = include_bytes!("template.tex.tera");
        let template_string = std::str::from_utf8(template_file)?;
        fs::write(p_short, template_string)?;
    }

    let p_long = PathBuf::from(template_long_file);
    // check if the template file already exists
    if !p_long.is_file() {
        // check if parent dir exists
        if let Some(parent) = p_long.parent() {
            if !parent.is_dir() {
                fs::create_dir_all(parent)?;
            }
        }
        // generate a new file with the desired content
        let template_file = include_bytes!("template_long.tex.tera");
        let template_string = std::str::from_utf8(template_file)?;
        fs::write(p_long, template_string)?;
    }

    Ok(())
}

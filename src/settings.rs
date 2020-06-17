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

use config::{Config, File};
use dirs::{cache_dir, config_dir};
use lazy_static::lazy_static;
use serde::Deserialize;
use shellexpand;
use std::fs;

lazy_static! {
    static ref CONFIG_FILE: String = {
        let mut p = config_dir().unwrap();
        p.push("meteoschweiz");
        p.push("config.toml");
        p.into_os_string().into_string().unwrap()
    };
    pub static ref SETTINGS: Settings = {
        generate_missing_config_file();
        let mut settings = Config::default();
        settings
            .set_default("icon_path", {
                let mut p = config_dir().unwrap();
                p.push("meteoschweiz");
                p.push("icons");
                p.into_os_string().into_string().unwrap()
            })
            .unwrap()
            .set_default("location_plz", 8001)
            .unwrap()
            .set_default("template_file", {
                let mut p = config_dir().unwrap();
                p.push("meteoschweiz");
                p.push("template.tex.tera");
                p.into_os_string().into_string().unwrap()
            })
            .unwrap()
            .set_default("template_long_file", {
                let mut p = config_dir().unwrap();
                p.push("meteoschweiz");
                p.push("template_long.tex.tera");
                p.into_os_string().into_string().unwrap()
            })
            .unwrap()
            .set_default("cache_folder", {
                let mut p = cache_dir().unwrap();
                p.push("meteoschweiz");
                p.into_os_string().into_string().unwrap()
            })
            .unwrap()
            .set_default("pdf_viewer", "zathura")
            .unwrap()
            .set_default("pdf_viewer_args", vec!["--fork"])
            .unwrap();
        settings.merge(File::with_name(&CONFIG_FILE)).unwrap();
        settings.try_into::<Settings>().unwrap().expand()
    };
}

fn generate_missing_config_file() {
    let mut p = config_dir().unwrap();
    p.push("meteoschweiz");
    p.push("config.toml");
    if !p.is_file() {
        if !p.parent().unwrap().is_dir() {
            // create directory recursive
            fs::create_dir_all(p.parent().unwrap()).unwrap();
        }
        fs::write(
            p,
            std::str::from_utf8(include_bytes!("default_config.toml")).unwrap(),
        )
        .unwrap();
    }
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub icon_path: String,
    pub location_plz: u32,
    pub template_file: String,
    pub template_long_file: String,
    pub cache_folder: String,
    pub pdf_viewer: String,
    pub pdf_viewer_args: Vec<String>,
}

impl Settings {
    fn expand(mut self) -> Self {
        self.icon_path = shellexpand::full(&self.icon_path).unwrap().into_owned();
        self.template_file = shellexpand::full(&self.template_file).unwrap().into_owned();
        self.template_long_file = shellexpand::full(&self.template_long_file)
            .unwrap()
            .into_owned();
        self.cache_folder = shellexpand::full(&self.cache_folder).unwrap().into_owned();
        self
    }
}

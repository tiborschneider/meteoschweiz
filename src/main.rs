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

mod cache;
mod errors;
mod forecast;
mod settings;
mod utils;

use settings::SETTINGS as CFG;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use clap::{App, Arg};
use tera;

pub use errors::{Error, Result};

fn main() -> Result<()> {
    let matches = App::new("Meteo Schweiz")
        .version("0.1")
        .author("Tibor Schneider")
        .about("Render and show meteo schweiz")
        .arg(
            Arg::with_name("day")
                .short("d")
                .long("day")
                .value_name("DAY")
                .takes_value(true)
                .default_value("0")
                .help("Show the specified day"),
        )
        .arg(
            Arg::with_name("long")
                .short("l")
                .long("long")
                .takes_value(false)
                .help("Show all 7 days"),
        )
        .get_matches();

    let day_idx: usize = matches.value_of("day").unwrap_or("0").parse().unwrap();
    let show_long: bool = matches.is_present("long");

    // download all missing icons
    utils::fetch_icons(&CFG.icon_path)?;

    // generate template if it does not yet exist
    utils::generate_template(&CFG.template_file, &CFG.template_long_file)?;

    // check last cached json name
    println!("Extracting json url...");
    let cached_json_url = cache::check_cache(&CFG.cache_folder)?;
    let new_json_url = forecast::get_forecast_chart_json_url(CFG.location_plz)?;

    if cached_json_url.unwrap_or("".to_string()) == new_json_url {
        if !cache::is_pdf_cached(&CFG.cache_folder, day_idx, show_long) {
            // get the forecast
            let fc = match cache::get_cached_forecast(&CFG.cache_folder) {
                Ok(fc) => {
                    println!("Using cached forecast...");
                    fc
                }
                Err(e) => {
                    eprintln!(
                        "Cannot read forecast from cache: {:?}\nPulling from meteoschweiz.ch...",
                        e
                    );
                    // fetch it
                    let fc = forecast::fetch_forecast(&new_json_url, &CFG.icon_path)?;
                    // serialize the forecast to a json file in the cache for future access
                    cache::cache_forecast(&CFG.cache_folder, &fc)?;
                    fc
                }
            };
            // generate the latex file and run pdflatex
            if show_long {
                apply_template_long(&fc, &CFG.template_long_file, &CFG.cache_folder)?;
            } else {
                apply_template(&fc[day_idx], &CFG.template_file, &CFG.cache_folder, day_idx)?;
            }
        }
    } else {
        // clear the cache
        cache::clear_cache(&CFG.cache_folder)?;
        // update the json url file
        cache::set_current_json_url(&CFG.cache_folder, &new_json_url)?;
        // fetch and parse the forecast
        println!("Fetching new forecast...");
        let fc = forecast::fetch_forecast(&new_json_url, &CFG.icon_path)?;
        // serialize the forecast to a json file in the cache for future access
        cache::cache_forecast(&CFG.cache_folder, &fc)?;
        // generate the latex file and run pdflatex
        if show_long {
            apply_template_long(&fc, &CFG.template_long_file, &CFG.cache_folder)?;
        } else {
            apply_template(&fc[day_idx], &CFG.template_file, &CFG.cache_folder, day_idx)?;
        }
    }

    // display the requested forecast
    show_forecast(&CFG.cache_folder, day_idx, show_long)?;

    Ok(())
}

fn apply_template_long(
    fc: &forecast::Forecast,
    template_filename: &str,
    cache_folder: &str,
) -> Result<()> {
    println!("Generating forecast pdf...");

    // read template file
    let template = fs::read_to_string(template_filename)?;

    // prepare target tex filename
    let mut target_file = PathBuf::from(&cache_folder);
    target_file.push("long.tex");

    // generate context and tex file
    let fc_long = forecast::ForecastLong::from(fc);
    let mut ctx = tera::Context::new();
    ctx.insert("forecast_long", &fc_long);
    let tex_file = tera::Tera::one_off(&template, &ctx, false)?;

    // write back
    fs::write(&target_file, &tex_file)?;

    // change directory into cache folder
    let old_dir = env::current_dir()?;
    env::set_current_dir(&cache_folder)?;

    // compile pdflatex
    let status = Command::new("pdflatex")
        .arg(&target_file)
        .arg("--output-format=pdf")
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .status()?;

    env::set_current_dir(old_dir)?;

    if !status.success() {
        Err(Error::PdflatexError(status.code()))
    } else {
        Ok(())
    }
}

fn apply_template(
    day: &forecast::ForecastDay,
    template_filename: &str,
    cache_folder: &str,
    forecast_idx: usize,
) -> Result<()> {
    println!("Generating forecast pdf...");

    // read template file
    let template = fs::read_to_string(template_filename)?;

    // prepare target tex filename
    let mut target_file = PathBuf::from(&cache_folder);
    target_file.push(format!("{}.tex", forecast_idx));

    // generate context and tex file
    let mut ctx = tera::Context::new();
    ctx.insert("forecast_day", day);
    let tex_file = tera::Tera::one_off(&template, &ctx, false)?;

    // write back
    fs::write(&target_file, &tex_file)?;

    // change directory into cache folder
    let old_dir = env::current_dir()?;
    env::set_current_dir(&cache_folder)?;

    // compile pdflatex
    let status = Command::new("pdflatex")
        .arg(&target_file)
        .arg("--output-format=pdf")
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .status()?;

    env::set_current_dir(old_dir)?;

    if !status.success() {
        Err(Error::PdflatexError(status.code()))
    } else {
        Ok(())
    }
}

fn show_forecast(cache_folder: &str, forecast_idx: usize, show_long: bool) -> Result<()> {
    Command::new(&CFG.pdf_viewer)
        .args(&CFG.pdf_viewer_args)
        .arg(match show_long {
            false => format!("{}/{}.pdf", cache_folder, forecast_idx),
            true => format!("{}/long.pdf", cache_folder),
        })
        .output()?;
    Ok(())
}

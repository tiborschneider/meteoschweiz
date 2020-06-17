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

use crate::{Error, Result};

use chrono::{self, TimeZone, Timelike};
use isahc::prelude::*;
use itertools::zip_eq as zip;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json;

const URL_WITH_JSON_LINK: &str = "https://www.meteoschweiz.admin.ch/home.html?tab=overview";
const CSS_SELECTOR_STR: &str = "div[class=\"overview__local-forecast clearfix\"]";
const HEADER_REFERER_K: &str = "Referer";
const HEADER_REFERER_V: &str = URL_WITH_JSON_LINK;

pub fn get_forecast_chart_json_url(plz: u32) -> Result<String> {
    // get the html page with the link to the json
    let mut response = isahc::get(URL_WITH_JSON_LINK)?;
    let html_string = response.text()?;

    let mut json_url: String = String::new();

    {
        let fragment = Html::parse_fragment(&html_string);
        let selector = match Selector::parse(CSS_SELECTOR_STR) {
            Ok(s) => s,
            Err(e) => return Err(Error::CSSError(format!("{:?}", e))),
        };
        let section: String = match fragment.select(&selector).next() {
            Some(s) => s.html(),
            None => {
                return Err(Error::HtmlError(
                    "Scraper was unable to find the forecast box!",
                ))
            }
        };

        json_url.push_str(
            &match match section.split("data-json-url=\"").take(2).last() {
                Some(s) => s,
                None => return Err(Error::HtmlError("Could not find \"data-json-url\"")),
            }
            .split('\"')
            .next()
            {
                Some(s) => Ok(format!("https://www.meteoschweiz.admin.ch{}", s)),
                None => Err(Error::HtmlError("\"data-json-url\" has an odd format!")),
            }?,
        );
    }

    // replace the plz
    // remove the XXXX00.json form the end
    json_url.truncate(json_url.len() - "800100.json".len());
    // push plz: XXXX
    json_url.push_str(&format!("{}", plz));
    // push 00.json
    json_url.push_str("00.json");

    Ok(json_url)
}

pub fn fetch_forecast(json_url: &String, icon_folder: &str) -> Result<Forecast> {
    let icon_folder = shellexpand::full(icon_folder)?.into_owned();
    let client = HttpClient::new()?;
    let request = Request::get(json_url)
        .header(HEADER_REFERER_K, HEADER_REFERER_V)
        .body(())?;
    let mut response = client.send(request)?;

    let json_string = response.text()?;
    let forecast_builder: ForecastBuilder = serde_json::from_str(&json_string)?;
    forecast_builder.build(&icon_folder)
}

#[derive(Debug, Serialize)]
pub struct ForecastLong {
    day_labels: String,
    temp_min: i32,
    temp_max: i32,
    rain_max: i32,
    rainfall: Vec<ForecastValueMinMax>,
    temperature: Vec<ForecastValueMinMax>,
    icons: Vec<ForecastIcon>,
}

impl ForecastLong {
    pub fn from(fc: &Forecast) -> Self {
        Self {
            day_labels: fc
                .iter()
                .map(|x| x.day.as_ref())
                .collect::<Vec<&str>>()
                .join(","),
            temp_min: fc.iter().fold(
                1000,
                |x, day| if x < day.temp_min { x } else { day.temp_min },
            ),
            temp_max: fc.iter().fold(
                -1000,
                |x, day| if x > day.temp_max { x } else { day.temp_max },
            ),
            rain_max: fc.iter().fold(
                -1000,
                |x, day| if x > day.rain_max { x } else { day.rain_max },
            ),
            rainfall: fix(fc
                .iter()
                .map(|x| x.rainfall.clone().into_iter())
                .flatten()
                .collect()),
            temperature: fix(fc
                .iter()
                .map(|x| x.temperature.clone().into_iter())
                .flatten()
                .collect()),
            icons: fix_icon(
                fc.iter()
                    .map(|x| x.icons.clone().into_iter())
                    .flatten()
                    .collect(),
            ),
        }
    }
}

fn fix<T>(v: Vec<T>) -> Vec<T>
where
    T: PartialEq + Timestamped,
{
    let v = remove_duplicate(v);
    fix_timestamps(v)
}

fn fix_icon<T>(v: Vec<T>) -> Vec<T>
where
    T: Timestamped,
{
    let v = remove_every_second(v);
    fix_timestamps(v)
}

fn fix_timestamps<T>(mut v: Vec<T>) -> Vec<T>
where
    T: Timestamped,
{
    let mut h_offset = -24.0;
    let mut current_t = 10000.0;
    for item in v.iter_mut() {
        if item.time() < current_t {
            h_offset += 24.0;
        }
        current_t = item.time();
        item.set_time((item.time() + h_offset) / 24.0);
    }
    v
}

fn remove_every_second<T>(v: Vec<T>) -> Vec<T> {
    let mut result: Vec<T> = Vec::with_capacity((v.len() + 1) / 2);
    let mut skip = false;
    for item in v.into_iter() {
        if !skip {
            result.push(item);
        }
        skip = !skip;
    }
    result
}

fn remove_duplicate<T>(v: Vec<T>) -> Vec<T>
where
    T: PartialEq,
{
    let mut result: Vec<T> = Vec::with_capacity(v.len() - 6);
    let mut iter = v.into_iter().peekable();
    while let Some(item) = iter.next() {
        if let Some(next) = iter.peek() {
            if next != &item {
                result.push(item);
            }
        } else {
            result.push(item);
        }
    }
    result
}

pub type Forecast = Vec<ForecastDay>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ForecastDay {
    day: String,
    rainfall: Vec<ForecastValueMinMax>,
    sunshine: Vec<ForecastValue>,
    temperature: Vec<ForecastValueMinMax>,
    icons: Vec<ForecastIcon>,
    wind: Vec<ForecastWind>,
    wind_gust_peak: Vec<ForecastValue>,
    temp_min: i32,
    temp_max: i32,
    rain_max: i32,
}

trait Timestamped {
    fn time(&self) -> f64;
    fn set_time(&mut self, new_time: f64);
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ForecastWind {
    time: f64,
    strength: f64,
    direction: String,
}

impl Timestamped for ForecastWind {
    fn time(&self) -> f64 {
        self.time
    }
    fn set_time(&mut self, new_time: f64) {
        self.time = new_time
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ForecastValue {
    time: f64,
    value: f64,
}

impl Timestamped for ForecastValue {
    fn time(&self) -> f64 {
        self.time
    }
    fn set_time(&mut self, new_time: f64) {
        self.time = new_time
    }
}

impl ForecastValue {
    fn from(obj: &Vec<I64orF64>) -> Result<Self> {
        if obj.len() != 2 {
            Err(Error::ForecastBuildError(
                "ForecastValue requires a vector with 2 elements!",
            ))
        } else {
            Ok(Self {
                time: timestamp_to_time(obj[0].to_i64()?)?,
                value: obj[1].as_f64(),
            })
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ForecastValueMinMax {
    time: f64,
    value: f64,
    low: f64,
    high: f64,
}

impl Timestamped for ForecastValueMinMax {
    fn time(&self) -> f64 {
        self.time
    }
    fn set_time(&mut self, new_time: f64) {
        self.time = new_time
    }
}

impl ForecastValueMinMax {
    fn from(value_obj: &Vec<I64orF64>, range_obj: &Vec<I64orF64>) -> Result<Self> {
        if value_obj.len() != 2 {
            Err(Error::ForecastBuildError(
                "ForecastValue requires a vector with 2 elements!",
            ))
        } else if range_obj.len() != 3 {
            Err(Error::ForecastBuildError(
                "ForecastRange requires a vector with 3 elements!",
            ))
        } else if value_obj[0] != range_obj[0] {
            Err(Error::ForecastBuildError(
                "Time of range-value pari does not match!",
            ))
        } else {
            Ok(Self {
                time: timestamp_to_time(value_obj[0].to_i64()?)?,
                value: value_obj[1].as_f64(),
                low: range_obj[1].as_f64(),
                high: range_obj[2].as_f64(),
            })
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForecastIcon {
    time: f64,
    icon: String,
}

impl Timestamped for ForecastIcon {
    fn time(&self) -> f64 {
        self.time
    }
    fn set_time(&mut self, new_time: f64) {
        self.time = new_time
    }
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct ForecastBuilder {
    days: Vec<ForecastDayBuilder>,
}

impl ForecastBuilder {
    fn build(self, icon_path: &str) -> Result<Forecast> {
        let mut result = Forecast::new();
        let mut days = self.days.into_iter().peekable();
        while let Some(day) = days.next() {
            let mut parsed_day = ForecastDay {
                day: day.day_string,
                rainfall: zip(day.variance_rain.iter(), day.rainfall.iter())
                    .map(|(range, value)| ForecastValueMinMax::from(value, range).unwrap())
                    .collect(),
                sunshine: day
                    .sunshine
                    .iter()
                    .map(|s| ForecastValue::from(s).unwrap())
                    .collect(),
                temperature: zip(day.variance_range.iter(), day.temperature.iter())
                    .map(|(range, value)| ForecastValueMinMax::from(value, range).unwrap())
                    .collect(),
                icons: day
                    .symbols
                    .into_iter()
                    .map(|s| s.build(icon_path).unwrap())
                    .collect(),
                wind: day.wind.build()?,
                wind_gust_peak: day.wind_gust_peak.build()?,
                temp_min: 0,
                temp_max: 0,
                rain_max: 10,
            };
            // push the next entry to the current one
            if let Some(next_day) = days.peek() {
                parsed_day.rainfall.push(ForecastValueMinMax::from(
                    &next_day.rainfall[0],
                    &next_day.variance_rain[0],
                )?);
                parsed_day
                    .sunshine
                    .push(ForecastValue::from(&next_day.sunshine[0])?);
                parsed_day.temperature.push(ForecastValueMinMax::from(
                    &next_day.temperature[0],
                    &next_day.variance_range[0],
                )?);
                parsed_day.rainfall.last_mut().unwrap().time += 24.0;
                parsed_day.sunshine.last_mut().unwrap().time += 24.0;
                parsed_day.temperature.last_mut().unwrap().time += 24.0;
            }

            let min_temp =
                parsed_day
                    .temperature
                    .iter()
                    .fold(1000.0, |x, i| if x < i.low { x } else { i.low });
            let max_temp =
                parsed_day
                    .temperature
                    .iter()
                    .fold(-1000.0, |x, i| if x > i.high { x } else { i.high });
            let max_rain =
                parsed_day
                    .rainfall
                    .iter()
                    .fold(0.0, |x, i| if x > i.high { x } else { i.high });

            let mut min_temp_round = min_temp as i32;
            let mut max_temp_round = (max_temp + 0.999) as i32;
            let mut max_rain_round = (max_rain + 0.999) as i32;
            if (min_temp - min_temp_round as f64) < 0.5 {
                min_temp_round -= 1
            }
            if (max_temp_round as f64 - max_temp) < 0.5 {
                max_temp_round += 1
            }
            if (max_rain_round as f64 - max_rain) < 1.0 {
                max_rain_round += 1
            }
            if max_rain_round < 10 {
                max_rain_round = 10;
            }

            parsed_day.temp_min = min_temp_round;
            parsed_day.temp_max = max_temp_round;
            parsed_day.rain_max = max_rain_round;

            result.push(parsed_day);
        }
        Ok(result)
    }
}

#[derive(Debug, Deserialize)]
struct ForecastDayBuilder {
    current_time: Option<i64>,
    current_time_string: Option<String>,
    min_date: i64,
    max_date: i64,
    day_string: String,
    rainfall: Vec<Vec<I64orF64>>,
    sunshine: Vec<Vec<I64orF64>>,
    temperature: Vec<Vec<I64orF64>>,
    variance_range: Vec<Vec<I64orF64>>,
    variance_rain: Vec<Vec<I64orF64>>,
    symbols: Vec<ForecastSymbolBuilder>,
    wind: ForecastWindBuilder,
    wind_gust_peak: ForecastGustBuilder,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
enum I64orF64 {
    I64(i64),
    F64(f64),
}

impl I64orF64 {
    fn to_i64(&self) -> Result<i64> {
        match self {
            Self::I64(x) => Ok(*x),
            Self::F64(_) => Err(Error::ForecastBuildError("Expected integer, found float")),
        }
    }
    fn as_f64(&self) -> f64 {
        match self {
            Self::I64(x) => *x as f64,
            Self::F64(x) => *x,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ForecastSymbolBuilder {
    timestamp: i64,
    weather_symbol_id: u64,
}

impl ForecastSymbolBuilder {
    fn build(self, icon_path: &str) -> Result<ForecastIcon> {
        Ok(ForecastIcon {
            time: timestamp_to_time(self.timestamp)?,
            icon: format!(
                "{}/{}.pdf",
                icon_path,
                if self.weather_symbol_id > 100 {
                    self.weather_symbol_id - 100
                } else {
                    self.weather_symbol_id
                }
            ),
        })
    }
}

#[derive(Debug, Deserialize)]
struct ForecastWindBuilder {
    data: Vec<Vec<I64orF64>>,
    symbols: Vec<ForecastWindSymbolBuilder>,
}

impl ForecastWindBuilder {
    fn build(self) -> Result<Vec<ForecastWind>> {
        let mut result: Vec<ForecastWind> = Vec::with_capacity(self.data.len());
        let mut data_iter = self.data.iter().peekable();
        let mut symbol_iter = self.symbols.iter().peekable();
        let mut current_symbol = match symbol_iter.next() {
            None => {
                return Err(Error::ForecastBuildError(
                    "At least one wind symbol must exist",
                ))
            }
            Some(s) => s,
        };

        if let Some(first_data_peek) = data_iter.peek() {
            if first_data_peek.len() != 2
                || first_data_peek[0].to_i64()? != current_symbol.timestamp
            {
                return Err(Error::ForecastBuildError(
                    "The first symbol and the first measurement of wind does not match!",
                ));
            }
        } else {
            return Err(Error::ForecastBuildError(
                "No values received for the wind!",
            ));
        }

        while let Some(data) = data_iter.next() {
            if data.len() != 2 {
                return Err(Error::ForecastBuildError(
                    "Wind data vector is expected to have length 2!",
                ));
            }
            if let Some(next_symbol) = symbol_iter.peek() {
                if next_symbol.timestamp <= data[0].to_i64()? {
                    current_symbol = symbol_iter.next().unwrap();
                }
            }
            result.push(ForecastWind {
                time: timestamp_to_time(data[0].to_i64()?)?,
                strength: data[1].as_f64(),
                direction: current_symbol.symbol_id.clone(),
            });
        }

        Ok(result)
    }
}

#[derive(Debug, Deserialize)]
struct ForecastWindSymbolBuilder {
    timestamp: i64,
    symbol_id: String,
}

#[derive(Debug, Deserialize)]
struct ForecastGustBuilder {
    data: Vec<Vec<I64orF64>>,
}

impl ForecastGustBuilder {
    fn build(self) -> Result<Vec<ForecastValue>> {
        let mut result = Vec::new();
        for data in self.data {
            result.push(ForecastValue::from(&data)?)
        }
        Ok(result)
    }
}

fn timestamp_to_time(timestamp: i64) -> Result<f64> {
    let local_tz = chrono::Local;
    let t = local_tz.timestamp(timestamp / 1000, 0);
    let time = t.time();
    Ok(time.hour() as f64 + ((time.minute() as f64 + (time.second() as f64 / 60.0)) / 60.0))
}

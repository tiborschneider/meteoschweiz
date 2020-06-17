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

use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Isahc Error: {0}")]
    IsahcError(#[from] isahc::Error),
    #[error("HTTP Error: {0}")]
    HttpError(#[from] http::Error),
    #[error("HTML Error: {0}")]
    HtmlError(&'static str),
    #[error("CSS Error: {0}")]
    CSSError(String),
    #[error("Json Error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Forecast Building error: {0}")]
    ForecastBuildError(&'static str),
    #[error("PathExpand Error: {0}")]
    PathExpandError(#[from] shellexpand::LookupError<std::env::VarError>),
    #[error("UTF8 Error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("Tera Template Error: {0}")]
    TeraError(#[from] tera::Error),
    #[error("Time Error: {0}")]
    TimeError(&'static str),
    #[error("Bincode Error: {0}")]
    BincodeError(#[from] bincode::Error),
    #[error("Pdflatex exited with error code: {0:?}")]
    PdflatexError(Option<i32>),
}

pub type Result<T> = std::result::Result<T, Error>;

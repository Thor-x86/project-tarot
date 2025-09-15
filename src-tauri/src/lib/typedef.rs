/*
 * Project::Tarot, a simple LSTM implementation with GUI
 * Copyright (C) 2025 Athaariq A. Ramadhani <foss@athaariq.my.id>
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
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use burn::backend::NdArray;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap, fs::File, io::BufReader, path::PathBuf, sync::Arc};

use crate::train::model::LstmNetwork;

#[derive(Default, Serialize, Clone)]
pub(crate) struct ErrorInfo {
	pub title: Cow<'static, str>,
	pub message: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum CellValue {
	String(String),
	Number(f64),
	RowID(u32),
	DateTime(DateTime<Local>),
	Boolean(bool),
}

impl Default for CellValue {
	fn default() -> Self {
		CellValue::String(String::new())
	}
}

#[derive(Default)]
pub(crate) enum SourceData {
	#[default]
	None,
	Csv(csv::Reader<File>),
	Spreadsheet(calamine::Sheets<BufReader<File>>),
}

#[derive(Default, Clone)]
pub(crate) struct HistoricalData {
	pub batches: Vec<Vec<f64>>,
	pub first_timestamp: u64,
	pub timestamp_interval: u32,
}

#[derive(Default, Clone)]
pub(crate) struct NormalParam {
	pub mean: f64,
	pub stdev: f64,
}

#[derive(Default)]
pub(crate) struct AppState {
	pub source_path: Option<PathBuf>,
	pub source_data: SourceData,
	pub dropped_row_indices: Vec<u64>,
	pub column_lookup: Option<HashMap<Arc<String>, usize>>,
	pub preprocessed_data: Option<HistoricalData>,
	pub train_progress: super::train::typedef::TrainProgress,
	pub trained_model: Option<LstmNetwork<NdArray>>,
	pub normal_param: Option<NormalParam>,
	pub predicted_data: Option<Vec<(DateTime<Local>, f64)>>,
	pub page_index: u8,
}

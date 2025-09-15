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
use serde::Serialize;

use crate::train::model::LstmNetwork;

#[derive(Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ComparisonPoint {
	pub x: DateTime<Local>,
	pub y0: Option<f64>,
	pub y1: Option<f64>,
}

#[derive(Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EvaluationReport {
	pub confidence: f32,
	pub graph: Vec<ComparisonPoint>,
	pub high_peak: Option<ComparisonPoint>,
	pub low_peak: Option<ComparisonPoint>,
}

pub(super) struct PredictionInput {
	pub preprocessed_data: crate::typedef::HistoricalData,
	pub trained_model: LstmNetwork<NdArray>,
	pub normal_param: crate::typedef::NormalParam,
	pub last_confidence: f32,
}

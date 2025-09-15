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

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use strum_macros::EnumIter;

#[derive(Default, PartialEq, Eq, Deserialize, Serialize, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ColumnType {
	#[default]
	STRING,
	NUMBER,
	#[serde(rename = "dateTime")]
	DATETIME,
	BOOLEAN,
}

#[derive(Default, Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ColumnInfo {
	pub field: Arc<String>,
	pub header_name: String,
	#[serde(rename = "type")]
	pub column_type: ColumnType,
}

#[derive(Default, PartialEq, Eq, EnumIter, Deserialize, Serialize, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum BatchPeriode {
	MINUTELY,
	HOURLY,
	DAILY,
	WEEKLY,
	MONTHLY,
	#[default]
	YEARLY,
}

impl std::fmt::Display for BatchPeriode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			BatchPeriode::MINUTELY => write!(f, "minutely"),
			BatchPeriode::HOURLY => write!(f, "hourly"),
			BatchPeriode::DAILY => write!(f, "daily"),
			BatchPeriode::WEEKLY => write!(f, "weekly"),
			BatchPeriode::MONTHLY => write!(f, "monthly"),
			BatchPeriode::YEARLY => write!(f, "yearly"),
		}
	}
}

#[derive(Default, PartialEq, Eq, Deserialize, Serialize, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum SelectionType {
	#[default]
	EXCLUDE,
	INCLUDE,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub(crate) struct RowSelection {
	pub ids: Vec<u64>,
	#[serde(rename = "type")]
	pub selection_type: SelectionType,
}

#[derive(Default, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SheetInfo {
	pub tab_name: Option<String>,
	pub columns: Vec<ColumnInfo>,
	pub rows: Vec<HashMap<Arc<String>, crate::typedef::CellValue>>,
	pub allowed_batch_periodes: Vec<BatchPeriode>,
	pub selected_datetime_column: Arc<String>,
	pub selected_predictable_column: Arc<String>,
	pub selected_batch_periode: BatchPeriode,
	pub row_selection: RowSelection,
}

#[derive(Default, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DataInfo {
	pub name: String,
	pub tabs: Option<Vec<String>>,
	pub sheet_info: SheetInfo,
}

#[derive(Default, Clone, Copy)]
pub(super) struct ColumnCounter {
	pub string: u64,
	pub number: u64,
	pub datetime: u64,
	pub boolean: u64,
}

#[derive(Default, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PreprocessConfig {
	pub tab_name: Option<String>, // None only if CSV
	pub datetime_column: Arc<String>,
	pub predictable_column: Arc<String>,
	pub batch_periode: BatchPeriode,
	pub row_selection: RowSelection,
}

pub(super) struct BatchInfo {
	pub sequence_size: u32,
	pub sequence_count: u32,
	pub interval: u32,
}

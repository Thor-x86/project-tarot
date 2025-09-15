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

use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Datelike, Local, MappedLocalTime, NaiveDateTime, Timelike};
use parse_datetime::parse_datetime;
use rsl_interpolation::{Akima, InterpType, Interpolation};
use strum::IntoEnumIterator;

use super::typedef::*;
use crate::typedef::CellValue;

pub(super) fn decide_allowed_batch_periodes(
	columns: &Vec<ColumnInfo>,
	rows: &Vec<HashMap<Arc<String>, CellValue>>,
) -> Vec<BatchPeriode> {
	let fields = columns
		.iter()
		.filter(|each| each.column_type == ColumnType::DATETIME)
		.map(|each| each.field.clone())
		.collect::<Vec<_>>();

	let datetime_columns = fields
		.iter()
		.map(|each_column| {
			rows.iter()
				.filter_map(|each_cell| match each_cell.get(each_column)? {
					CellValue::DateTime(date_time) => Some(date_time),
					_ => None,
				})
				.collect::<Vec<_>>()
		})
		.collect::<Vec<_>>();

	let smallest_interval = datetime_columns
		.iter()
		.filter_map(|each_column| {
			let future_iter = each_column.iter().skip(1);

			each_column
				.iter()
				.zip(future_iter)
				.map(|(now, next)| next.timestamp() - now.timestamp())
				.min()
		})
		.min();

	match smallest_interval {
		Some(found) => {
			let skip_count = match found.abs() as u64 {
				0..60 => 0,           // Minutely
				60..3600 => 1,        // Hourly
				3600..86400 => 2,     // Daily
				86400..604800 => 3,   // Weekly
				604800..2592000 => 4, // Monthly
				_ => 5,               // Yearly
			};

			BatchPeriode::iter().skip(skip_count).collect::<Vec<_>>()
		}
		None => BatchPeriode::iter().collect::<Vec<_>>(),
	}
}

pub(super) fn parse_cell(cell: &str) -> CellValue {
	if let Ok(datetime_value) = parse_datetime(cell) {
		CellValue::DateTime(datetime_value.into())
	} else if let Ok(number_value) = cell.parse::<f64>() {
		CellValue::Number(number_value)
	} else if let Ok(boolean_value) = cell.parse::<bool>() {
		CellValue::Boolean(boolean_value)
	} else {
		CellValue::String(cell.to_string())
	}
}

pub(super) fn unwrap_timezone_assumption(
	timezone: MappedLocalTime<DateTime<Local>>,
	raw_cell: &calamine::Data,
) -> CellValue {
	match timezone {
		chrono::offset::LocalResult::Single(datetime) => CellValue::DateTime(datetime),
		chrono::offset::LocalResult::Ambiguous(earliest, _latest) => CellValue::DateTime(earliest),
		chrono::offset::LocalResult::None => CellValue::String(raw_cell.to_string()),
	}
}

pub(super) fn calculate_batch_info(
	selected_source_data: &Vec<(u64, f64)>,
	periode: BatchPeriode,
) -> BatchInfo {
	let timeseries_data = selected_source_data
		.iter()
		.map(|(x, y)| {
			(
				DateTime::from_timestamp(x.clone() as i64, 0)
					.unwrap()
					.naive_utc(),
				*y,
			)
		})
		.collect::<Vec<_>>();

	// Find for sequence size
	let mut sequence_size = 0u32;
	let mut count = 0u32;
	let mut last_datetime = timeseries_data
		.get(0)
		.and_then(|found| Some(found.0))
		.unwrap_or(NaiveDateTime::MIN);
	match periode {
		BatchPeriode::MINUTELY => timeseries_data.iter().for_each(|each| {
			if each.0.minute() != last_datetime.minute() {
				sequence_size = sequence_size.max(count);
				count = 0;
				last_datetime = each.0;
			} else {
				count += 1;
			}
		}),
		BatchPeriode::HOURLY => timeseries_data.iter().for_each(|each| {
			if each.0.hour() != last_datetime.hour() {
				sequence_size = sequence_size.max(count);
				count = 0;
				last_datetime = each.0;
			} else {
				count += 1;
			}
		}),
		BatchPeriode::DAILY => timeseries_data.iter().for_each(|each| {
			if each.0.day() != last_datetime.day() {
				sequence_size = sequence_size.max(count);
				count = 0;
				last_datetime = each.0;
			} else {
				count += 1;
			}
		}),
		BatchPeriode::WEEKLY => timeseries_data.iter().for_each(|each| {
			if each.0.iso_week().week() != last_datetime.iso_week().week() {
				sequence_size = sequence_size.max(count);
				count = 0;
				last_datetime = each.0;
			} else {
				count += 1;
			}
		}),
		BatchPeriode::MONTHLY => timeseries_data.iter().for_each(|each| {
			if each.0.month() != last_datetime.month() {
				sequence_size = sequence_size.max(count);
				count = 0;
				last_datetime = each.0;
			} else {
				count += 1;
			}
		}),
		BatchPeriode::YEARLY => timeseries_data.iter().for_each(|each| {
			if each.0.year() != last_datetime.year() {
				sequence_size = sequence_size.max(count);
				count = 0;
				last_datetime = each.0;
			} else {
				count += 1;
			}
		}),
	}

	if sequence_size == 0 {
		sequence_size = 1;
	}

	let periodic_secs = match periode {
		BatchPeriode::MINUTELY => 60u32,
		BatchPeriode::HOURLY => 3600u32,
		BatchPeriode::DAILY => 86400u32,
		BatchPeriode::WEEKLY => 604800u32,
		BatchPeriode::MONTHLY => 2592000u32,
		BatchPeriode::YEARLY => 31556952u32,
	};

	let interval = periodic_secs / sequence_size;

	let min_datetime = timeseries_data
		.first()
		.and_then(|found| Some(found.0))
		.unwrap_or_default();
	let max_datetime = timeseries_data
		.last()
		.and_then(|found| Some(found.0))
		.unwrap_or(min_datetime);
	let delta_secs = (max_datetime - min_datetime).num_seconds().abs() as u64;

	let sequence_count = (delta_secs / (interval as u64 * sequence_size as u64)) as u32;

	BatchInfo {
		sequence_size,
		sequence_count,
		interval,
	}
}

pub(super) fn resample_to_batches(
	selected_source_data: &Vec<(u64, f64)>,
	batch_info: &BatchInfo,
) -> Result<Vec<Vec<f64>>, String> {
	if selected_source_data.is_empty() {
		return Err(format!("Cannot re-sample an empty table"));
	}

	let xa = selected_source_data
		.iter()
		.map(|each| each.0 as f64)
		.collect::<Vec<_>>();
	let ya = selected_source_data
		.iter()
		.map(|each| each.1)
		.collect::<Vec<_>>();

	let interp = match Akima.build(&xa, &ya) {
		Ok(ok) => ok,
		Err(err) => return Err(err.to_string()),
	};

	let mut interp_cache = rsl_interpolation::Accelerator::new();

	let xa_first = xa.first().cloned().unwrap_or_default();
	let xa_last = xa.last().cloned().unwrap_or(xa_first);
	let ya_first = ya.first().cloned().unwrap_or_default();
	let ya_last = ya.last().cloned().unwrap_or(ya_first);

	let mut output = Vec::<Vec<f64>>::with_capacity(batch_info.sequence_count as usize);
	for batch_index in 0..batch_info.sequence_count {
		let offset = batch_index as f64 * batch_info.sequence_size as f64;

		let mut each_batch = Vec::<f64>::with_capacity(batch_info.sequence_size as usize);
		for element_index in 0..batch_info.sequence_size {
			let x = xa_first + (batch_info.interval as f64 * (element_index as f64 + offset));

			// GSL's Akima cannot do extrapolation, need to clip it
			let y = if x <= xa_first {
				ya_first
			} else if x >= xa_last {
				ya_last
			} else {
				match interp.eval(&xa, &ya, x, &mut interp_cache) {
					Ok(ok) => ok,
					Err(err) => return Err(err.to_string()),
				}
			};

			each_batch.push(y);
		}

		output.push(each_batch);
	}

	Ok(output)
}

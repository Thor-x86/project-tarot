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

use calamine::{DataType, Reader};
use chrono::Local;
use csv::Position;
use parse_datetime::parse_datetime;
use std::{
	borrow::Cow,
	collections::HashMap,
	hash::RandomState,
	sync::{Arc, Mutex},
};
use tauri::{AppHandle, Emitter, State};

use super::helper::*;
use super::typedef::*;
use crate::typedef::{AppState, CellValue, ErrorInfo, HistoricalData, SourceData};

const ERROR_EXTENSION: &'static str = "File Type Unsupported";
const ERROR_HEADER: &'static str = "Cannot Read Header";
const ERROR_CONTENT: &'static str = "Failed to Read Content";
const ERROR_INCOMPLETE: &'static str = "Data is Incomplete";
const ERROR_INCOMPLETE_SUFFIX: &'static str = "this CSV file";
const ERROR_INCONSISTENT: &'static str = "Inconsistent Data Type";
const ERROR_MODIFIED: &'static str = "Selected Data just Modified";
const ERROR_RESET: &'static str = "Cannot Re-read the Data";

#[tauri::command]
pub(crate) fn get_data_info(app: AppHandle, state: State<'_, Mutex<AppState>>) -> DataInfo {
	let source_path = {
		let mut guarded_state = match state.lock() {
			Ok(ok) => ok,
			Err(err) => {
				app.emit::<ErrorInfo>(
					crate::event::DIALOG_ERROR,
					ErrorInfo {
						title: Cow::Borrowed("State Inaccessible on Getting Data"),
						message: err.to_string(),
					},
				)
				.unwrap();
				app.emit(crate::event::PAGE_MOVE, 0).unwrap();
				return Default::default();
			}
		};

		// Fallback to Data page if guarded_state.source_path was unset
		if guarded_state.source_path.is_none() {
			*guarded_state = Default::default();
			app.emit(crate::event::PAGE_MOVE, guarded_state.page_index)
				.unwrap();
			return Default::default();
		}

		guarded_state.source_path.clone().unwrap()
	};

	let extension_option = source_path.extension();
	if extension_option.is_none() {
		app.emit::<ErrorInfo>(
			crate::event::DIALOG_ERROR,
			ErrorInfo {
				title: Cow::Borrowed(ERROR_EXTENSION),
				message: format!(
					"Because \"{}\" file has no extension",
					source_path.to_string_lossy().into_owned()
				),
			},
		)
		.unwrap();

		crate::restart(app, state);
		return Default::default();
	}

	let extension_str_option = extension_option.unwrap().to_str();
	if extension_str_option.is_none() {
		app.emit::<ErrorInfo>(
			crate::event::DIALOG_ERROR,
			ErrorInfo {
				title: Cow::Borrowed(ERROR_EXTENSION),
				message: String::from("Cannot transform file extension encoding"),
			},
		)
		.unwrap();

		crate::restart(app, state);
		return Default::default();
	}

	let extension = extension_str_option.unwrap().to_lowercase();

	let source_data = match extension.as_str() {
		"csv" => match csv::Reader::from_path(&source_path) {
			Ok(csv_file) => SourceData::Csv(csv_file),
			Err(err) => {
				app.emit::<ErrorInfo>(
					crate::event::DIALOG_ERROR,
					ErrorInfo {
						title: Cow::Borrowed("Failed to Read CSV File"),
						message: err.to_string(),
					},
				)
				.unwrap();

				crate::restart(app, state);
				return Default::default();
			}
		},
		"xlsx" | "xls" | "xlsb" | "ods" => match calamine::open_workbook_auto(&source_path) {
			Ok(sheet_file) => SourceData::Spreadsheet(sheet_file),
			Err(err) => {
				app.emit::<ErrorInfo>(
					crate::event::DIALOG_ERROR,
					ErrorInfo {
						title: Cow::Owned(format!(
							"Failed to Read {} File",
							extension.to_uppercase()
						)),
						message: err.to_string(),
					},
				)
				.unwrap();

				crate::restart(app, state);
				return Default::default();
			}
		},
		_ => {
			app.emit::<ErrorInfo>(
				crate::event::DIALOG_ERROR,
				ErrorInfo {
					title: Cow::Borrowed(ERROR_EXTENSION),
					message: format!("Cannot parse \"{}\" file extension", extension),
				},
			)
			.unwrap();

			crate::restart(app, state);
			return Default::default();
		}
	};

	let name = source_path
		.file_name()
		.and_then(|name| Some(name.to_str()?.to_string()))
		.unwrap_or(String::from("(unknown)"));

	let tabs = match &source_data {
		SourceData::None => unreachable!(),
		SourceData::Csv(_reader) => None,
		SourceData::Spreadsheet(sheets) => Some(sheets.sheet_names()),
	};

	{
		let mut guarded_state = match state.lock() {
			Ok(ok) => ok,
			Err(err) => {
				app.emit::<ErrorInfo>(
					crate::event::DIALOG_ERROR,
					ErrorInfo {
						title: Cow::Borrowed("State Inaccessible after Picked File"),
						message: err.to_string(),
					},
				)
				.unwrap();
				app.emit(crate::event::PAGE_MOVE, 0).unwrap();
				return Default::default();
			}
		};

		guarded_state.source_data = source_data;
	};

	let selected_tab = tabs
		.as_ref()
		.and_then(|tabs| tabs.iter().nth(0).cloned())
		.unwrap_or(String::new());

	let sheet_info = select_sheet(app, state, selected_tab);

	DataInfo {
		name,
		tabs,
		sheet_info,
	}
}

#[tauri::command]
pub(crate) fn select_sheet(
	app: AppHandle,
	state: State<'_, Mutex<AppState>>,
	tab_name: String,
) -> SheetInfo {
	let mut guarded_state = match state.lock() {
		Ok(ok) => ok,
		Err(err) => {
			app.emit::<ErrorInfo>(
				crate::event::DIALOG_ERROR,
				ErrorInfo {
					title: Cow::Borrowed("State Inaccessible on Reading Sheet"),
					message: err.to_string(),
				},
			)
			.unwrap();
			app.emit(crate::event::PAGE_MOVE, 0).unwrap();
			return Default::default();
		}
	};

	guarded_state.column_lookup = None;

	if let SourceData::None = &guarded_state.source_data {
		app.emit(crate::event::PAGE_MOVE, guarded_state.page_index)
			.unwrap();
		return Default::default();
	}

	// CSV will always ignore this
	let selected_sheet = match &mut guarded_state.source_data {
		SourceData::Spreadsheet(sheets) => {
			let sheet = match sheets.worksheet_range(&tab_name) {
				Ok(ok) => ok,
				Err(err) => {
					app.emit::<ErrorInfo>(
						crate::event::DIALOG_ERROR,
						ErrorInfo {
							title: Cow::Borrowed("Cannot Read Worksheet"),
							message: err.to_string(),
						},
					)
					.unwrap();

					return SheetInfo {
						tab_name: Some(tab_name),
						..Default::default()
					};
				}
			};

			Some(sheet)
		}
		_ => None,
	};

	let column_names = match &mut guarded_state.source_data {
		SourceData::None => unreachable!(),
		SourceData::Csv(reader) => {
			let headers = match reader.headers() {
				Ok(ok) => ok,
				Err(err) => {
					app.emit::<ErrorInfo>(
						crate::event::DIALOG_ERROR,
						ErrorInfo {
							title: Cow::Borrowed(ERROR_HEADER),
							message: err.to_string(),
						},
					)
					.unwrap();

					*guarded_state = Default::default();
					app.emit(crate::event::PAGE_MOVE, guarded_state.page_index)
						.unwrap();

					return Default::default();
				}
			};

			headers
				.iter()
				.map(|each| each.to_string())
				.collect::<Vec<_>>()
		}
		SourceData::Spreadsheet(_) => {
			let selected_sheet = selected_sheet.as_ref().unwrap();
			match selected_sheet.headers() {
				Some(found) => found,
				None => {
					app.emit::<ErrorInfo>(
						crate::event::DIALOG_ERROR,
						ErrorInfo {
							title: Cow::Borrowed(ERROR_HEADER),
							message: String::from("The selected sheet has no header"),
						},
					)
					.unwrap();

					return SheetInfo {
						tab_name: Some(tab_name),
						..Default::default()
					};
				}
			}
		}
	};

	let cells = match &mut guarded_state.source_data {
		SourceData::None => unreachable!(),
		SourceData::Csv(reader) => {
			let results = reader
				.records()
				.map(|each| {
					let row = each.or_else(|err| Err(err.to_string()))?;
					Ok(row.iter().map(parse_cell).collect::<Vec<_>>())
				})
				.collect::<Vec<Result<Vec<CellValue>, String>>>();

			let error = results.iter().find_map(|each| each.as_ref().err()).cloned();
			if let Some(message) = error {
				app.emit::<ErrorInfo>(
					crate::event::DIALOG_ERROR,
					ErrorInfo {
						title: Cow::Borrowed(ERROR_CONTENT),
						message,
					},
				)
				.unwrap();

				return SheetInfo {
					tab_name: Some(tab_name),
					..Default::default()
				};
			}

			results
				.iter()
				.filter_map(|each| each.as_ref().ok())
				.cloned()
				.collect::<Vec<_>>()
		}
		SourceData::Spreadsheet(_) => {
			let sheet = selected_sheet.as_ref().unwrap();
			sheet
				.rows()
				.skip(1) // Skip header row
				.map(|each| {
					each.iter()
						.map(|each| {
							if let Some(number_value) = each.as_f64() {
								CellValue::Number(number_value)
							} else if let Some(datetime_value) = each.as_datetime() {
								unwrap_timezone_assumption(
									datetime_value.and_local_timezone(Local),
									each,
								)
							} else if let Some(date_value) = each.as_date() {
								unwrap_timezone_assumption(
									date_value
										.and_time(Local::now().time())
										.and_local_timezone(Local),
									each,
								)
							} else if let Some(time_value) = each.as_time() {
								unwrap_timezone_assumption(
									Local::now()
										.date_naive()
										.and_time(time_value)
										.and_local_timezone(Local),
									each,
								)
							} else {
								let string_value = each.to_string();
								parse_cell(string_value.as_str())
							}
						})
						.collect::<Vec<_>>()
				})
				.collect::<Vec<_>>()
		}
	};

	// Count how many data types (string, number, ...) occurence each column
	let column_count = column_names.len();
	let column_type_counters = cells.iter().fold(
		std::iter::repeat_n(ColumnCounter::default(), column_count).collect::<Vec<_>>(),
		|mut counters, each_row| {
			for i in 0..column_count {
				let counter = counters.get_mut(i).unwrap();
				let each_cell = match each_row.get(i) {
					Some(found) => found,
					None => continue,
				};

				match each_cell {
					CellValue::String(_) => counter.string += 1,
					CellValue::Number(_) => counter.number += 1,
					CellValue::RowID(_) => {
						unreachable!()
					}
					CellValue::DateTime(_) => counter.datetime += 1,
					CellValue::Boolean(_) => counter.boolean += 1,
				}
			}

			counters
		},
	);

	// Pick the most occuring data type for each column
	let column_types = column_type_counters
		.iter()
		.map(|each| {
			let mut winner = ColumnType::STRING;
			if each.string < each.number {
				winner = ColumnType::NUMBER
			}
			if each.number < each.datetime {
				winner = ColumnType::DATETIME
			}
			if each.datetime < each.boolean {
				winner = ColumnType::BOOLEAN
			}
			winner
		})
		.collect::<Vec<_>>();

	let columns = column_names
		.iter()
		.zip(&column_types)
		.map(|each| ColumnInfo {
			field: Arc::new(each.0.to_lowercase()),
			header_name: each.0.clone(),
			column_type: each.1.clone(),
		})
		.collect::<Vec<_>>();

	let selected_datetime_column = match columns
		.iter()
		.find(|each| each.column_type == ColumnType::DATETIME)
	{
		Some(found) => found.field.clone(),
		None => {
			app.emit::<ErrorInfo>(
				crate::event::DIALOG_ERROR,
				ErrorInfo {
					title: Cow::Borrowed(ERROR_INCOMPLETE),
					message: format!(
						"There is no Date/Time column in {}",
						match &selected_sheet {
							Some(_) => &tab_name,
							None => ERROR_INCOMPLETE_SUFFIX,
						}
					),
				},
			)
			.unwrap();

			// If CSV file
			if selected_sheet.is_none() {
				*guarded_state = Default::default();
				app.emit(crate::event::PAGE_MOVE, guarded_state.page_index)
					.unwrap();
				return Default::default();
			}

			return SheetInfo {
				tab_name: Some(tab_name),
				..Default::default()
			};
		}
	};

	let selected_predictable_column = match columns
		.iter()
		.find(|each| each.column_type == ColumnType::NUMBER)
	{
		Some(found) => found.field.clone(),
		None => {
			app.emit::<ErrorInfo>(
				crate::event::DIALOG_ERROR,
				ErrorInfo {
					title: Cow::Borrowed(ERROR_INCOMPLETE),
					message: format!(
						"There is no predictable column in {}",
						match &selected_sheet {
							Some(_) => &tab_name,
							None => ERROR_INCOMPLETE_SUFFIX,
						}
					),
				},
			)
			.unwrap();

			// If CSV file
			if selected_sheet.is_none() {
				*guarded_state = Default::default();
				app.emit(crate::event::PAGE_MOVE, guarded_state.page_index)
					.unwrap();
				return Default::default();
			}

			return SheetInfo {
				tab_name: Some(tab_name),
				..Default::default()
			};
		}
	};

	let id_field = Arc::new(String::from("id"));

	let row_count = cells.len();
	let mut dropped_row_indices = Vec::<u64>::new();
	let rows = cells
		.iter()
		.zip(0..row_count)
		.filter(|(each_row, row_index)| {
			let is_row_ok = (0..column_count).all(|i| {
				let each_cell = match each_row.get(i) {
					Some(found) => found,
					None => return false,
				};

				let expected = column_types.get(i).unwrap().clone();
				let actual = match each_cell {
					CellValue::String(_) => ColumnType::STRING,
					CellValue::Number(_) => ColumnType::NUMBER,
					CellValue::RowID(_) => return false,
					CellValue::DateTime(_) => ColumnType::DATETIME,
					CellValue::Boolean(_) => ColumnType::BOOLEAN,
				};

				expected == actual
			});

			if !is_row_ok {
				dropped_row_indices.push(*row_index as u64);
			}

			is_row_ok
		})
		.map(|(each_row, id)| {
			let pairs_iter = columns
				.iter()
				.zip(each_row)
				.map(|(each_column, each_cell)| (each_column.field.clone(), each_cell.clone()));

			let pairs_iter_with_id = [(id_field.clone(), CellValue::RowID(id as u32))]
				.into_iter()
				.chain(pairs_iter);

			HashMap::<_, _, RandomState>::from_iter(pairs_iter_with_id)
		})
		.collect::<Vec<_>>();

	let filtered_row_count = rows.len();
	let source_row_count = cells.len();
	if filtered_row_count != source_row_count {
		if filtered_row_count < 10 {
			app.emit(
				crate::event::DIALOG_ERROR,
				ErrorInfo {
					title: Cow::Borrowed(ERROR_INCONSISTENT),
					message: format!(
					"Only {} rows are unusable after data cleaning, left only {} rows which is too few",
					source_row_count - filtered_row_count,
					filtered_row_count
				),
				},
			)
			.unwrap();

			return SheetInfo {
				tab_name: Some(tab_name),
				..Default::default()
			};
		} else {
			app.emit(crate::event::DIALOG_ERROR, ErrorInfo {
				title: Cow::Borrowed(ERROR_INCONSISTENT),
				message: format!(
					"Data cleaning was automatically done and there are {} rows dropped because of inconsistent cell data type",
					source_row_count - filtered_row_count
				)
			}).unwrap();
		}
	}

	if filtered_row_count < 10 {
		app.emit(
			crate::event::DIALOG_ERROR,
			ErrorInfo {
				title: Cow::Borrowed("Not Enough Rows"),
				message: format!(
					"You selected only {} rows, we need more than (or equal) 10 rows",
					source_row_count - filtered_row_count
				),
			},
		)
		.unwrap();

		return SheetInfo {
			tab_name: Some(tab_name),
			..Default::default()
		};
	}

	let allowed_batch_periodes = decide_allowed_batch_periodes(&columns, &rows);
	let selected_batch_periode = allowed_batch_periodes
		.get(1)
		.or(allowed_batch_periodes.get(0))
		.cloned()
		.unwrap_or(BatchPeriode::YEARLY);

	let row_selection = RowSelection {
		ids: Vec::new(),
		selection_type: SelectionType::EXCLUDE,
	};

	// We don't want to re-parse the header row
	guarded_state.column_lookup = Some(HashMap::from_iter(
		columns
			.iter()
			.map(|each| each.field.clone())
			.zip(0..column_count),
	));

	guarded_state.dropped_row_indices = dropped_row_indices;

	SheetInfo {
		tab_name: selected_sheet.and(Some(tab_name)),
		columns,
		rows,
		allowed_batch_periodes,
		selected_datetime_column,
		selected_predictable_column,
		selected_batch_periode,
		row_selection,
	}
}

#[tauri::command]
pub(crate) async fn submit_preprocess_config(
	app: AppHandle,
	state: State<'_, Mutex<AppState>>,
	config: PreprocessConfig,
) -> Result<(), ()> {
	let mut selected_source_data: Vec<(u64, f64)> = {
		let mut guarded_state = match state.lock() {
			Ok(ok) => ok,
			Err(err) => {
				app.emit::<ErrorInfo>(
					crate::event::DIALOG_ERROR,
					ErrorInfo {
						title: Cow::Borrowed("State Inaccessible before Submission"),
						message: err.to_string(),
					},
				)
				.unwrap();
				app.emit(crate::event::PAGE_MOVE, 0).unwrap();
				return Err(());
			}
		};

		if let SourceData::None = &guarded_state.source_data {
			return Err(());
		}

		let column_lookup = guarded_state
			.column_lookup
			.as_ref()
			.expect("Forgot to set column_lookup in select_sheet");

		let datetime_index = match column_lookup.get(&config.datetime_column) {
			Some(found) => *found,
			None => {
				app.emit::<ErrorInfo>(
					crate::event::DIALOG_ERROR,
					ErrorInfo {
						title: Cow::Borrowed(ERROR_MODIFIED),
						message: String::from("Date/time column suddenly gone in this CSV file"),
					},
				)
				.unwrap();
				return Err(());
			}
		};

		let predictable_index = match column_lookup.get(&config.predictable_column) {
			Some(found) => *found,
			None => {
				app.emit::<ErrorInfo>(
					crate::event::DIALOG_ERROR,
					ErrorInfo {
						title: Cow::Borrowed(ERROR_MODIFIED),
						message: String::from("Predictable column suddenly gone in this CSV file"),
					},
				)
				.unwrap();
				return Err(());
			}
		};

		let dropped_row_indices = guarded_state.dropped_row_indices.clone();

		match &mut guarded_state.source_data {
			SourceData::None => unreachable!(),
			SourceData::Csv(reader) => {
				if let Err(err) = reader.seek(Position::new()) {
					app.emit::<ErrorInfo>(
						crate::event::DIALOG_ERROR,
						ErrorInfo {
							title: Cow::Borrowed(ERROR_RESET),
							message: err.to_string(),
						},
					)
					.unwrap();
					return Err(());
				}

				let mut row_index = 0u64;
				reader
					.records()
					.skip(1)
					.filter(|_| {
						let is_id_listed = config.row_selection.ids.contains(&row_index);
						let is_dropped = dropped_row_indices.contains(&row_index);
						row_index += 1;

						!is_dropped
							&& match config.row_selection.selection_type {
								SelectionType::EXCLUDE => !is_id_listed,
								SelectionType::INCLUDE => is_id_listed,
							}
					})
					.filter_map(|each| {
						let each_row = each.and_then(|ok| Ok(Some(ok))).unwrap_or_else(|_| None)?;

						let x = parse_datetime(each_row.get(datetime_index)?)
							.and_then(|ok| Ok(Some(ok)))
							.unwrap_or_else(|_| None)?
							.timestamp() as u64;

						let y = each_row
							.get(predictable_index)?
							.parse::<f64>()
							.and_then(|ok| Ok(Some(ok)))
							.unwrap_or_else(|_| None)?;

						Some((x, y))
					})
					.collect::<Vec<_>>()
			}
			SourceData::Spreadsheet(sheets) => {
				if config.tab_name.is_none() {
					app.emit::<ErrorInfo>(
						crate::event::DIALOG_ERROR,
						ErrorInfo {
							title: Cow::Borrowed(ERROR_INCOMPLETE),
							message: String::from(
								"Because tab name is missing while loading spreadsheet",
							),
						},
					)
					.unwrap();
					return Err(());
				}

				let tab_name = config.tab_name.unwrap();
				let sheet = match sheets.worksheet_range(&tab_name) {
					Ok(ok) => ok,
					Err(err) => {
						app.emit::<ErrorInfo>(
							crate::event::DIALOG_ERROR,
							ErrorInfo {
								title: Cow::Borrowed(ERROR_MODIFIED),
								message: err.to_string(),
							},
						)
						.unwrap();
						return Err(());
					}
				};

				let mut row_index = 0u64;
				sheet
					.rows()
					.skip(1)
					.filter(|_| {
						let is_id_listed = config.row_selection.ids.contains(&row_index);
						let is_dropped = dropped_row_indices.contains(&row_index);
						row_index += 1;

						!is_dropped
							&& match config.row_selection.selection_type {
								SelectionType::EXCLUDE => !is_id_listed,
								SelectionType::INCLUDE => is_id_listed,
							}
					})
					.filter_map(|each_row| {
						let datetime_cell = each_row.get(datetime_index)?;
						let x = datetime_cell
							.as_datetime()
							.or(datetime_cell
								.as_date()
								.and_then(|found| Some(found.and_time(Local::now().time()))))
							.or(datetime_cell
								.as_time()
								.and_then(|found| Some(Local::now().date_naive().and_time(found))))?
							.and_local_timezone(Local)
							.unwrap()
							.timestamp() as u64;

						let y = each_row.get(predictable_index)?.as_f64()?;

						Some((x, y))
					})
					.collect::<Vec<_>>()
			}
		}
	};

	selected_source_data.sort_unstable_by_key(|each| each.0);

	let batch_info = calculate_batch_info(&selected_source_data, config.batch_periode);

	if batch_info.sequence_count < 2 {
		app.emit::<ErrorInfo>(
			crate::event::DIALOG_ERROR,
			ErrorInfo {
				title: Cow::Borrowed("Unable to Spot The Pattern"),
				message: format!(
					"Because data pattern is not there {}, try to change the quicker period",
					config.batch_periode
				),
			},
		)
		.unwrap();
		return Err(());
	}

	let first_timestamp = selected_source_data.first().cloned().unwrap_or_default().0;
	let timestamp_interval = batch_info.interval;

	let batches_result = match tauri::async_runtime::spawn_blocking(move || {
		resample_to_batches(&selected_source_data, &batch_info)
	})
	.await
	{
		Ok(ok) => ok,
		Err(err) => {
			app.emit::<ErrorInfo>(
				crate::event::DIALOG_ERROR,
				ErrorInfo {
					title: Cow::Borrowed("Cannot Create New Process Thread"),
					message: err.to_string(),
				},
			)
			.unwrap();
			return Err(());
		}
	};

	let batches = match batches_result {
		Ok(ok) => ok,
		Err(err) => {
			app.emit::<ErrorInfo>(
				crate::event::DIALOG_ERROR,
				ErrorInfo {
					title: Cow::Borrowed("Re-sampling Failed"),
					message: err.to_string(),
				},
			)
			.unwrap();
			return Err(());
		}
	};

	let mut guarded_state = match state.lock() {
		Ok(ok) => ok,
		Err(err) => {
			app.emit::<ErrorInfo>(
				crate::event::DIALOG_ERROR,
				ErrorInfo {
					title: Cow::Borrowed("State Inaccessible while Submitting Config"),
					message: err.to_string(),
				},
			)
			.unwrap();
			app.emit(crate::event::PAGE_MOVE, 0).unwrap();
			return Err(());
		}
	};

	guarded_state.preprocessed_data = Some(HistoricalData {
		batches,
		first_timestamp,
		timestamp_interval,
	});

	Ok(())
}

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

use crate::typedef::ErrorInfo;
use std::{borrow::Cow, sync::Mutex};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_dialog::FilePath;

use super::helper::*;
use super::typedef::*;

const ERROR_SAVE: &'static str = "Cannot Save File";

#[tauri::command]
pub(crate) async fn get_evaluation(
	app: AppHandle,
	state: State<'_, Mutex<crate::typedef::AppState>>,
) -> Result<EvaluationReport, ()> {
	let input = {
		let mut guarded_state = match state.lock() {
			Ok(ok) => ok,
			Err(err) => {
				app.emit(crate::event::CORE_PANIC, ()).unwrap();
				app.emit::<ErrorInfo>(
					crate::event::DIALOG_ERROR,
					ErrorInfo {
						title: Cow::Borrowed("State Inaccessible before Prediction"),
						message: err.to_string(),
					},
				)
				.unwrap();
				return Err(());
			}
		};

		let preprocessed_data = match &guarded_state.preprocessed_data {
			Some(found) => found.clone(),
			None => {
				*guarded_state = Default::default();
				app.emit(crate::event::PAGE_MOVE, guarded_state.page_index)
					.unwrap();
				return Err(());
			}
		};

		guarded_state.preprocessed_data = None;

		let trained_model = match &guarded_state.trained_model {
			Some(found) => found.clone(),
			None => {
				*guarded_state = Default::default();
				app.emit(crate::event::PAGE_MOVE, guarded_state.page_index)
					.unwrap();
				return Err(());
			}
		};

		guarded_state.trained_model = None;

		let normal_param = match &guarded_state.normal_param {
			Some(found) => found.clone(),
			None => {
				*guarded_state = Default::default();
				app.emit(crate::event::PAGE_MOVE, guarded_state.page_index)
					.unwrap();
				return Err(());
			}
		};

		let last_confidence = guarded_state
			.train_progress
			.confidence_points
			.last()
			.and_then(|found| Some(found.y))
			.unwrap_or_default();

		PredictionInput {
			preprocessed_data,
			trained_model,
			normal_param,
			last_confidence,
		}
	};

	let confidence = *(&input.last_confidence);

	let cloned_app = app.clone();
	let graph =
		match tauri::async_runtime::spawn_blocking(move || predict(&input, &cloned_app)).await {
			Ok(ok) => ok,
			Err(err) => {
				app.emit::<ErrorInfo>(
					crate::event::DIALOG_ERROR,
					ErrorInfo {
						title: Cow::Borrowed("Prediction Failed"),
						message: err.to_string(),
					},
				)
				.unwrap();
				crate::restart(app, state);
				return Err(());
			}
		};

	let [high_peak, low_peak] = graph.iter().filter(|each| each.y1.is_some()).fold(
		[Option::<&ComparisonPoint>::None; 2],
		|[last_max, last_min], each| {
			let y1 = each.y1.unwrap();
			let last_max_y1 = last_max.and_then(|found| found.y1).unwrap_or(f64::MIN);
			let last_min_y1 = last_min.and_then(|found| found.y1).unwrap_or(f64::MAX);

			let max = Some(if y1 > last_max_y1 {
				each
			} else {
				last_max.unwrap_or(each)
			});

			let min = Some(if y1 < last_min_y1 {
				each
			} else {
				last_min.unwrap_or(each)
			});

			[max, min]
		},
	);

	let [high_peak, low_peak] = [high_peak.cloned(), low_peak.cloned()];

	let mut guarded_state = match state.lock() {
		Ok(ok) => ok,
		Err(err) => {
			app.emit(crate::event::CORE_PANIC, ()).unwrap();
			app.emit::<ErrorInfo>(
				crate::event::DIALOG_ERROR,
				ErrorInfo {
					title: Cow::Borrowed("State Inaccessible after Prediction"),
					message: err.to_string(),
				},
			)
			.unwrap();
			return Err(());
		}
	};

	// Free-up the memory
	guarded_state.trained_model = None;
	guarded_state.predicted_data = None;

	guarded_state.predicted_data = Some(
		graph
			.iter()
			.filter_map(|each| match each.y1 {
				Some(y1) => Some((each.x, y1)),
				None => None,
			})
			.collect::<Vec<_>>(),
	);

	Ok(EvaluationReport {
		confidence,
		graph,
		high_peak,
		low_peak,
	})
}

#[tauri::command]
pub(crate) async fn save_prediction(
	app: AppHandle,
	state: State<'_, Mutex<crate::typedef::AppState>>,
) -> Result<(), ()> {
	let (tx, mut rx) = tauri::async_runtime::channel::<Option<FilePath>>(1024);

	app.dialog()
		.file()
		.set_title("Save the predicted data")
		.add_filter("Comma-Separated Values (CSV) File", &["csv"])
		.add_filter("Other File Type", &["*"])
		.save_file(move |file_path| {
			let _ = tx.blocking_send(file_path);
		});

	let file_path_option = rx.recv().await;
	if file_path_option
		.as_ref()
		.is_none_or(|found| found.is_none())
	{
		return Ok(());
	}

	let file_path = file_path_option.unwrap().unwrap();
	let source_path = match file_path.into_path() {
		Ok(ok) => ok.with_extension("csv"),
		Err(err) => {
			app.emit::<ErrorInfo>(
				crate::event::DIALOG_ERROR,
				ErrorInfo {
					title: Cow::Borrowed("Cannot Parse File Path"),
					message: err.to_string(),
				},
			)
			.unwrap();
			return Ok(());
		}
	};

	let mut writer = match csv::Writer::from_path(source_path) {
		Ok(ok) => ok,
		Err(err) => {
			app.emit::<ErrorInfo>(
				crate::event::DIALOG_ERROR,
				ErrorInfo {
					title: Cow::Borrowed(ERROR_SAVE),
					message: err.to_string(),
				},
			)
			.unwrap();
			return Err(());
		}
	};

	if let Err(err) = writer.write_record(["Date/Time", "Predicted Value"]) {
		app.emit::<ErrorInfo>(
			crate::event::DIALOG_ERROR,
			ErrorInfo {
				title: Cow::Borrowed(ERROR_SAVE),
				message: err.to_string(),
			},
		)
		.unwrap();
		return Err(());
	}

	let guarded_state = match state.lock() {
		Ok(ok) => ok,
		Err(err) => {
			app.emit(crate::event::CORE_PANIC, ()).unwrap();
			app.emit::<ErrorInfo>(
				crate::event::DIALOG_ERROR,
				ErrorInfo {
					title: Cow::Borrowed("State Inaccessible before Saving"),
					message: err.to_string(),
				},
			)
			.unwrap();
			return Err(());
		}
	};

	let predicted_data = match &guarded_state.predicted_data {
		Some(found) => found,
		None => {
			app.emit::<ErrorInfo>(
				crate::event::DIALOG_ERROR,
				ErrorInfo {
					title: Cow::Borrowed(ERROR_SAVE),
					message: String::from("The predicted data does not exist in memory"),
				},
			)
			.unwrap();
			return Err(());
		}
	};

	for each in predicted_data {
		let datetime = each.0.to_rfc3339();
		let value = each.1.to_string();
		if let Err(err) = writer.write_record([datetime, value]) {
			app.emit::<ErrorInfo>(
				crate::event::DIALOG_ERROR,
				ErrorInfo {
					title: Cow::Borrowed(ERROR_SAVE),
					message: err.to_string(),
				},
			)
			.unwrap();
			return Err(());
		}
	}

	if let Err(err) = writer.flush() {
		app.emit::<ErrorInfo>(
			crate::event::DIALOG_ERROR,
			ErrorInfo {
				title: Cow::Borrowed("Potentially Corrupted"),
				message: err.to_string(),
			},
		)
		.unwrap();
		return Err(());
	}

	Ok(())
}

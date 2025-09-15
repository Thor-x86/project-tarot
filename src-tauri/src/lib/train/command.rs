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

use burn::backend::ndarray::NdArrayDevice;
use burn::backend::Autodiff;
use burn::backend::NdArray;
use burn::module::AutodiffModule;
use burn::prelude::*;
use std::borrow::Cow;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

use crate::typedef::ErrorInfo;
use crate::typedef::NormalParam;

use super::helper::*;
use super::typedef::*;

#[tauri::command]
pub(crate) async fn start_train(
	app: AppHandle,
	state: State<'_, Mutex<crate::typedef::AppState>>,
) -> Result<(), ()> {
	let device = NdArrayDevice::Cpu;

	let random_seed: u64 = rand::random();
	Autodiff::<NdArray>::seed(random_seed);

	let input_result = {
		let mut guarded_state = match state.lock() {
			Ok(ok) => ok,
			Err(err) => {
				app.emit::<ErrorInfo>(
					crate::event::DIALOG_ERROR,
					ErrorInfo {
						title: Cow::Borrowed("State Inaccessible before Training"),
						message: err.to_string(),
					},
				)
				.unwrap();
				app.emit(crate::event::PAGE_MOVE, 1).unwrap();
				return Err(());
			}
		};

		if let None = guarded_state.preprocessed_data {
			guarded_state.page_index = 1;
			app.emit(crate::event::PAGE_MOVE, guarded_state.page_index)
				.unwrap();
			return Err(());
		}

		guarded_state.page_index = 2;
		app.emit(crate::event::PAGE_MOVE, guarded_state.page_index)
			.unwrap();

		let preprocessed_data = guarded_state
			.preprocessed_data
			.as_ref()
			.unwrap()
			.batches
			.clone();

		let cloned_device = device.clone();
		tauri::async_runtime::spawn_blocking(move || {
			send_batches_to_gpu(&preprocessed_data, &cloned_device)
		})
	};

	let input: TrainInput<Autodiff<NdArray>> = match input_result.await {
		Ok(ok) => ok,
		Err(err) => {
			app.emit::<ErrorInfo>(
				crate::event::DIALOG_ERROR,
				ErrorInfo {
					title: Cow::Borrowed("Cannot Use GPU"),
					message: err.to_string(),
				},
			)
			.unwrap();
			crate::restart(app, state);
			return Err(());
		}
	};

	let normal_param = NormalParam {
		mean: input.mean,
		stdev: input.stdev,
	};

	let cloned_app = app.clone();
	let trained_model = match tauri::async_runtime::spawn_blocking(move || {
		train_new_model(input, cloned_app, &device)
	})
	.await
	{
		Ok(ok) => ok,
		Err(err) => {
			app.emit::<ErrorInfo>(
				crate::event::DIALOG_ERROR,
				ErrorInfo {
					title: Cow::Borrowed("Training Failed"),
					message: err.to_string(),
				},
			)
			.unwrap();
			crate::restart(app, state);
			return Err(());
		}
	};

	let mut guarded_state = match state.lock() {
		Ok(ok) => ok,
		Err(err) => {
			app.emit::<()>(crate::event::CORE_PANIC, ()).unwrap();
			app.emit::<ErrorInfo>(
				crate::event::DIALOG_ERROR,
				ErrorInfo {
					title: Cow::Borrowed("State Inaccessible after Training"),
					message: err.to_string(),
				},
			)
			.unwrap();
			return Err(());
		}
	};

	guarded_state.trained_model = Some(trained_model.valid());
	guarded_state.normal_param = Some(normal_param);
	guarded_state.page_index = 3;
	if let Err(err) = app.emit(crate::event::PAGE_MOVE, guarded_state.page_index) {
		app.emit::<ErrorInfo>(
			crate::event::DIALOG_ERROR,
			ErrorInfo {
				title: Cow::Borrowed("Unable to Move Page after Load Data"),
				message: err.to_string(),
			},
		)
		.unwrap();
		*guarded_state = Default::default();
		app.emit(crate::event::PAGE_MOVE, guarded_state.page_index)
			.unwrap();
		return Err(());
	}

	Ok(())
}

#[tauri::command]
pub(crate) fn get_train_progress(
	app: AppHandle,
	state: State<'_, Mutex<crate::typedef::AppState>>,
) -> TrainProgress {
	let guarded_state = match state.lock() {
		Ok(ok) => ok,
		Err(err) => {
			app.emit::<()>(crate::event::CORE_PANIC, ()).unwrap();
			app.emit::<ErrorInfo>(
				crate::event::DIALOG_ERROR,
				ErrorInfo {
					title: Cow::Borrowed("State Inaccessible while Getting The Progress"),
					message: err.to_string(),
				},
			)
			.unwrap();
			return Default::default();
		}
	};

	guarded_state.train_progress.clone()
}

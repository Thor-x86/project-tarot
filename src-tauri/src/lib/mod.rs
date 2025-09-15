#![recursion_limit = "256"]

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

use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

mod typedef;
use typedef::*;

mod event;

mod data;
mod evaluate;
mod preprocess;
mod train;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use data::command::*;
use evaluate::command::*;
use preprocess::command::*;
use train::command::*;

#[tauri::command]
fn get_page_index(state: State<'_, Mutex<AppState>>) -> u8 {
	state.lock().unwrap().page_index
}

#[tauri::command]
fn restart(app: AppHandle, state: State<'_, Mutex<crate::typedef::AppState>>) {
	let mut guarded_state = state.lock().unwrap();
	*guarded_state = Default::default();
	app.emit(crate::event::PAGE_MOVE, guarded_state.page_index)
		.unwrap();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()
		.plugin(tauri_plugin_opener::init())
		.plugin(tauri_plugin_dialog::init())
		.invoke_handler(tauri::generate_handler![
			get_page_index,
			load_data,
			get_data_info,
			select_sheet,
			submit_preprocess_config,
			start_train,
			get_train_progress,
			get_evaluation,
			save_prediction,
			restart
		])
		.setup(|app| {
			app.manage(Mutex::new(AppState::default()));
			Ok(())
		})
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}

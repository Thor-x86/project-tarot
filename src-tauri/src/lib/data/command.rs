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

use std::{borrow::Cow, sync::Mutex};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::{DialogExt, FilePath};

use crate::typedef::{AppState, ErrorInfo};

#[tauri::command]
pub(crate) async fn load_data(app: AppHandle, state: State<'_, Mutex<AppState>>) -> Result<(), ()> {
	let (tx, mut rx) = tauri::async_runtime::channel::<Option<FilePath>>(1024);

	app.dialog()
		.file()
		.set_title("Choose a historical data file")
		.add_filter(
			"Supported Spreadsheet File",
			&["csv", "xlsx", "xlsb", "xls", "ods"],
		)
		.add_filter("Comma-Separated Values (CSV) File", &["csv"])
		.add_filter("Microsoft Excel File", &["xlsx"])
		.add_filter("Microsoft Excel Binary File", &["xlsb"])
		.add_filter("Legacy Microsoft Excel File", &["xls"])
		.add_filter("OpenDocument Spreadsheet (ODS) File", &["ods"])
		.add_filter("Other File Type", &["*"])
		.pick_file(move |file_path| {
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
		Ok(ok) => ok,
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

	let mut guarded_state = match state.lock() {
		Ok(ok) => ok,
		Err(err) => {
			app.emit::<ErrorInfo>(
				crate::event::DIALOG_ERROR,
				ErrorInfo {
					title: Cow::Borrowed("State Inaccessible on Load Data"),
					message: err.to_string(),
				},
			)
			.unwrap();
			return Ok(());
		}
	};

	guarded_state.source_path = Some(source_path);
	guarded_state.page_index = 1;

	if let Err(err) = app.emit(crate::event::PAGE_MOVE, guarded_state.page_index) {
		app.emit::<ErrorInfo>(
			crate::event::DIALOG_ERROR,
			ErrorInfo {
				title: Cow::Borrowed("Unable to Move Page after Load Data"),
				message: err.to_string(),
			},
		)
		.unwrap();
		guarded_state.source_path = None;
	}

	Ok(())
}

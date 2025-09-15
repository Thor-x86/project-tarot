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

use burn::{
	backend::{ndarray::NdArrayDevice, NdArray},
	tensor::Tensor,
};
use chrono::{DateTime, Local};
use tauri::{AppHandle, Emitter};

use super::typedef::*;

pub(super) fn predict(input: &PredictionInput, app: &AppHandle) -> Vec<ComparisonPoint> {
	let device = NdArrayDevice::Cpu;

	// Flatten the batch from 2D array into 1D array, then normalize
	let past_data = input
		.preprocessed_data
		.batches
		.iter()
		.flatten()
		.map(|each| (*each - input.normal_param.mean) / input.normal_param.stdev)
		.collect::<Vec<_>>();

	let past_length = past_data.len();
	let predict_length = (past_length / 2).min(200);
	let predict_offset = past_length - predict_length;

	// We will shift the sequence and modify its end element in this tensor
	let mut tensor = Tensor::<NdArray, 1>::from_data(
		past_data
			.iter()
			.skip(predict_offset)
			.cloned()
			.collect::<Vec<_>>()
			.as_slice(),
		&device,
	)
	.unsqueeze_dims::<3>(&[0, -1]);

	let total_length = past_length + predict_length;

	// Iteration is a handy tool to minimize .clone() and prevent excessive RAM usage
	let past_data_iter = past_data.iter();

	// This is the core process of prediction, matrix operation should only happen in NdArray
	let future_tensors = (0..predict_length)
		.map(|index| {
			let (predicted, _) = input.trained_model.forward(&tensor, None);

			tensor = Tensor::cat(
				[
					tensor.clone().slice([0..1, 1..predict_length, 0..1]),
					predicted.clone().unsqueeze_dim(0),
				]
				.to_vec(),
				1,
			);

			let progress = (index as f64) * 100f64 / (predict_length as f64);
			let _ = app.emit(super::event::PROGRESS, progress);

			predicted.flatten::<1>(0, 1)
		})
		.collect::<Vec<_>>();

	// Gather the result after all computations are done in NdArray
	let future_tensor = Tensor::cat(future_tensors, 0).into_data();
	let future_tensor_iter = future_tensor.iter::<f64>();

	// Denormalize and format it in (index, y0, y1) tuple
	let future_data_iter = (past_length..total_length)
		.zip(future_tensor_iter)
		.map(|(i, y)| {
			let y = (y * input.normal_param.stdev) + input.normal_param.mean;

			(i, Option::<f64>::None, Some(y))
		});

	let x0 = input.preprocessed_data.first_timestamp as i64;
	let x_delta = input.preprocessed_data.timestamp_interval as i64;

	// Same (index, y0, y1) tuple for historical data. Then combine both historical and prediction
	// data iteratively, format it into ComparisonPoint, and send to ReactJS.
	(predict_offset..past_length)
		.zip(past_data_iter)
		.map(|(i, y)| {
			let y = (*y * input.normal_param.stdev) + input.normal_param.mean;

			(i, Some(y), None)
		})
		.chain(future_data_iter)
		.filter_map(|(i, y0, y1)| {
			let x = x0 + (i as i64 * x_delta);
			let x_result = DateTime::from_timestamp(x as i64, 0)?
				.naive_utc()
				.and_local_timezone(Local);
			let x = match x_result {
				chrono::offset::LocalResult::Single(found) => found,
				chrono::offset::LocalResult::Ambiguous(earliest, _) => earliest,
				chrono::offset::LocalResult::None => return None,
			};

			Some(ComparisonPoint { x, y0, y1 })
		})
		.collect::<Vec<_>>()
}

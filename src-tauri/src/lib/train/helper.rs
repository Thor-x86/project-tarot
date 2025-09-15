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

use burn::{
	grad_clipping::GradientClippingConfig,
	module::AutodiffModule,
	nn::loss::{MseLoss, Reduction::Mean},
	optim::{AdamConfig, GradientsParams, Optimizer},
	tensor::{backend::AutodiffBackend, ElementConversion, Tensor},
};
use tauri::{AppHandle, Emitter, Manager};

use crate::typedef::AppState;

use super::model::*;
use super::typedef::*;

pub(super) fn send_batches_to_gpu<B: AutodiffBackend>(
	preprocessed_batches: &Vec<Vec<f64>>,
	device: &B::Device,
) -> TrainInput<B> {
	let batch_count = preprocessed_batches.len();

	// Calculate mean and stdev for normalization, to prevent training from diverging
	let flatten_length = preprocessed_batches.iter().flatten().count();
	let sum = preprocessed_batches
		.iter()
		.flatten()
		.fold(0f64, |last_value, each| last_value + *each);
	let mean = sum / flatten_length as f64;
	let deviations = preprocessed_batches
		.iter()
		.flatten()
		.fold(0f64, |last_value, each| {
			last_value + (*each - mean).powf(2f64)
		});
	let stdev = (deviations / (flatten_length as f64 - 1f64)).sqrt();

	let normalized_batches = preprocessed_batches
		.iter()
		.map(|each_sequence| {
			each_sequence
				.iter()
				.map(|each| (*each - mean) / stdev)
				.collect::<Vec<_>>()
		})
		.collect::<Vec<_>>();

	let mut all_targets = Vec::<Tensor<B, 1>>::with_capacity(batch_count);
	let all_tensors = normalized_batches
		.iter()
		.map(|each_batch| {
			let mut tensors = each_batch
				.iter()
				.map(|y| Tensor::<B, 1>::from_floats([*y], device))
				.collect::<Vec<_>>();

			let last_tensor = tensors
				.pop()
				.unwrap_or(Tensor::<B, 1>::from_floats([0], device));
			all_targets.push(last_tensor);

			Tensor::stack::<2>(tensors, 0)
		})
		.collect::<Vec<_>>();

	// Split the tensors
	let slice_line = (batch_count * 8 / 10).min(1).max(batch_count - 1);
	let train_batches = all_tensors[0..slice_line].to_vec();
	let train_targets = all_targets[0..slice_line].to_vec();
	let valid_batches = all_tensors[slice_line..batch_count]
		.iter()
		.map(|each| each.valid())
		.collect::<Vec<_>>();
	let valid_targets = all_targets[slice_line..batch_count]
		.iter()
		.map(|each| each.valid())
		.collect::<Vec<_>>();

	let train_tensor: Tensor<B, 3> = Tensor::stack(train_batches, 0);
	let train_target_tensor: Tensor<B, 2> = Tensor::stack(train_targets, 0);
	let valid_tensor: Tensor<B::InnerBackend, 3> = Tensor::stack(valid_batches, 0);
	let valid_target_tensor: Tensor<B::InnerBackend, 2> = Tensor::stack(valid_targets, 0);

	TrainInput {
		train_tensor,
		train_target_tensor,
		valid_tensor,
		valid_target_tensor,
		mean,
		stdev,
	}
}

pub(super) fn train_new_model<B: AutodiffBackend>(
	input: TrainInput<B>,
	app: AppHandle,
	device: &B::Device,
) -> LstmNetwork<B> {
	let config = TrainingConfig::new(
		LstmNetworkConfig::new(),
		// Gradient clipping via optimizer config
		AdamConfig::new().with_grad_clipping(Some(GradientClippingConfig::Norm(1.0))),
	);

	let valid_num_items = input.valid_tensor.dims()[0];
	let mut model = config.model.init::<B>(device);
	let mut optim = config.optimizer.init::<B, LstmNetwork<B>>();
	let mut max_valid_loss = 0f32;

	// We do 500 epochs of training because it is guaranteed to give best result
	for epoch in 1u32..=500u32 {
		// Initialize the training and validation metrics at the start of each epoch
		let mut valid_loss = 0f32;

		// Training phase
		{
			let output = model.forward(&input.train_tensor, None).0;
			let loss = MseLoss::new().forward(output, input.train_target_tensor.clone(), Mean);

			// Gradients for the current backward pass
			let grads = loss.backward();
			// Gradients linked to each parameter of the model
			let grads = GradientsParams::from_grads(grads, &model);
			// Update the model using the optimizer
			model = optim.step(config.lr, model, grads);
		}

		// Validation phase
		{
			let model = model.valid();
			let output = model.forward(&input.valid_tensor, None).0;
			let loss = MseLoss::new().forward(output, input.valid_target_tensor.clone(), Mean);
			valid_loss += loss.clone().into_scalar().elem::<f32>()
				* input.valid_target_tensor.dims()[0] as f32;
		}

		// The averaged train loss per epoch
		let avg_valid_loss = valid_loss / valid_num_items as f32;
		max_valid_loss = max_valid_loss.max(avg_valid_loss);

		// Display the averaged validation metrics
		{
			let new_point = IterativePoint {
				x: epoch,
				y: 100f32 - (avg_valid_loss * 100f32 / max_valid_loss),
			};

			let _ = app.emit(super::event::PROGRESS_NEW, new_point.clone());

			let state = app.state::<Mutex<AppState>>();
			if let Ok(mut guarded_state) = state.lock() {
				guarded_state
					.train_progress
					.confidence_points
					.push(new_point);
			};

			if new_point.y > 98f32 && epoch >= 250u32 {
				break;
			}
		}
	}

	model
}

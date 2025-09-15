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
	optim::AdamConfig,
	tensor::{backend::AutodiffBackend, Tensor},
};
use serde::Serialize;

use super::model::*;

#[derive(Default, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IterativePoint {
	pub x: u32,
	pub y: f32,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TrainProgress {
	pub confidence_points: Vec<IterativePoint>,
	pub end_x: u32,
}

impl Default for TrainProgress {
	fn default() -> Self {
		Self {
			confidence_points: Default::default(),
			end_x: 500,
		}
	}
}

#[derive(burn::config::Config)]
pub(super) struct TrainingConfig {
	pub model: LstmNetworkConfig,
	pub optimizer: AdamConfig,

	#[config(default = 1e-3)]
	pub lr: f64,
}

pub(super) struct TrainInput<B: AutodiffBackend> {
	pub train_tensor: Tensor<B, 3>,
	pub train_target_tensor: Tensor<B, 2>,
	pub valid_tensor: Tensor<B::InnerBackend, 3>,
	pub valid_target_tensor: Tensor<B::InnerBackend, 2>,
	pub mean: f64,
	pub stdev: f64,
}

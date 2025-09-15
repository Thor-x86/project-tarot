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

import type { BoxProps } from '@mui/material/Box';

import { useState, useReducer, useMemo, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

import Box from '@mui/material/Box';
import CircularProgress from '@mui/material/CircularProgress';
import Typography from '@mui/material/Typography';
import Skeleton from '@mui/material/Skeleton';

import { LineChart } from '@mui/x-charts/LineChart';

type Point = {
	x: number;
	y: number;
};

type ConfidenceGraphAction =
	| { type: 'set'; value: Point[] }
	| { type: 'add'; value: Point }
	| { type: 'clear' };

interface TrainProgress {
	endX: number;
	confidencePoints: Point[];
}

export default function TrainPage(props: BoxProps) {
	const [endX, setEndX] = useState(0);

	const [confidencePoints, modifyConfidenceGraph] = useReducer(
		(state: Point[], action: ConfidenceGraphAction) => {
			switch (action.type) {
				case 'add':
					if (state.length === 1 && state[0].x === 0 && action.value.x === 0)
						return [action.value];
					else return [...state, action.value];
				// @ts-ignore 7029: intended to fallthrough
				case 'set':
					if (action.value.length > 0) return action.value;
				case 'clear':
					return [{ x: 0, y: 0 }];
			}
		},
		[{ x: 0, y: 0 }]
	);

	const pageDisabled = useMemo(
		() => typeof props.tabIndex === 'number' && props.tabIndex < 0,
		[props.tabIndex]
	);

	const confidenceGraph = useMemo(
		() =>
			pageDisabled ? (
				<Skeleton sx={{ width: '100%', height: '100%' }} />
			) : (
				<LineChart
					sx={{ width: '100%', height: '100%', minHeight: '0' }}
					xAxis={[{ dataKey: 'x', min: 0, max: endX, label: 'Training Epoch' }]}
					yAxis={[{ min: 0, max: 100, label: 'Confidence (%)' }]}
					series={[{ dataKey: 'y', area: true, showMark: false }]}
					fillOpacity="0.5"
					grid={{ horizontal: true, vertical: true }}
					skipAnimation
					dataset={confidencePoints}
				/>
			),
		[pageDisabled, confidencePoints, endX]
	);

	const latestConfidence = useMemo<number>(
		() =>
			confidencePoints.length > 0
				? Math.round((confidencePoints[confidencePoints.length - 1]?.y ?? 0) * 100) / 100
				: 0,
		[confidencePoints.length]
	);

	useEffect(() => {
		if (pageDisabled) {
			setEndX(0);
			modifyConfidenceGraph({ type: 'clear' });
			return;
		}

		let isCanceled = false;

		invoke<TrainProgress>('get_train_progress').then((value) => {
			if (isCanceled) return;
			setEndX(value.endX);
			modifyConfidenceGraph({ type: 'set', value: value.confidencePoints });
		});

		let unlistener = listen<Point>('App://train/progress/new', (e) => {
			modifyConfidenceGraph({ type: 'add', value: e.payload });
		});

		return () => {
			isCanceled = true;
			unlistener.then((unlisten) => unlisten());
		};
	}, [pageDisabled, setEndX, modifyConfidenceGraph]);

	return (
		<Box
			{...props}
			component="section"
			sx={{
				display: 'flex',
				flexDirection: 'column',
				justifyContent: 'start',
				alignItems: 'stretch',
				padding: '16px',
				gap: '16px'
			}}
		>
			<Box
				sx={{
					display: 'flex',
					flexDirection: 'row',
					justifyContent: 'center',
					alignItems: 'center',
					gap: '16px'
				}}
			>
				<CircularProgress variant="indeterminate" size={32} />
				<Typography variant="h6" component="h1">
					Understanding historical data...
				</Typography>
			</Box>
			<Box sx={{ flex: 1, overflow: 'hidden' }}>{confidenceGraph}</Box>
			<Typography variant="body1" sx={{ textAlign: 'center' }}>
				{latestConfidence ? (
					<>
						Prediction confidence: <strong>{latestConfidence.toString()}%</strong>
					</>
				) : (
					<>☕ Brew coffee and enjoy... This will take quite amount of time</>
				)}
			</Typography>
		</Box>
	);
}

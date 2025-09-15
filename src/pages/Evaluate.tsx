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

import { useState, useMemo, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

import Box from '@mui/material/Box';
import CircularProgress from '@mui/material/CircularProgress';
import Typography from '@mui/material/Typography';
import Fade from '@mui/material/Fade';
import Button from '@mui/material/Button';

import { LineChart } from '@mui/x-charts/LineChart';

import SaveIcon from '@mui/icons-material/Save';
import RestartIcon from '@mui/icons-material/RotateLeft';

type RawComparisonPoint = {
	x: string;
	y0: number;
	y1: number;
};

interface EvaluationReport {
	confidence: number;
	graph: RawComparisonPoint[];
	highPeak?: RawComparisonPoint;
	lowPeak?: RawComparisonPoint;
}

export default function EvaluatePage(props: BoxProps) {
	const [report, setReport] = useState<EvaluationReport | null>(null);
	const [isSaving, setSaving] = useState(false);
	const [predictProgress, setPredictProgress] = useState(0);

	const pageDisabled = useMemo(
		() => typeof props.tabIndex === 'number' && props.tabIndex < 0,
		[props.tabIndex]
	);

	const confidence = useMemo(
		() => Math.round((report?.confidence ?? 0) * 100) / 100,
		[report?.confidence]
	);

	const dataset = useMemo(
		() =>
			report?.graph.map((each) => ({
				...each,
				x: new Date(each.x)
			})),
		[report?.graph]
	);

	const formatter = useMemo(
		() =>
			new Intl.DateTimeFormat('en-US', {
				weekday: 'long',
				year: 'numeric',
				month: 'long',
				day: 'numeric',
				hour: '2-digit',
				minute: '2-digit',
				second: '2-digit',
				hour12: false
			}),
		[]
	);

	const highPeak = useMemo(
		() => (report?.highPeak ? formatter.format(new Date(report.highPeak.x)) : undefined),
		[report?.highPeak]
	);

	const lowPeak = useMemo(
		() => (report?.lowPeak ? formatter.format(new Date(report.lowPeak.x)) : undefined),
		[report?.lowPeak]
	);

	const isPredicting = pageDisabled || report === null;
	const isBusy = isPredicting || isSaving;

	useEffect(() => {
		setReport(null);
		setPredictProgress(0);
		if (pageDisabled) return;

		let unlistenProgress = listen<number>('App://evaluate/progress', (event) =>
			setPredictProgress(Math.round(event.payload * 100) / 100)
		);

		let isCanceled = false;
		invoke<EvaluationReport>('get_evaluation').then((value) => {
			if (isCanceled) return;
			setReport(value);
		});

		return () => {
			unlistenProgress.then((unlisten) => unlisten());
			isCanceled = true;
		};
	}, [pageDisabled, setReport, setPredictProgress]);

	const handleSave = useCallback(() => {
		setSaving(true);
		invoke<void>('save_prediction').finally(() => {
			setSaving(false);
		});
	}, [setSaving]);

	function handleRestart() {
		invoke<void>('restart');
	}

	if (isPredicting)
		return (
			<Box
				{...props}
				component="section"
				sx={{
					display: 'flex',
					flexDirection: 'row',
					justifyContent: 'center',
					alignItems: 'center',
					padding: '16px',
					gap: '16px'
				}}
			>
				<CircularProgress variant="indeterminate" size={32} />
				<Typography variant="h6" component="h1">
					Predicting the future{predictProgress ? ` (${predictProgress}%)` : '...'}
				</Typography>
			</Box>
		);

	return (
		<Fade timeout={500} in>
			<Box
				{...props}
				component="section"
				sx={{
					display: 'flex',
					flexDirection: 'column',
					justifyContent: 'center',
					alignItems: 'center',
					padding: '16px',
					textAlign: 'center'
				}}
			>
				<LineChart
					sx={{ width: '100%', height: '100%', minHeight: '0' }}
					xAxis={[
						{
							dataKey: 'x',
							valueFormatter: (value: number) => {
								return formatter.format(new Date(value));
							}
						}
					]}
					series={[
						{ dataKey: 'y0', label: 'Historical Data', showMark: false },
						{ dataKey: 'y1', label: 'Predicted', showMark: false }
					]}
					skipAnimation
					grid={{ horizontal: true, vertical: false }}
					{...{ dataset }}
				/>
				<Typography sx={{ mb: '16px' }}>
					Within <strong>{confidence}% confidence</strong>
					{highPeak || lowPeak ? ', I predicted ' : ''}
					{highPeak ? (
						<>
							the next <strong>highest peak</strong> is on <strong>{highPeak}</strong>
						</>
					) : (
						''
					)}
					{highPeak && lowPeak ? ' and ' : ''}
					{lowPeak ? (
						<>
							the next <strong>lowest peak</strong> is on <strong>{lowPeak}</strong>
						</>
					) : (
						''
					)}
				</Typography>
				<Box
					sx={{
						display: 'flex',
						flexDirection: 'row',
						justifyContent: 'center',
						alignItems: 'center',
						gap: '16px'
					}}
				>
					<Button
						variant="contained"
						color="secondary"
						startIcon={<SaveIcon />}
						onClick={handleSave}
						disabled={isBusy}
						loading={isSaving}
					>
						Save
					</Button>
					<Button
						variant="outlined"
						color="secondary"
						startIcon={<RestartIcon />}
						onClick={handleRestart}
						disabled={isBusy}
					>
						Restart
					</Button>
				</Box>
			</Box>
		</Fade>
	);
}

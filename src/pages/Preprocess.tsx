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
import type { SxProps, Theme } from '@mui/material/styles';
import type { GridApi } from '@mui/x-data-grid';
import type { SelectChangeEvent } from '@mui/material/Select';

import { useState, useRef, useMemo, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import Collapse from '@mui/material/Collapse';
import FormControl from '@mui/material/FormControl';
import InputLabel from '@mui/material/InputLabel';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import Skeleton from '@mui/material/Skeleton';

import { DataGrid } from '@mui/x-data-grid';

import NextIcon from '@mui/icons-material/ArrowForward';

type ColumnType = 'string' | 'number' | 'dateTime' | 'boolean';

interface ColumnInfo {
	field: string;
	headerName: string;
	type: ColumnType;
}

type RowInfo = Record<string, string | number | Date | boolean>;

type BatchPeriode = 'Minutely' | 'Hourly' | 'Daily' | 'Weekly' | 'Monthly' | 'Yearly';

interface RowSelection {
	type: 'include' | 'exclude';
	ids: number[];
}

interface SheetInfo {
	tabName?: string;
	columns: ColumnInfo[];
	rows: RowInfo[];
	allowedBatchPeriodes: BatchPeriode[];
	selectedDatetimeColumn: string;
	selectedPredictableColumn: string;
	selectedBatchPeriode: BatchPeriode;
	rowSelection: RowSelection;
}

interface DataInfo {
	name: string;
	tabs?: string[];
	sheetInfo: SheetInfo;
}

interface PreprocessConfig {
	tabName?: string;
	datetimeColumn: string;
	predictableColumn: string;
	batchPeriode: BatchPeriode;
	rowSelection: RowSelection;
}

export default function PreprocessPage(props: BoxProps) {
	const defaultSx: SxProps<Theme> = { my: '8px' };
	const inputLabelSx: SxProps<Theme> = { backgroundColor: 'white', px: '8px' };

	const [isLoading, setLoading] = useState(false);
	const [isSubmitting, setSubmitting] = useState(false);
	const [name, setName] = useState<string>('');
	const [tabs, setTabs] = useState<string[]>([]);
	const [columns, setColumns] = useState<ColumnInfo[]>([]);
	const [rows, setRows] = useState<RowInfo[]>([]);
	const [allowedBatchPeriodes, setAllowedBatchPeriodes] = useState<BatchPeriode[]>([]);
	const [selectedTab, setSelectedTab] = useState<string>('');
	const [selectedDatetime, setSelectedDatetime] = useState<string>('');
	const [selectedPredictable, setSelectedPredictable] = useState<string>('');
	const [selectedBatchPeriode, setSelectedBatchPeriode] = useState<BatchPeriode | ''>('');
	const [rowSelection, setRowSelection] = useState<RowSelection>({
		type: 'exclude',
		ids: []
	});

	const gridApiRef = useRef<GridApi>(null);

	const pageDisabled = useMemo(
		() => typeof props.tabIndex === 'number' && props.tabIndex < 0,
		[props.tabIndex]
	);

	const disabled = useMemo(
		() => pageDisabled || isLoading || isSubmitting,
		[pageDisabled, isLoading, isSubmitting]
	);

	const allowSubmit = useMemo(
		() =>
			!disabled &&
			selectedDatetime &&
			selectedPredictable &&
			selectedBatchPeriode &&
			((rowSelection.type === 'include' && rowSelection.ids.length !== 0) ||
				(rowSelection.type === 'exclude' && rowSelection.ids.length !== rows.length)),
		[
			disabled,
			selectedDatetime,
			selectedPredictable,
			selectedBatchPeriode,
			rowSelection,
			rows.length
		]
	);

	const tabSelections = useMemo(
		() =>
			pageDisabled
				? []
				: tabs.map((each) => (
						<MenuItem key={each} value={each}>
							{each}
						</MenuItem>
				  )),
		[pageDisabled, tabs]
	);

	const datetimeSelections = useMemo(
		() =>
			disabled
				? []
				: columns
						.filter((each) => each.type === 'dateTime')
						.map((each) => (
							<MenuItem key={each.field} value={each.field}>
								{each.headerName ?? each.field}
							</MenuItem>
						)),
		[disabled, columns]
	);

	const predictableSelections = useMemo(
		() =>
			disabled
				? []
				: columns
						.filter((each) => each.type === 'number')
						.map((each) => (
							<MenuItem key={each.field} value={each.field}>
								{each.headerName ?? each.field}
							</MenuItem>
						)),
		[disabled, columns]
	);

	const batchPeriode = useMemo(
		() =>
			disabled
				? []
				: allowedBatchPeriodes.map((each) => (
						<MenuItem key={each} value={each}>
							{each}
						</MenuItem>
				  )),
		[disabled, allowedBatchPeriodes]
	);

	const rowSelector = useMemo(
		() =>
			isLoading || pageDisabled ? (
				<Box
					sx={{
						flex: 1,
						width: '100%',
						display: 'flex',
						flexDirection: 'column',
						justifyContent: 'start',
						alignItems: 'stretch',
						padding: '16px',
						gap: '16px',
						overflowX: 'hidden',
						overflowY: 'auto'
					}}
				>
					{new Array<number>(5).fill(0).map((_, index) => (
						<Skeleton key={index} variant="text" />
					))}
				</Box>
			) : (
				// disableVirtualization is needed because its buggy on most WebView
				<DataGrid
					sx={{
						flex: 1,
						width: '100%',
						pointerEvents: isSubmitting ? 'none' : undefined,
						opacity: isSubmitting ? 0.5 : undefined
					}}
					checkboxSelection
					disableVirtualization
					apiRef={gridApiRef}
					onRowSelectionModelChange={({ type, ids }) => {
						setRowSelection({
							type,
							ids: Array.from(ids).filter((each) => typeof each === 'number')
						});
					}}
					{...{ columns, rows }}
				/>
			),
		[isLoading, pageDisabled, isSubmitting, columns, rows]
	);

	const handleSelectTab = useCallback(
		(event: SelectChangeEvent<string>) => {
			if (disabled) return;
			let value = event.target.value;
			setSelectedTab(value);

			let isCanceled = false;
			setLoading(true);
			invoke<SheetInfo>('select_sheet', { tabName: value }).then((sheetInfo) => {
				if (isCanceled) return;
				changeSheetInfo(sheetInfo);
				setLoading(false);
			});

			return () => {
				isCanceled = true;
				setLoading(false);
			};
		},
		[disabled, setSelectedTab]
	);

	function changeSheetInfo(sheetInfo: SheetInfo) {
		setColumns(sheetInfo.columns);
		setRows(
			sheetInfo.rows.map((each) => {
				let datetimeKeys = Object.keys(each).filter((eachKey) =>
					sheetInfo.columns
						.filter((eachCol) => eachCol.type === 'dateTime')
						.map((eachCol) => eachCol.field)
						.includes(eachKey)
				);

				let overrideObj: RowInfo = {};
				for (const eachKey of datetimeKeys) {
					if (typeof each[eachKey] !== 'string') continue;
					overrideObj[eachKey] = new Date(each[eachKey]);
				}

				return { ...each, ...overrideObj };
			})
		);
		setAllowedBatchPeriodes(sheetInfo.allowedBatchPeriodes);
		setSelectedTab(sheetInfo.tabName ?? '');
		setSelectedDatetime(sheetInfo.selectedDatetimeColumn);
		setSelectedPredictable(sheetInfo.selectedPredictableColumn);
		setSelectedBatchPeriode(sheetInfo.selectedBatchPeriode);
		setRowSelection(sheetInfo.rowSelection);
	}

	useEffect(() => {
		setLoading(true);
		setName('');
		setTabs([]);
		setColumns([]);
		setRows([]); // Save memory on exit page
		if (pageDisabled) return;

		let isCanceled = false;
		let timeoutHandler = NaN;

		new Promise((resolve) => {
			// Fix page stuck in some platforms
			timeoutHandler = setTimeout(resolve, 1000);
		}).then(() => {
			if (isCanceled) return;
			timeoutHandler = NaN;
			return invoke<DataInfo>('get_data_info').then((value) => {
				if (isCanceled) return;
				setName(value.name);
				setTabs(value.tabs ?? []);
				changeSheetInfo(value.sheetInfo);
				setLoading(false);
			});
		});

		return () => {
			isCanceled = true;
			if (!isNaN(timeoutHandler)) clearTimeout(timeoutHandler);
		};
	}, [
		pageDisabled,
		setName,
		setColumns,
		setRows,
		setAllowedBatchPeriodes,
		setSelectedDatetime,
		setSelectedPredictable,
		setSelectedBatchPeriode,
		setRowSelection,
		setLoading
	]);

	useEffect(() => {
		if (!gridApiRef.current) return;
		if (disabled) return;
		const gridApi = gridApiRef.current;
		gridApi.autosizeColumns().then(() => {
			return gridApi.autosizeColumns();
		});
	}, [gridApiRef.current, disabled, columns]);

	useEffect(() => {
		if (!gridApiRef.current) return;
		if (disabled) return;
		const gridApi = gridApiRef.current;
		gridApi.setRowSelectionModel({ ids: new Set(rowSelection.ids), type: rowSelection.type });
	}, [gridApiRef.current, disabled, rows]);

	function handleClickChange() {
		invoke('restart');
	}

	function handleClickStart() {
		if (!selectedBatchPeriode) return;
		setSubmitting(true);

		let config: PreprocessConfig = {
			tabName: selectedTab || undefined,
			datetimeColumn: selectedDatetime,
			predictableColumn: selectedPredictable,
			batchPeriode: selectedBatchPeriode,
			rowSelection
		};

		invoke<void>('submit_preprocess_config', { config })
			.finally(() => {
				setSubmitting(false);
			})
			.then(() => invoke<void>('start_train'));
	}

	return (
		<Box
			{...props}
			component="section"
			sx={{
				display: 'flex',
				flexDirection: 'row',
				justifyContent: 'start',
				alignItems: 'stretch'
			}}
		>
			<Box
				sx={{
					width: '45%',
					maxWidth: '360px',
					borderInlineEnd: 'solid 1px rgba(127,127,127,0.5)',
					padding: '16px',
					overflowX: 'hidden',
					overflowY: 'auto'
				}}
			>
				<Typography variant="h6" component="p">
					Selected file:
				</Typography>
				{name ? (
					<Typography variant="body2">{name}</Typography>
				) : (
					<Skeleton variant="text" />
				)}
				<Button
					variant="outlined"
					sx={defaultSx}
					onClick={handleClickChange}
					{...{ disabled }}
				>
					Change
				</Button>
				<Collapse timeout={1000} in={!disabled && tabs.length > 0} unmountOnExit>
					<Divider sx={defaultSx} />
					<FormControl fullWidth sx={defaultSx} {...{ disabled }}>
						<InputLabel id="page-preprocess-label-tab" sx={inputLabelSx}>
							Spreadsheet Tab
						</InputLabel>
						<Select
							labelId="page-preprocess-label-tab"
							value={pageDisabled ? '' : selectedTab}
							onChange={handleSelectTab}
						>
							{tabSelections}
						</Select>
					</FormControl>
				</Collapse>
				<Divider sx={defaultSx} />
				<FormControl fullWidth sx={defaultSx} {...{ disabled }}>
					<InputLabel id="page-preprocess-label-time" sx={inputLabelSx}>
						Date/Time Column
					</InputLabel>
					<Select
						labelId="page-preprocess-label-time"
						value={disabled ? '' : selectedDatetime}
						onChange={(e) => {
							setSelectedDatetime(e.target.value);
						}}
					>
						{datetimeSelections}
					</Select>
				</FormControl>
				<FormControl fullWidth sx={defaultSx} {...{ disabled }}>
					<InputLabel id="page-preprocess-label-value" sx={inputLabelSx}>
						Column to Predict
					</InputLabel>
					<Select
						labelId="page-preprocess-label-value"
						value={disabled ? '' : selectedPredictable}
						onChange={(e) => {
							setSelectedPredictable(e.target.value);
						}}
					>
						{predictableSelections}
					</Select>
				</FormControl>
				<FormControl fullWidth sx={defaultSx} {...{ disabled }}>
					<InputLabel id="page-preprocess-label-batch" sx={inputLabelSx}>
						Most Patterns Recurring...
					</InputLabel>
					<Select
						labelId="page-preprocess-label-batch"
						value={disabled ? '' : selectedBatchPeriode}
						onChange={(e) => {
							setSelectedBatchPeriode(e.target.value as BatchPeriode);
						}}
					>
						{batchPeriode}
					</Select>
				</FormControl>
			</Box>
			<Box
				sx={{
					width: '55%',
					maxWidth: 'calc(100%-360px)',
					padding: '16px',
					gap: '16px',
					display: 'flex',
					flex: 1,
					flexDirection: 'column',
					justifyContent: 'start',
					alignItems: 'stretch'
				}}
			>
				<Typography variant="body1">Select rows:</Typography>
				{rowSelector}
				<Divider />
				<Button
					variant="contained"
					color="secondary"
					endIcon={<NextIcon />}
					sx={{ alignSelf: 'end' }}
					disabled={!allowSubmit}
					loading={isSubmitting}
					onClick={handleClickStart}
				>
					Start
				</Button>
			</Box>
		</Box>
	);
}

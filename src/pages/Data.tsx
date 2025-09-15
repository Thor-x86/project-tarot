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

import { useState, useMemo, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';

import spadeIcon from '../assets/cards.svg';

export default function DataPage(props: BoxProps) {
	const [loading, setLoading] = useState(false);

	const disabled = useMemo(
		() => (typeof props.tabIndex === 'number' && props.tabIndex < 0) || loading,
		[props.tabIndex, loading]
	);

	const handleClickLoad = useCallback(() => {
		setLoading(true);
		invoke<void>('load_data').finally(() => {
			setLoading(false);
		});
	}, [setLoading]);

	return (
		<Box
			{...props}
			component="section"
			sx={{
				display: 'flex',
				flexDirection: 'column',
				justifyContent: 'center',
				alignItems: 'center',
				gap: '32px'
			}}
		>
			<Box
				component="img"
				src={spadeIcon}
				alt=""
				role="presentation"
				sx={{ width: '96px', height: '96px', opacity: 0.5 }}
				draggable={false}
			/>
			<Typography component="p" variant="h5">
				Let's start by understanding past historical data
			</Typography>
			<Button
				variant="contained"
				color="secondary"
				onClick={handleClickLoad}
				{...{ disabled, loading }}
			>
				Load File
			</Button>
		</Box>
	);
}

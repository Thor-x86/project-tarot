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

import { useState, useMemo, useEffect } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';

import AppBar from '@mui/material/AppBar';
import Toolbar from '@mui/material/Toolbar';
import Typography from '@mui/material/Typography';
import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';

import InfoIcon from '@mui/icons-material/InfoOutline';
import MinimizeIcon from '@mui/icons-material/Minimize';
import MaximizeIcon from '@mui/icons-material/CropDin';
import RestoreIcon from '@mui/icons-material/FilterNone';
import CloseIcon from '@mui/icons-material/Close';

export default function Titlebar() {
	const [isMaximized, setMaximized] = useState(false);
	const currentWindow = useMemo(getCurrentWindow, []);

	useEffect(() => {
		currentWindow.listen('tauri://resize', () => {
			currentWindow.isMaximized().then(setMaximized);
		});
	}, [currentWindow, setMaximized]);

	return (
		<AppBar position="static" sx={{ w: '100%' }}>
			<Toolbar variant="dense" sx={{ pl: '16px' }} disableGutters>
				<Typography
					variant="h6"
					component="div"
					sx={{ flexGrow: 1, pr: 'auto', userSelect: 'none' }}
					data-tauri-drag-region
				>
					Project::Tarot
				</Typography>
				<Tooltip title="Made with ❤️ by Athaariq A. R." arrow>
					<IconButton
						color="inherit"
						href="https://www.linkedin.com/in/athaariq-ardhiansyah/"
						target="_blank"
					>
						<InfoIcon />
					</IconButton>
				</Tooltip>
				<IconButton
					onClick={currentWindow.minimize.bind(globalThis)}
					color="inherit"
					title="Minimize"
				>
					<MinimizeIcon />
				</IconButton>
				<IconButton
					onClick={currentWindow.toggleMaximize.bind(globalThis)}
					color="inherit"
					title={isMaximized ? 'Restore' : 'Maximize'}
				>
					{isMaximized ? <RestoreIcon sx={{ p: '2px' }} /> : <MaximizeIcon />}
				</IconButton>
				<IconButton
					onClick={currentWindow.close.bind(globalThis)}
					color="inherit"
					title="Close"
				>
					<CloseIcon />
				</IconButton>
			</Toolbar>
		</AppBar>
	);
}

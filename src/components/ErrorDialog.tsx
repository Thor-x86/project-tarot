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

import { useState, useEffect } from 'react';

import Dialog from '@mui/material/Dialog';
import DialogTitle from '@mui/material/DialogTitle';
import DialogContent from '@mui/material/DialogContent';
import DialogContentText from '@mui/material/DialogContentText';
import DialogActions from '@mui/material/DialogActions';
import Button from '@mui/material/Button';

export interface ErrorInfo {
	title: string;
	message: string;
}

export interface ErrorDialogProps {
	info: ErrorInfo | null;
	onClose: () => void;
}

export default function ErrorDialog({ info, onClose }: ErrorDialogProps) {
	const [open, setOpen] = useState(false);
	const [title, setTitle] = useState('');
	const [message, setMessage] = useState('');

	useEffect(() => {
		setOpen(info != null);
		if (info == null) return;

		setTitle(info.title);
		setMessage(info.message);
	}, [info, setOpen, setTitle, setMessage]);

	return (
		<Dialog {...{ open, onClose }}>
			<DialogTitle>{title}</DialogTitle>
			<DialogContent>
				<DialogContentText
					component="pre"
					sx={
						title.includes('\n')
							? undefined
							: { wordWrap: 'normal', whiteSpace: 'break-spaces' }
					}
				>
					{message}
				</DialogContentText>
			</DialogContent>
			<DialogActions>
				<Button variant="outlined" color="secondary" onClick={onClose} autoFocus>
					OK
				</Button>
			</DialogActions>
		</Dialog>
	);
}

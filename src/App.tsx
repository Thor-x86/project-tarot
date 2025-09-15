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

import type { ErrorInfo } from './components/ErrorDialog';

import { useRef, useState, useReducer, useMemo, useEffect } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

import Box from '@mui/material/Box';
import Stepper from '@mui/material/Stepper';
import Step from '@mui/material/Step';
import StepLabel from '@mui/material/StepLabel';

import Titlebar from './components/Titlebar';
import ErrorDialog from './components/ErrorDialog';

import DataPage from './pages/Data';
import PreprocessPage from './pages/Preprocess';
import TrainPage from './pages/Train';
import EvaluatePage from './pages/Evaluate';

type ErrorAction = { type: 'add'; value: ErrorInfo } | { type: 'remove' };

export default function App() {
	const pages: (typeof Box)[] = [DataPage, PreprocessPage, TrainPage, EvaluatePage];
	const steps: string[] = ['Data', 'Preprocess', 'Training', 'Evaluate'];

	const pageViewRef = useRef<HTMLElement>(undefined);

	const [pageIndex, setPageIndex] = useState(0);
	const [scrollableClass, setScrollableClass] = useState<undefined | string>(undefined);
	const [isPanic, setPanic] = useState<boolean>(false);

	const [allErrorInfo, modifyErrorInfo] = useReducer(
		(state: ErrorInfo[], action: ErrorAction) => {
			switch (action.type) {
				case 'add':
					if (state.length >= 10) return state; // Prevent accidental message spamming
					return [...state, action.value];
				case 'remove':
					state.shift();
					return [...state];
				default:
					return state;
			}
		},
		[]
	);

	const currentWindow = useMemo(getCurrentWindow, []);

	useEffect(() => {
		let isCanceled = false;

		invoke<number>('get_page_index').then((value) => {
			if (isCanceled) return;
			setPageIndex(value);
		});

		let unlistenPageMove = listen<number>('App://page/move', (event) =>
			setPageIndex(event.payload)
		);

		let unlistenError = listen<ErrorInfo>('App://dialog/error', (event) => {
			modifyErrorInfo({ type: 'add', value: event.payload });
		});

		let unlistenPanic = listen<void>('App://core/panic', () => {
			setPanic(true);
		});

		return () => {
			isCanceled = true;
			unlistenPageMove.then((unlisten) => unlisten());
			unlistenError.then((unlisten) => unlisten());
			unlistenPanic.then((unlisten) => unlisten());
		};
	}, [setPageIndex, modifyErrorInfo, setPanic]);

	useEffect(() => {
		if (!pageViewRef?.current) return;
		const pageView = pageViewRef.current;

		function workaround() {
			pageView.scrollTo({ left: pageView.clientWidth * pageIndex, behavior: 'instant' });
		}

		window.addEventListener('resize', workaround);
		setScrollableClass('scrolling');
		pageView.scrollTo(pageView.clientWidth * pageIndex, 0);

		let watchdogHandler = NaN;

		let timeoutHandler = setTimeout(() => {
			setScrollableClass(undefined);
			watchdogHandler = setInterval(() => {
				pageView.scrollTo({
					left: pageView.clientWidth * pageIndex,
					behavior: 'instant'
				});
			}, 1000);
		}, 500);

		return () => {
			clearTimeout(timeoutHandler);
			if (!isNaN(watchdogHandler)) clearInterval(watchdogHandler);
			window.removeEventListener('resize', workaround);
		};
	}, [pageIndex, pageViewRef?.current, setScrollableClass]);

	function handleCloseDialog() {
		modifyErrorInfo({ type: 'remove' });
		if (isPanic && allErrorInfo.length <= 1) {
			currentWindow.close();
		}
	}

	return (
		<>
			<Titlebar />
			<Box
				component="main"
				className={scrollableClass}
				ref={pageViewRef}
				sx={{
					width: '100%',
					height: '100%',
					display: 'flex',
					flexDirection: 'row',
					justifyContent: 'flex-start',
					alignItems: 'stretch',
					overflow: 'hidden',
					scrollBehavior: 'smooth',
					scrollSnapType: 'x mandatory',
					'&.scrolling': {
						scrollSnapType: 'none'
					},
					'>*': {
						width: '100%',
						height: '100%',
						overflow: 'hidden',
						flexShrink: 0,
						scrollSnapAlign: 'start'
					},
					'>*.disabled': {
						pointerEvents: 'none',
						userSelect: 'none',
						'*': { pointerEvents: 'none', userSelect: 'none' }
					}
				}}
				onScroll={(e) => {
					e.preventDefault();
				}}
			>
				{pages.map((Each, index) => (
					<Each
						key={index}
						className={pageIndex === index ? undefined : 'disabled'}
						tabIndex={pageIndex === index ? undefined : -1}
					/>
				))}
			</Box>
			<Stepper
				activeStep={pageIndex}
				sx={{ width: '100%', maxWidth: '720px', p: '16px', alignSelf: 'center' }}
				component="nav"
			>
				{steps.map((each, index) => (
					<Step key={index}>
						<StepLabel>{each}</StepLabel>
					</Step>
				))}
			</Stepper>
			<ErrorDialog info={allErrorInfo[0]} onClose={handleCloseDialog} />
		</>
	);
}

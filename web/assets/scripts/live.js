// SPDX-License-Identifier: GPL-2.0-or-later

import Hls from "./vendor/hls.js";
import { uniqueID, sortByName } from "./libs/common.js";
import { newOptionsMenu, newOptionsBtn } from "./components/optionsMenu.js";
import { newFeed, newFeedBtn } from "./components/feed.js";
import { initBandwidthMonitor } from "./bandwidthMonitor.js";

/**
 * @typedef {import("./components/feed.js").FullscreenButton} FullscreenButton
 * @typedef {import("./components/optionsMenu.js").Button} Button
 */

function newViewer($parent, monitors, hls, preferLowRes) {
	let selectedMonitors = [];
	const isMonitorSelected = (monitor) => {
		if (selectedMonitors.length === 0) {
			return true;
		}
		for (const id of selectedMonitors) {
			if (monitor["id"] == id) {
				return true;
			}
		}
		return false;
	};

	const sortedMonitors = sortByName(monitors);
	let feeds = [];

	/** @type {FullscreenButton[]} */
	let fullscreenButtons = [];

	return {
		setMonitors(input) {
			selectedMonitors = input;
		},
		reset() {
			for (const feed of feeds) {
				feed.destroy();
			}
			feeds = [];
			for (const monitor of Object.values(sortedMonitors)) {
				if (!isMonitorSelected(monitor)) {
					continue;
				}
				if (monitor["enable"] !== true) {
					continue;
				}

				const recordingsPath = toAbsolutePath("recordings");

				const fullscreenBtn = newFeedBtn.fullscreen();
				fullscreenButtons.push(fullscreenBtn);
				const buttons = [
					newFeedBtn.recordings(recordingsPath, monitor["id"]),
					fullscreenBtn,
					newFeedBtn.mute(monitor),
				];
				feeds.push(newFeed(hls, monitor, preferLowRes ?? false, buttons));
			}

			let html = "";
			for (const feed of feeds) {
				html += feed.html;
			}
			$parent.innerHTML = html;

			for (const feed of feeds) {
				feed.init();
			}
		},
		exitFullscreen() {
			for (const btn of fullscreenButtons) {
				btn.exitFullscreen();
			}
		},
	};
}

function toAbsolutePath(input) {
	return window.location.href.replace("live", input);
}

function init(preferLowRes) {
	// Globals.
	//const groups = Groups; // eslint-disable-line no-undef
	// @ts-ignore
	const monitors = MonitorsInfo; // eslint-disable-line no-undef

	const $contentGrid = document.querySelector("#content-grid");
	const viewer = newViewer($contentGrid, monitors, Hls, preferLowRes);

	const buttons = [
		newOptionsBtn.gridSize(viewer),
	];
	const optionsMenu = newOptionsMenu(buttons);
	document.querySelector("#options-menu").innerHTML = optionsMenu.html();
	optionsMenu.init();
	viewer.reset();

	initBandwidthMonitor();

	window.addEventListener("keydown", (e) => {
		if (e.key === "Escape") {
			viewer.exitFullscreen();
		}
	});
}

export { init, newViewer };

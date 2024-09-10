// SPDX-License-Identifier: GPL-2.0-or-later

import { uidReset } from "./libs/common.js";
import { newViewer } from "./live.js";

class mockHls {
	constructor() {}
	init() {}
	destroy() {}
	static isSupported() {
		return true;
	}
}
mockHls.Events = {
	MEDIA_ATTACHED() {},
};

test("fullscreen", () => {
	uidReset();
	document.body.innerHTML = `<div></div>`;
	const element = document.querySelector("div");
	const viewer = newViewer(element, [{ enable: true }, { enable: true }], mockHls);
	viewer.reset();

	const got = element.innerHTML.replaceAll(/\s/g, "");
	const want = `
		<div style="display:flex; justify-content:center;">
			<div id="uid1" class="grid-item-container">
				<input
					class="js-checkbox player-overlay-checkbox"
					id="uid2"
					type="checkbox"
				>
				<label class="player-overlay-selector" for="uid2"></label>
				<div class="js-overlay player-overlay feed-menu">
					<a href="http://localhost/#monitors=undefined" class="feed-btn">
						<img
							class="feed-btn-img icon"
							style="height:0.65rem;"
							src="assets/icons/feather/film.svg"
						>
					</a>
					<button class="js-fullscreen-btn feed-btn">
						<img class="feed-btn-img icon" src="assets/icons/feather/maximize.svg">
					</button>
				</div>
				<video
					class="grid-item"
					muted=""
					disablepictureinpicture=""
					playsinline=""
				></video>
			</div>
		</div>
			<div style="display:flex; justify-content:center;">
				<div id="uid3" class="grid-item-container">
					<input
						class="js-checkbox player-overlay-checkbox"
						id="uid4"
						type="checkbox"
					>
					<label class="player-overlay-selector" for="uid4"></label>
					<div class="js-overlay player-overlay feed-menu">
					<a href="http://localhost/#monitors=undefined" class="feed-btn">
						<img
							class="feed-btn-imgicon"
							style="height:0.65rem;"
							src="assets/icons/feather/film.svg"
						>
					</a>
					<button class="js-fullscreen-btn feed-btn">
						<img class="feed-btn-img icon" src="assets/icons/feather/maximize.svg">
					</button>
				</div>
				<video
					class="grid-item"
					muted=""
					disablepictureinpicture=""
					playsinline=""
				></video>
			</div>
		</div>`.replaceAll(/\s/g, "");
	expect(got).toEqual(want);

	const isFullscreen = (i) => {
		return element.children[i].classList.contains("grid-fullscreen");
	};

	expect(isFullscreen(0)).toBe(false);
	expect(isFullscreen(1)).toBe(false);
	// @ts-ignore
	element.querySelector(".js-fullscreen-btn").click();
	expect(isFullscreen(0)).toBe(true);
	expect(isFullscreen(1)).toBe(false);
});

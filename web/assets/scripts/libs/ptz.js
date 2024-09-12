export async function fetchAllPtzCapabilities(monitors) {
	const fetches = Object.keys(monitors)
		.map(id =>
			fetch(`/api/ptz/capabilities/${id}`)
				.then(response => response.json())
				.catch(() => undefined)
		);

	const results = await Promise.all(fetches);
	const capabilities = {};
	for (const [index, result] of results.entries()) {
		if (result) {
			capabilities[Object.keys(monitors)[index]] = result;
		}
	}
	return capabilities;
}

/**
 * @typedef {Object} PtzCapabilities
 * @property {string[]} supported_movements
 * @property {string[]} supported_zoom
 */

/** @param {PtzCapabilities|undefined} ptzCapabilities */
export function hasAnyPtzControls(ptzCapabilities) {
	return (ptzCapabilities?.supported_movements?.length ?? 0) > 0 || (ptzCapabilities?.supported_zoom?.length ?? 0) > 0;
}

/**
 * @param {HTMLElement} $container
 * @param {PtzButton} ptzBtn
 */
export function createPtzControls($container, ptzBtn) {
	const canMove = (ptzBtn.capabilities?.supported_movements?.length ?? 0) > 0;
	const canZoom = (ptzBtn.capabilities?.supported_zoom?.length ?? 0) > 0;

	if (!canMove && !canZoom) {
		return;
	}

	const html = `
	<div id="ptz-controls-${ptzBtn.id}" class="player-overlay ptz-menu">
		${canMove ? `
		<div>
			<button class="js-ptz-up" data-direction="Up">&#x25b2;</button>
			<button class="js-ptz-down" data-direction="Down">&#x25bc;</button>
			<button class="js-ptz-left" data-direction="Left">&#x25c0;</button>
			<button class="js-ptz-right" data-direction="Right">&#x25b6;</button>
		</div>
		` : ''}
		${canZoom ? `
		<div>
			<button class="js-ptz-zoom-in" data-direction="ZoomIn"> + </button>
			<button class="js-ptz-zoom-out" data-direction="ZoomOut"> - </button>
		</div>
		` : ''}
	</div>
	`

	$container.insertAdjacentHTML('beforeend', html);
	const $ptzButtons = document.querySelectorAll(`#ptz-controls-${ptzBtn.id} button`);
	for (const $ptzButton of $ptzButtons) {
		$ptzButton.addEventListener('click', async () => {
			// disable all buttons while the request is in progress
			for (const $button of $ptzButtons) {
				$button.disabled = true;
			}
			try {
				const response = await fetch(`/api/ptz/move/${ptzBtn.id}`,
					{
						method: 'POST',
						body: JSON.stringify({ direction: $ptzButton.dataset.direction }),
						headers: { 'Content-Type': 'application/json' }
					}
				);
				if (response.ok) {
					// TODO: blink the button green?
				} else {
					// TODO: blink the button red?
				}

			} catch(e) {
				console.error(e);
				// TODO: blink the button red?
			} finally {
				// re-enable all buttons
				for (const $button of $ptzButtons) {
					$button.disabled = false
				}
			}
		});
	}
}

export function togglePtzControls($container, ptzBtn) {
	const $ptzControls = document.querySelector(`#ptz-controls-${ptzBtn.id}`);
	if ($ptzControls) {
		$ptzControls.remove();
	} else {
		createPtzControls($container, ptzBtn);
	}
}

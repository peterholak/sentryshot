<!-- SPDX-License-Identifier: GPL-2.0-or-later -->

<!DOCTYPE html>
{% include "html" %}
<head>
	{% include "meta" %}
	<script type="module" defer>
		import { init } from "./assets/scripts/live.js";
		init({% if flags?.hd %}false{% else %}true{% endif %});
	</script>
</head>
<body>
	{% include "sidebar" %}
	<div id="content">
		<div id="content-grid-wrapper">
			<div id="content-grid"></div>
		</div>
	</div>
</body>

<style>
	#nav-link-live{% if flags?.hd %}-hd{% endif %} {
		background: var(--color1-hover);
	}
</style>
{% include "html2" %}

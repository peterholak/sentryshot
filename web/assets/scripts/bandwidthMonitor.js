let totalBytes = 0;
let updateInterval;
let recentBandwidths = [];
let originalFetch;

function updateBandwidth() {
    const now = performance.now();
    
    recentBandwidths.push({bytes: totalBytes, time: now});
    if (recentBandwidths.length > 5) {
        recentBandwidths.shift();
    }

    if (recentBandwidths.length > 1) {
        const oldestEntry = recentBandwidths[0];
        const newestEntry = recentBandwidths[recentBandwidths.length - 1];
        const duration = (newestEntry.time - oldestEntry.time) / 1000;
        const bytesDiff = newestEntry.bytes - oldestEntry.bytes;
        const averageBandwidth = (bytesDiff) / (1000000 * duration); // Convert to MB/s
        
        const $bandwidthValue = document.getElementById('bandwidth-value');
        if ($bandwidthValue) {
            $bandwidthValue.textContent = averageBandwidth.toFixed(2);
        }
    }

    const $bandwidthTotal = document.getElementById('bandwidth-total');
    if ($bandwidthTotal) {
        $bandwidthTotal.textContent = (totalBytes / 1000000).toFixed(2);
    }
}

export function initBandwidthMonitor() {
    if (originalFetch !== undefined && window.fetch !== originalFetch) {
        console.error('Bandwidth monitor already initialized');
        return;
    }

    totalBytes = 0;
    recentBandwidths = [];

    originalFetch = window.fetch;
    window.fetch = function() {
        return originalFetch.apply(this, arguments).then(response => {
            const clonedResponse = response.clone();
            clonedResponse.blob().then(blob => {
                totalBytes += blob.size;
            });
            return response;
        });
    };

    updateInterval = setInterval(updateBandwidth, 1000);
}

export function stopBandwidthMonitor() {
    if (updateInterval) {
        clearInterval(updateInterval);
    }

    if (window.fetch !== originalFetch) {
        window.fetch = originalFetch;
    }
}

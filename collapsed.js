// JavaScript bridge for collapsed tabs viewer page
// Provides Chrome API access for the viewer

/**
 * Get storage data
 * @param {string} key - Storage key
 * @returns {Promise<any>} Stored data
 */
export async function getStorage(key) {
  const result = await chrome.storage.local.get(key);
  return result[key];
}

/**
 * Set storage data
 * @param {string} key - Storage key
 * @param {any} value - Value to store
 */
export async function setStorage(key, value) {
  await chrome.storage.local.set({ [key]: value });
}

/**
 * Create tabs from URLs
 * @param {Array} tabs - Array of tab objects with url property
 * @param {Function} progressCallback - Called with progress percentage
 */
export async function restoreTabs(tabs, progressCallback) {
  const CHUNK_SIZE = 50;
  const total = tabs.length;
  const chunks = [];

  // Split into chunks
  for (let i = 0; i < tabs.length; i += CHUNK_SIZE) {
    chunks.push(tabs.slice(i, i + CHUNK_SIZE));
  }

  // Process each chunk
  for (let chunkIndex = 0; chunkIndex < chunks.length; chunkIndex++) {
    const chunk = chunks[chunkIndex];

    // Create all tabs in this chunk in parallel
    const createPromises = chunk.map(tab => {
      return chrome.tabs.create({
        url: tab.url,
        active: false,
        pinned: tab.pinned || false
      });
    });

    await Promise.all(createPromises);

    // Update progress
    const processed = Math.min((chunkIndex + 1) * CHUNK_SIZE, total);
    const progress = Math.round((processed / total) * 100);
    if (progressCallback) {
      progressCallback(progress);
    }

    // Yield control to browser
    await new Promise(resolve => setTimeout(resolve, 0));
  }
}

/**
 * Export data as JSON file
 * @param {string} data - JSON string to export
 * @param {string} filename - Filename for download
 */
export function exportToFile(data, filename) {
  const blob = new Blob([data], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

// Log that the bridge is loaded
console.log('Tab Hoarder collapsed.js bridge loaded');

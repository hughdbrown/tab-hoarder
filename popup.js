// JavaScript bridge for Chrome APIs
// Provides functions callable from Rust/WASM

const CHUNK_SIZE = 50; // Process tabs in chunks of 50

/**
 * Get all tabs in the current window
 * @returns {Promise<Array>} Array of tab objects
 */
export async function getCurrentWindowTabs() {
  const tabs = await chrome.tabs.query({ currentWindow: true });
  return tabs.map(tab => ({
    id: tab.id,
    url: tab.url || '',
    title: tab.title || '',
    pinned: tab.pinned || false,
    index: tab.index
  }));
}

/**
 * Sort tabs by domain with batch processing
 * @param {Array} sortedTabIds - Array of tab IDs in desired order
 * @param {Function} progressCallback - Called with progress percentage (0-100)
 */
export async function sortTabsByDomain(sortedTabIds, progressCallback) {
  const total = sortedTabIds.length;
  const chunks = [];

  // Split into chunks
  for (let i = 0; i < sortedTabIds.length; i += CHUNK_SIZE) {
    chunks.push(sortedTabIds.slice(i, i + CHUNK_SIZE));
  }

  // Process each chunk
  for (let chunkIndex = 0; chunkIndex < chunks.length; chunkIndex++) {
    const chunk = chunks[chunkIndex];
    const startIndex = chunkIndex * CHUNK_SIZE;

    // Move all tabs in this chunk in parallel
    const movePromises = chunk.map((tabId, offsetInChunk) => {
      const newIndex = startIndex + offsetInChunk;
      return chrome.tabs.move(tabId, { index: newIndex });
    });

    await Promise.all(movePromises);

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
 * Remove duplicate tabs
 * @param {Array} tabIdsToRemove - Array of tab IDs to close
 * @param {Function} progressCallback - Called with progress percentage (0-100)
 */
export async function removeTabs(tabIdsToRemove, progressCallback) {
  if (tabIdsToRemove.length === 0) return;

  const total = tabIdsToRemove.length;
  const chunks = [];

  // Split into chunks
  for (let i = 0; i < tabIdsToRemove.length; i += CHUNK_SIZE) {
    chunks.push(tabIdsToRemove.slice(i, i + CHUNK_SIZE));
  }

  // Process each chunk
  for (let chunkIndex = 0; chunkIndex < chunks.length; chunkIndex++) {
    const chunk = chunks[chunkIndex];

    await chrome.tabs.remove(chunk);

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
 * Close tabs (for collapse operation)
 * @param {Array} tabIds - Array of tab IDs to close
 * @param {Function} progressCallback - Called with progress percentage (0-100)
 */
export async function closeTabs(tabIds, progressCallback) {
  return removeTabs(tabIds, progressCallback);
}

/**
 * Create tabs (for restore operation)
 * @param {Array} urls - Array of URLs to open
 * @param {Function} progressCallback - Called with progress percentage (0-100)
 */
export async function createTabs(urls, progressCallback) {
  const total = urls.length;
  const chunks = [];

  // Split into chunks
  for (let i = 0; i < urls.length; i += CHUNK_SIZE) {
    chunks.push(urls.slice(i, i + CHUNK_SIZE));
  }

  // Process each chunk
  for (let chunkIndex = 0; chunkIndex < chunks.length; chunkIndex++) {
    const chunk = chunks[chunkIndex];

    // Create all tabs in this chunk in parallel
    const createPromises = chunk.map(url => {
      return chrome.tabs.create({ url, active: false });
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
 * Activate (focus) a specific tab
 * @param {number} tabId - ID of the tab to activate
 */
export async function activateTab(tabId) {
  await chrome.tabs.update(tabId, { active: true });
}

/**
 * Close a single tab
 * @param {number} tabId - ID of the tab to close
 */
export async function closeTab(tabId) {
  await chrome.tabs.remove(tabId);
}

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
 * Get storage quota information
 * @returns {Promise<{bytesInUse: number, quota: number, percentUsed: number}>}
 */
export async function getStorageQuota() {
  const bytesInUse = await chrome.storage.local.getBytesInUse();
  const quota = chrome.storage.local.QUOTA_BYTES || 10485760; // 10MB default
  const percentUsed = Math.round((bytesInUse / quota) * 100);

  return {
    bytesInUse,
    quota,
    percentUsed
  };
}

/**
 * Open collapsed tabs viewer in a new tab
 */
export async function openCollapsedViewer() {
  const url = chrome.runtime.getURL('collapsed.html');
  await chrome.tabs.create({ url });
}

// Log that the bridge is loaded
console.log('Tab Hoarder popup.js bridge loaded');

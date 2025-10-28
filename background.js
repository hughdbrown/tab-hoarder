// Background service worker for Tab Hoarder
// Handles extension lifecycle events

chrome.runtime.onInstalled.addListener(() => {
  console.log('Tab Hoarder extension installed');
});

// Optional: Handle messages from content scripts or popup
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  console.log('Background received message:', message);

  // Handle different message types if needed
  switch (message.type) {
    case 'ping':
      sendResponse({ status: 'pong' });
      break;
    default:
      sendResponse({ status: 'unknown command' });
  }

  return true; // Keep message channel open for async response
});

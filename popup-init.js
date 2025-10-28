// Popup initialization script
import init, { start_popup } from './pkg/tab_hoarder.js';

async function run() {
    await init();
    start_popup();
}

run();

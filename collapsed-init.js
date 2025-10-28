// Collapsed viewer initialization script
import init, { start_collapsed_viewer } from './pkg/tab_hoarder.js';

async function run() {
    await init();
    start_collapsed_viewer();
}

run();

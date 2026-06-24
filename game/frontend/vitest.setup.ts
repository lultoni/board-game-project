// Vitest global setup: install fake-indexeddb so tests that touch the
// telemetry store can run in Node without a browser.
import "fake-indexeddb/auto";

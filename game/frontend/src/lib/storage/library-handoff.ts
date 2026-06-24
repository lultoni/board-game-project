// Cross-route handoff for "Open in Inspector" from the library.
//
// The inspector already accepts a MatchLog via paste (entryFromMatchLog).
// To avoid coupling the library route to inspector internals, we stash the
// log in sessionStorage at navigation time and the inspector consumes it
// on mount. sessionStorage is per-tab and cleared by the browser when the
// tab closes — exactly the lifetime we want for a one-shot handoff.
//
// Failures are swallowed: private-mode Safari can throw on sessionStorage
// writes. A failed handoff just means the user lands on a blank inspector
// and can still paste the log manually.

const KEY = "boardgame:pending-matchlog";

export function setPendingMatchLog(json: string): void {
  try {
    sessionStorage.setItem(KEY, json);
  } catch {
    // private-mode quota / SecurityError — ignore.
  }
}

export function consumePendingMatchLog(): string | null {
  try {
    const v = sessionStorage.getItem(KEY);
    if (v !== null) sessionStorage.removeItem(KEY);
    return v;
  } catch {
    return null;
  }
}

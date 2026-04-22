// Abstraction lint — the load-bearing assertion that no substrate vocabulary
// leaks into the rendered UI. Runs at startup AND after every view render so
// that any view accidentally writing forbidden tokens is caught immediately in
// development. Failures log a console warning; they do NOT block rendering
// (the reader still deserves to see the page). Authors iterating on the bundle
// should treat a warning as a must-fix.
// @trace spec:calmecac
// @Lesson S1-1500

const FORBIDDEN = [
  // substrate format
  /\bmarkdown\b/i,
  /\.md\b/i,
  // filesystem vocabulary
  /\bfile\b/i,
  /\bfilename\b/i,
  /\bfilepath\b/i,
  /\bpath\b/i,
  // version-control mechanism
  /\bcommit\b/i,
  /\bcommits\b/i,
  /\brepo\b/i,
  /\brepository\b/i,
  /\bhash\b/i,
  // spec vocabulary (should be "rule")
  /\bspec\b/i,
  /\bspecs\b/i,
];

// Exceptions: some tokens are legitimate display text — notably the `@trace
// spec:<name>` citation form, which was introduced in the comic and is allowed
// per the Abstraction vocabulary table. The lint respects that by ignoring
// matches that occur *inside* the `@trace` citation marker.
const ALLOWED_SUBSTRINGS = [
  /@trace\s+rule:/,
  /@trace\s+spec:/,
  /@Lesson\s+S\d+-\d+/,
];

function shouldIgnore(text) {
  return ALLOWED_SUBSTRINGS.some((re) => re.test(text));
}

export function installAbstractionLint(root) {
  if (!root) return;
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, null);
  const hits = [];
  let node;
  while ((node = walker.nextNode())) {
    const raw = node.nodeValue || "";
    if (!raw.trim()) continue;
    if (shouldIgnore(raw)) continue;
    for (const re of FORBIDDEN) {
      if (re.test(raw)) {
        // Special-case: the literal `@trace spec:…` text is allowed because
        // the reader learned it from the comic. Only complain if the forbidden
        // word appears outside that context.
        const cleaned = raw
          .replace(/@trace\s+(rule|spec):[^\s]+/g, "")
          .replace(/@Lesson\s+S\d+-\d+/g, "");
        if (re.test(cleaned)) {
          hits.push({ term: re.source, text: raw.trim().slice(0, 140) });
        }
      }
    }
  }
  if (hits.length > 0) {
    // Warn, don't throw. A noisy console is enough of a signal for iteration.
    console.warn(
      `[calmecac abstraction-lint] ${hits.length} substrate-vocabulary leak(s) detected in visible UI text:`,
      hits.slice(0, 10)
    );
    // Expose on window for dev inspection.
    window.__calmecacAbstractionHits = hits;
  } else {
    window.__calmecacAbstractionHits = [];
  }
  return hits;
}

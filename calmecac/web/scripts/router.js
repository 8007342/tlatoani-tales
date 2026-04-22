// Minimal hash-based router. Hash routing is used (not pushState) so the bundle
// can be served from any static host without requiring catch-all rewrites.
// @trace spec:calmecac

let routes = [];
let opts = {};

export function registerRoutes(routeList, options = {}) {
  routes = routeList;
  opts = options;
}

export function start(forceRerender = false) {
  if (!window.__calmecacRouterWired) {
    window.addEventListener("hashchange", () => dispatch());
    window.__calmecacRouterWired = true;
  }
  dispatch(forceRerender);
}

export function navigate(path) {
  window.location.hash = path.startsWith("#") ? path : `#${path}`;
}

function dispatch() {
  const raw = window.location.hash || "#/";
  const path = raw.replace(/^#/, "");
  for (const route of routes) {
    const m = path.match(route.path);
    if (m) {
      if (opts.onBeforeRender && opts.onBeforeRender(route, m) === false) return;
      try {
        Promise.resolve(route.render(m, path)).then(() => {
          if (opts.onAfterRender) opts.onAfterRender(route, m);
        });
      } catch (err) {
        console.error("view render failed", err);
      }
      return;
    }
  }
  if (opts.onUnmatched) opts.onUnmatched(path);
}

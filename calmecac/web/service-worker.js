// Calmecac service worker — cache-first for the static bundle and the concept index.
// @trace spec:calmecac
// @Lesson S1-1000

const CACHE = "calmecac-v1";

const PRECACHE = [
  "./",
  "./index.html",
  "./manifest.webmanifest",
  "./styles/base.css",
  "./styles/comic-view.css",
  "./styles/lesson-view.css",
  "./styles/rule-view.css",
  "./styles/convergence.css",
  "./styles/graph.css",
  "./styles/tombstones.css",
  "./scripts/main.js",
  "./scripts/router.js",
  "./scripts/breadcrumbs.js",
  "./scripts/abstraction-lint.js",
  "./scripts/views/comic.js",
  "./scripts/views/lesson.js",
  "./scripts/views/rule.js",
  "./scripts/views/convergence.js",
  "./scripts/views/graph.js",
  "./scripts/views/tombstones.js",
  "./assets/icons/icon.svg",
  "./assets/icons/icon-192.svg",
  "./assets/icons/icon-512.svg"
];

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(CACHE).then((cache) => cache.addAll(PRECACHE)).then(() => self.skipWaiting())
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k)))
    ).then(() => self.clients.claim())
  );
});

self.addEventListener("fetch", (event) => {
  const req = event.request;
  if (req.method !== "GET") return;
  const url = new URL(req.url);
  if (url.origin !== self.location.origin) return;

  event.respondWith(
    caches.match(req).then((cached) => {
      if (cached) return cached;
      return fetch(req).then((res) => {
        if (res && res.ok && res.type === "basic") {
          const copy = res.clone();
          caches.open(CACHE).then((cache) => cache.put(req, copy));
        }
        return res;
      }).catch(() => cached || new Response("Offline — this view is not yet cached.", {
        status: 503,
        headers: { "Content-Type": "text/plain; charset=utf-8" }
      }));
    })
  );
});

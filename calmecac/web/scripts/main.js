// Calmecac — entry point. Loads the concept index, wires the router, runs the
// abstraction-lint sweep, and registers the service worker.
// @trace spec:calmecac
// @Lesson S1-1000
// @Lesson S1-1500

import { registerRoutes, start as startRouter } from "./router.js";
import { renderBreadcrumbs } from "./breadcrumbs.js";
import { installAbstractionLint } from "./abstraction-lint.js";

import { render as renderComic } from "./views/comic.js";
import { render as renderLesson } from "./views/lesson.js";
import { render as renderRule } from "./views/rule.js";
import { render as renderConvergence } from "./views/convergence.js";
import { render as renderGraph } from "./views/graph.js";
import { render as renderTombstones } from "./views/tombstones.js";

const INDEX_CACHE_KEY = "calmecac-index-v1";
const INDEX_URL = "./calmecac-index.json";
const FALLBACK_INDEX_URL = "./examples/calmecac-index.json";

async function loadIndex() {
  // sessionStorage cache: small enough for a single-session hydration,
  // avoids refetching between view transitions. IndexedDB was considered and
  // rejected — the index is one blob, sessionStorage is sufficient, and the
  // service worker handles cross-session caching.
  const cached = sessionStorage.getItem(INDEX_CACHE_KEY);
  if (cached) {
    try { return JSON.parse(cached); } catch (_) { sessionStorage.removeItem(INDEX_CACHE_KEY); }
  }
  const tryFetch = async (url) => {
    const res = await fetch(url, { cache: "no-cache" });
    if (!res.ok) throw new Error(`index fetch failed: ${res.status}`);
    return res.json();
  };
  let data;
  try {
    data = await tryFetch(INDEX_URL);
  } catch (_) {
    data = await tryFetch(FALLBACK_INDEX_URL);
  }
  try { sessionStorage.setItem(INDEX_CACHE_KEY, JSON.stringify(data)); } catch (_) { /* quota: fine */ }
  return data;
}

function renderError(container, message) {
  container.innerHTML = "";
  const card = document.createElement("div");
  card.className = "empty-state";
  card.textContent = message;
  container.appendChild(card);
}

function getReaderProgress() {
  const stored = localStorage.getItem("calmecac-reader-progress");
  return stored === "s1s2" ? "s1s2" : "s1";
}

function setReaderProgress(value) {
  localStorage.setItem("calmecac-reader-progress", value);
  document.body.dataset.progress = value;
}

function wireReaderProgress() {
  const select = document.getElementById("reader-progress");
  if (!select) return;
  select.value = getReaderProgress();
  document.body.dataset.progress = select.value;
  select.addEventListener("change", (e) => {
    setReaderProgress(e.target.value);
    // Re-render current view so any season-gated content updates.
    startRouter(true);
  });
}

async function boot() {
  const main = document.getElementById("main");
  const crumbs = document.getElementById("breadcrumbs");
  wireReaderProgress();

  let data;
  try {
    main.innerHTML = `<div class="loading">Opening the upstairs library…</div>`;
    data = await loadIndex();
  } catch (err) {
    renderError(main, "The concept index is not available. Run the build once, then refresh this page.");
    console.error(err);
    return;
  }

  const ctx = {
    data,
    progress: () => getReaderProgress(),
  };

  registerRoutes([
    { path: /^\/?$/, name: "home", render: (p, c) => renderHome(main, ctx, p) },
    { path: /^\/comic\/(\d+)$/, name: "comic", render: (p) => renderComic(main, ctx, { episode: p[1] }) },
    { path: /^\/lesson\/([A-Za-z0-9-]+)$/, name: "lesson", render: (p) => renderLesson(main, ctx, { id: p[1] }) },
    { path: /^\/rule\/([A-Za-z0-9āēīōūǎ-]+)$/, name: "rule", render: (p) => renderRule(main, ctx, { name: p[1] }) },
    { path: /^\/convergence\/?$/, name: "convergence", render: () => renderConvergence(main, ctx) },
    { path: /^\/graph\/?$/, name: "graph", render: () => renderGraph(main, ctx) },
    { path: /^\/tombstones\/?$/, name: "tombstones", render: () => renderTombstones(main, ctx) },
    // Season-2 stubs — gated below
    { path: /^\/trust-boundary\/?$/, name: "trust-boundary", season: 2, render: () => renderSeasonTwoStub(main, "Trust-boundary map") },
    { path: /^\/offline-proof\/?$/, name: "offline-proof", season: 2, render: () => renderSeasonTwoStub(main, "Offline-proof timeline") },
  ], {
    onBeforeRender: (route, params) => {
      if (route.season === 2 && getReaderProgress() !== "s1s2") {
        renderError(main, "This view unlocks after Season 2. Set reader progress to Season 1 + 2 in the header to peek ahead.");
        renderBreadcrumbs(crumbs, [{ label: "Home", href: "#/" }, { label: route.name }]);
        return false;
      }
      return true;
    },
    onAfterRender: (route, params) => {
      renderBreadcrumbs(crumbs, crumbsFor(route, params, ctx));
      main.focus();
      installAbstractionLint(document.getElementById("main"));
    },
    onUnmatched: () => {
      renderError(main, "That door is not in this library. Try the home page.");
      renderBreadcrumbs(crumbs, [{ label: "Home", href: "#/" }, { label: "Not found" }]);
    }
  });

  startRouter();

  // Register service worker (progressive enhancement — silent failures OK).
  if ("serviceWorker" in navigator && location.protocol !== "file:") {
    navigator.serviceWorker.register("./service-worker.js").catch(() => { /* dev mode: fine */ });
  }
}

function crumbsFor(route, params, ctx) {
  const home = { label: "Home", href: "#/" };
  if (route.name === "home") return [{ label: "Home" }];
  if (route.name === "comic") {
    return [home, { label: `Strip ${params[1]}` }];
  }
  if (route.name === "lesson") {
    return [home, { label: "Lessons", href: "#/" }, { label: params[1] }];
  }
  if (route.name === "rule") {
    return [home, { label: "Rules", href: "#/" }, { label: params[1] }];
  }
  if (route.name === "convergence") return [home, { label: "Convergence" }];
  if (route.name === "graph") return [home, { label: "Graph" }];
  if (route.name === "tombstones") return [home, { label: "Tombstones" }];
  return [home, { label: route.name }];
}

function renderHome(main, ctx) {
  const { data } = ctx;
  main.innerHTML = "";
  const frag = document.createDocumentFragment();

  const header = document.createElement("header");
  header.className = "view-header";
  header.innerHTML = `
    <div class="eyebrow">Calmecac — the upstairs library</div>
    <h1>Every strip, and how it was made.</h1>
    <p class="teaching">A Lesson is a teaching. A Rule is a contract. A Strip is a published comic. Click a plate on any strip to begin.</p>
  `;
  frag.appendChild(header);

  const seasonsSection = document.createElement("section");
  seasonsSection.innerHTML = `<h2>Season 1</h2>`;
  const stripsGrid = document.createElement("div");
  stripsGrid.className = "grid";
  (data.strips || []).forEach((strip) => {
    const card = document.createElement("a");
    card.className = "card";
    card.href = `#/comic/${strip.episode}`;
    card.style.textDecoration = "none";
    card.innerHTML = `
      <div class="eyebrow">Strip ${strip.episode}</div>
      <h3>${escapeHtml(strip.title || "")}</h3>
      <p class="meta-row"><strong>Lesson:</strong> ${escapeHtml(strip.primary_lesson || "—")}</p>
      <p class="meta-row"><strong>Rule:</strong> ${escapeHtml(strip.primary_rule || "—")}</p>
    `;
    stripsGrid.appendChild(card);
  });
  seasonsSection.appendChild(stripsGrid);
  frag.appendChild(seasonsSection);

  const convSection = document.createElement("section");
  convSection.innerHTML = `
    <h2>Convergence at a glance</h2>
    <p>The project's progress toward its own Rules.</p>
    <p><a class="btn" href="#/convergence">Open the Convergence dashboards</a> <a class="btn" href="#/graph">Open the Graph</a> <a class="btn" href="#/tombstones">Browse Tombstones</a></p>
  `;
  frag.appendChild(convSection);

  if (data.last_change && data.last_change.subject) {
    const banner = document.createElement("aside");
    banner.className = "teaching";
    banner.innerHTML = `Last change: <em>${escapeHtml(data.last_change.subject)}</em> on ${escapeHtml(data.last_change.date || "—")}.`;
    frag.appendChild(banner);
  }

  main.appendChild(frag);
}

function renderSeasonTwoStub(container, title) {
  container.innerHTML = "";
  const wrap = document.createElement("section");
  wrap.innerHTML = `
    <header class="view-header"><h1>${escapeHtml(title)}</h1></header>
    <div class="empty-state">(view in progress — content will appear once data is indexed for Season 2)</div>
  `;
  container.appendChild(wrap);
}

function escapeHtml(s) {
  return String(s == null ? "" : s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
}

boot();

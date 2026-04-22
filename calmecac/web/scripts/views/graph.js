// Graph view — the whole observability scaffold as a tiny SVG diagram.
// No external layout library; simple tidy-tree-like placement suffices.
// Narrow screens and reduced-motion readers get an outline list instead.
// @trace spec:calmecac

import { escapeHtml, h } from "./_util.js";

export function render(container, ctx) {
  const { data } = ctx;
  container.innerHTML = "";

  const root = document.createElement("section");
  root.className = "graph-view";

  const head = document.createElement("header");
  head.className = "view-header";
  head.innerHTML = `
    <div class="eyebrow">Graph</div>
    <h1>Lessons, Rules, Strips — the whole scaffold.</h1>
    <p class="teaching">Click any node to open its view. The edges are teachings and citations.</p>
  `;
  root.appendChild(head);

  const legend = document.createElement("div");
  legend.className = "legend";
  legend.innerHTML = `
    <span><span class="swatch swatch-lesson"></span>Lesson</span>
    <span><span class="swatch swatch-rule"></span>Rule</span>
    <span><span class="swatch swatch-strip"></span>Strip</span>
    <span><span class="swatch swatch-tombstone"></span>Tombstone</span>
  `;
  root.appendChild(legend);

  const frame = document.createElement("div");
  frame.className = "graph-frame";
  frame.appendChild(buildSvg(data));
  root.appendChild(frame);

  root.appendChild(buildOutline(data));

  container.appendChild(root);
}

function buildSvg(data) {
  const NS = "http://www.w3.org/2000/svg";
  const svg = document.createElementNS(NS, "svg");
  const W = 900, H = 640;
  svg.setAttribute("viewBox", `0 0 ${W} ${H}`);
  svg.setAttribute("preserveAspectRatio", "xMidYMid meet");
  svg.setAttribute("role", "img");
  svg.setAttribute("aria-label", "Diagram of Lessons, Rules, and Strips");

  const lessons = data.lessons || [];
  const rules = data.rules || [];
  const strips = data.strips || [];

  // Place Lessons across the top; Rules in the middle; Strips along the bottom.
  const margin = 60;
  const laneY = (i) => margin + i * ((H - margin * 2) / 2);

  const positions = new Map();

  function layoutLane(items, laneIndex, prefix) {
    const n = items.length || 1;
    const step = (W - margin * 2) / Math.max(1, n - 1);
    items.forEach((item, i) => {
      const x = n === 1 ? W / 2 : margin + i * step;
      const y = laneY(laneIndex);
      positions.set(prefix + ":" + (item.id || item.name || item.episode), { x, y, item, prefix });
    });
  }
  layoutLane(lessons, 0, "lesson");
  layoutLane(rules, 1, "rule");
  layoutLane(strips, 2, "strip");

  // Edges
  const edges = [];
  strips.forEach((s) => {
    if (s.primary_lesson) edges.push(["strip:" + s.episode, "lesson:" + s.primary_lesson]);
    if (s.primary_rule) edges.push(["strip:" + s.episode, "rule:" + s.primary_rule]);
    (s.reinforced_lessons || []).forEach((l) => edges.push(["strip:" + s.episode, "lesson:" + l]));
  });
  rules.forEach((r) => {
    (r.lessons || []).forEach((l) => {
      const id = l.id || l.short_id || l;
      edges.push(["rule:" + r.name, "lesson:" + id]);
    });
    (r.cites || []).forEach((c) => edges.push(["rule:" + r.name, "rule:" + c]));
  });

  const edgeGroup = document.createElementNS(NS, "g");
  edges.forEach(([a, b]) => {
    const A = positions.get(a), B = positions.get(b);
    if (!A || !B) return;
    const p = document.createElementNS(NS, "path");
    const midY = (A.y + B.y) / 2;
    p.setAttribute("d", `M${A.x},${A.y} C${A.x},${midY} ${B.x},${midY} ${B.x},${B.y}`);
    p.setAttribute("class", "edge");
    edgeGroup.appendChild(p);
  });
  svg.appendChild(edgeGroup);

  // Nodes
  const nodeGroup = document.createElementNS(NS, "g");
  for (const [key, { x, y, item, prefix }] of positions.entries()) {
    const g = document.createElementNS(NS, "g");
    g.setAttribute("class", "node-group");
    g.setAttribute("tabindex", "0");
    const label = prefix === "lesson" ? (item.short_id || item.display || item.id)
      : prefix === "rule" ? item.name
      : "TT " + item.episode;
    g.setAttribute("aria-label", `${prefix}: ${label}`);
    const href = prefix === "lesson" ? `#/lesson/${item.id || item.short_id}`
      : prefix === "rule" ? `#/rule/${item.name}`
      : `#/comic/${item.episode}`;
    g.addEventListener("click", () => { window.location.hash = href; });
    g.addEventListener("keydown", (e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); window.location.hash = href; } });

    const r = prefix === "lesson" ? 14 : prefix === "rule" ? 11 : 8;
    const circle = document.createElementNS(NS, "circle");
    circle.setAttribute("cx", x);
    circle.setAttribute("cy", y);
    circle.setAttribute("r", r);
    circle.setAttribute("class", item.retired ? "node-tombstone" : `node-${prefix}`);
    g.appendChild(circle);

    const text = document.createElementNS(NS, "text");
    text.setAttribute("x", x);
    text.setAttribute("y", y + r + 12);
    text.setAttribute("text-anchor", "middle");
    text.setAttribute("class", "node-label");
    text.textContent = label;
    g.appendChild(text);

    nodeGroup.appendChild(g);
  }
  svg.appendChild(nodeGroup);

  return svg;
}

function buildOutline(data) {
  // Narrow-screen / reduced-motion fallback: a plain nested outline.
  const wrap = document.createElement("div");
  wrap.className = "outline-fallback";
  wrap.appendChild(h("h2", {}, "Outline"));

  const lessonsSection = document.createElement("section");
  lessonsSection.appendChild(h("h3", {}, "Lessons"));
  const lessonsUl = document.createElement("ul");
  (data.lessons || []).forEach((l) => {
    const li = document.createElement("li");
    const a = document.createElement("a");
    a.href = `#/lesson/${l.id || l.short_id}`;
    a.textContent = `${l.short_id || l.id} — ${l.display || ""}`;
    li.appendChild(a);
    lessonsUl.appendChild(li);
  });
  lessonsSection.appendChild(lessonsUl);
  wrap.appendChild(lessonsSection);

  const rulesSection = document.createElement("section");
  rulesSection.appendChild(h("h3", {}, "Rules"));
  const rulesUl = document.createElement("ul");
  (data.rules || []).forEach((r) => {
    const li = document.createElement("li");
    const a = document.createElement("a");
    a.href = `#/rule/${r.name}`;
    a.textContent = `${r.name}${r.role ? " — " + r.role : ""}`;
    li.appendChild(a);
    rulesUl.appendChild(li);
  });
  rulesSection.appendChild(rulesUl);
  wrap.appendChild(rulesSection);

  const stripsSection = document.createElement("section");
  stripsSection.appendChild(h("h3", {}, "Strips"));
  const stripsUl = document.createElement("ul");
  (data.strips || []).forEach((s) => {
    const li = document.createElement("li");
    const a = document.createElement("a");
    a.href = `#/comic/${s.episode}`;
    a.textContent = `Strip ${s.episode} — ${s.title || ""}`;
    li.appendChild(a);
    stripsUl.appendChild(li);
  });
  stripsSection.appendChild(stripsUl);
  wrap.appendChild(stripsSection);

  return wrap;
}

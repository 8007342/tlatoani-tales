// Lesson view — abstract, aha, reinforcing strips, rule chips, argument trail.
// @trace spec:calmecac, spec:lessons

import { escapeHtml, h, openArchiveLink } from "./_util.js";

export function render(container, ctx, params) {
  const { data } = ctx;
  const id = params.id;
  const lesson = (data.lessons || []).find((l) => l.id === id || l.short_id === id);

  container.innerHTML = "";
  if (!lesson) {
    container.appendChild(h("div", { class: "empty-state" }, `Lesson ${escapeHtml(id)} is not in this index.`));
    return;
  }

  const root = document.createElement("section");
  root.className = "lesson-view";

  const head = document.createElement("header");
  head.className = "view-header";
  head.innerHTML = `
    <div class="eyebrow">Lesson</div>
    <span class="lesson-id">${escapeHtml(lesson.short_id || lesson.id)}</span>
    <h1 class="lesson-title">${escapeHtml(lesson.display || "")}</h1>
    ${lesson.takeaway ? `<p class="takeaway">${escapeHtml(lesson.takeaway)}</p>` : ""}
  `;
  root.appendChild(head);

  const body = document.createElement("div");
  body.className = "lesson-head";

  const left = document.createElement("div");
  if (lesson.abstract) {
    left.appendChild(h("h2", {}, "Abstract"));
    const p = document.createElement("p");
    p.textContent = lesson.abstract;
    left.appendChild(p);
  }
  // Neighbours
  const neighbours = document.createElement("div");
  neighbours.className = "neighbours";
  if (lesson.predecessors && lesson.predecessors.length) {
    neighbours.appendChild(h("span", { class: "label" }, "After:"));
    lesson.predecessors.forEach((p) => neighbours.appendChild(h("a", { class: "chip", "data-kind": "lesson", href: `#/lesson/${p.id || p}` }, p.display || p.id || p)));
  }
  if (lesson.successors && lesson.successors.length) {
    neighbours.appendChild(h("span", { class: "label" }, "Before:"));
    lesson.successors.forEach((p) => neighbours.appendChild(h("a", { class: "chip", "data-kind": "lesson", href: `#/lesson/${p.id || p}` }, p.display || p.id || p)));
  }
  if (neighbours.childNodes.length) left.appendChild(neighbours);

  // Aha moment
  if (lesson.aha) {
    const aha = document.createElement("p");
    aha.className = "aha";
    aha.textContent = lesson.aha;
    left.appendChild(aha);
  }

  body.appendChild(left);

  // Right: primary strip preview.
  const right = document.createElement("div");
  if (lesson.primary_strip) {
    const strip = (data.strips || []).find((s) => String(s.episode) === String(lesson.primary_strip));
    if (strip) {
      const fig = document.createElement("figure");
      fig.className = "strip-preview";
      const a = document.createElement("a");
      a.href = `#/comic/${strip.episode}`;
      if (strip.image_url) {
        const img = document.createElement("img");
        img.src = strip.image_url;
        img.alt = strip.alt_text || `Strip ${strip.episode}: ${strip.title || ""}`;
        img.loading = "lazy";
        a.appendChild(img);
      } else {
        const placeholder = h("div", { class: "empty-state" }, `Strip ${strip.episode} · render pending`);
        a.appendChild(placeholder);
      }
      fig.appendChild(a);
      fig.appendChild(h("figcaption", {}, `Primary strip — ${strip.title || "Strip " + strip.episode}`));
      right.appendChild(fig);
    }
  }
  body.appendChild(right);

  root.appendChild(body);

  // Rules row — one chip per rule in coverage.
  if (lesson.rules && lesson.rules.length) {
    const section = document.createElement("section");
    section.appendChild(h("h2", {}, "Rules that govern this teaching"));
    const row = document.createElement("div");
    row.className = "rules-row";
    lesson.rules.forEach((r) => {
      const name = r.name || r;
      row.appendChild(h("a", { class: "chip", "data-kind": "rule", href: `#/rule/${name}` }, `@trace rule:${name}`));
    });
    section.appendChild(row);
    root.appendChild(section);
  }

  // Reinforcing strips
  const reinforcing = (data.strips || []).filter((s) => (s.reinforced_lessons || []).some((x) => x === lesson.short_id || x === lesson.id));
  if (reinforcing.length) {
    const section = document.createElement("section");
    section.appendChild(h("h2", {}, "Reinforcing strips"));
    const grid = document.createElement("div");
    grid.className = "reinforcing-grid";
    reinforcing.forEach((strip) => {
      const a = document.createElement("a");
      a.href = `#/comic/${strip.episode}`;
      if (strip.image_url) {
        const img = document.createElement("img");
        img.src = strip.image_url;
        img.alt = strip.alt_text || `Strip ${strip.episode}`;
        img.loading = "lazy";
        a.appendChild(img);
      }
      const body = document.createElement("div");
      body.className = "tile-body";
      body.appendChild(h("div", { class: "tile-title" }, strip.title || `Strip ${strip.episode}`));
      body.appendChild(h("div", { class: "tile-ep" }, `Strip ${strip.episode}`));
      a.appendChild(body);
      grid.appendChild(a);
    });
    section.appendChild(grid);
    root.appendChild(section);
  }

  // Argument trail — changes that cite this lesson or its covered rules.
  const covered = new Set(((lesson.rules || []).map((r) => r.name || r)));
  const trail = (data.changes || []).filter((c) => {
    const tags = c.tags || [];
    const touchesLesson = tags.includes(`@Lesson ${lesson.short_id}`) || tags.includes(`@Lesson ${lesson.id}`);
    const touchesCoveredRule = tags.some((t) => covered.has(t.replace(/^@trace\s+(rule|spec):/, "")));
    return touchesLesson || touchesCoveredRule;
  });
  if (trail.length) {
    const section = document.createElement("section");
    section.className = "argument-trail";
    section.appendChild(h("h2", {}, "Argument trail"));
    section.appendChild(h("p", { class: "meta-row" }, "Every change that shaped this teaching, in reverse time order."));
    section.appendChild(renderChangeList(trail));
    root.appendChild(section);
  }

  container.appendChild(root);
}

function renderChangeList(changes) {
  const ol = document.createElement("ol");
  changes.forEach((c) => {
    const li = document.createElement("li");
    li.appendChild(h("span", { class: "date" }, c.date || "—"));
    li.appendChild(h("strong", { class: "subject" }, c.subject || c.summary || "(no summary)"));
    const tags = document.createElement("div");
    tags.className = "tags";
    (c.tags || []).forEach((t) => tags.appendChild(h("span", { class: "chip" }, t)));
    li.appendChild(tags);
    if (c.archive_url) {
      const ctr = document.createElement("div");
      ctr.style.marginTop = "0.25rem";
      ctr.appendChild(h("span", { style: "font-family:var(--font-mono); font-size:0.75rem; color:var(--ink-ghost); margin-right:0.5rem;" }, `change ID ${c.change_id || ""}`));
      const link = openArchiveLink(c.archive_url);
      if (link) ctr.appendChild(link);
      li.appendChild(ctr);
    }
    ol.appendChild(li);
  });
  return ol;
}

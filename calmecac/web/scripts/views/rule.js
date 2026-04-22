// Rule view — visual gallery (if artefacts) or boring dashboard (non-visual).
// @trace spec:calmecac

import { escapeHtml, h, openArchiveLink, sparkline } from "./_util.js";

export function render(container, ctx, params) {
  const { data } = ctx;
  const name = params.name;
  const rule = (data.rules || []).find((r) => r.name === name);

  container.innerHTML = "";
  if (!rule) {
    container.appendChild(h("div", { class: "empty-state" }, `Rule ${escapeHtml(name)} is not in this index.`));
    return;
  }

  const root = document.createElement("section");
  root.className = "rule-view";

  const head = document.createElement("header");
  head.className = "view-header rule-head";
  head.innerHTML = `
    <div class="eyebrow">Rule</div>
    <h1 class="rule-name">${escapeHtml(rule.name)}</h1>
    ${rule.role ? `<p class="rule-role">${escapeHtml(rule.role)}</p>` : ""}
    <span class="kind-banner">${rule.kind === "visual" ? "Visual rule — gallery" : "Process rule — dashboard"}</span>
  `;
  root.appendChild(head);

  // Neighbours — lessons that cite + rules cited
  const neighbours = document.createElement("div");
  neighbours.className = "neighbours";
  if (rule.lessons && rule.lessons.length) {
    neighbours.appendChild(h("span", { class: "label" }, "Cited by:"));
    rule.lessons.forEach((l) => neighbours.appendChild(h("a", { class: "chip", "data-kind": "lesson", href: `#/lesson/${l.id || l}` }, `@Lesson ${l.short_id || l.id || l}`)));
  }
  if (rule.cites && rule.cites.length) {
    neighbours.appendChild(h("span", { class: "label" }, "Cites:"));
    rule.cites.forEach((r) => neighbours.appendChild(h("a", { class: "chip", "data-kind": "rule", href: `#/rule/${r}` }, `@trace rule:${r}`)));
  }
  if (neighbours.childNodes.length) root.appendChild(neighbours);

  if (rule.kind === "visual" && rule.gallery && rule.gallery.length) {
    const section = document.createElement("section");
    section.appendChild(h("h2", {}, "Gallery"));
    const teach = document.createElement("p");
    teach.className = "teaching";
    teach.textContent = `You are looking at every rendered element that upholds this Rule. ${rule.gallery.length} artefacts so far.`;
    section.appendChild(teach);
    const gallery = document.createElement("div");
    gallery.className = "gallery";
    rule.gallery.forEach((g) => {
      const fig = document.createElement("figure");
      if (g.image_url) {
        const img = document.createElement("img");
        img.src = g.image_url;
        img.alt = g.alt_text || `Artefact citing rule ${rule.name}`;
        img.loading = "lazy";
        fig.appendChild(img);
      }
      if (g.caption) fig.appendChild(h("figcaption", {}, g.caption));
      if (g.strip_episode) {
        const link = document.createElement("a");
        link.href = `#/comic/${g.strip_episode}`;
        link.style.display = "block";
        link.style.padding = "0.25rem 0.75rem";
        link.textContent = `Open Strip ${g.strip_episode}`;
        fig.appendChild(link);
      }
      gallery.appendChild(fig);
    });
    section.appendChild(gallery);
    root.appendChild(section);
  } else {
    // Boring dashboard path.
    const section = document.createElement("section");
    section.appendChild(h("h2", {}, "Convergence dashboard"));
    section.appendChild(h("p", { class: "boring-note" }, "Boring by design — the feedback loop reads this, not you. You can still read it."));
    section.appendChild(renderDashboard(rule));
    root.appendChild(section);
  }

  // Convergence history — changes citing this rule.
  const trail = (data.changes || []).filter((c) => (c.tags || []).some((t) => t === `@trace rule:${rule.name}` || t === `@trace spec:${rule.name}`));
  if (trail.length) {
    const section = document.createElement("section");
    section.className = "convergence-history";
    section.appendChild(h("h2", {}, "Convergence history"));
    section.appendChild(h("p", { class: "meta-row" }, "Changes citing this Rule, in reverse time order."));
    const ol = document.createElement("ol");
    ol.className = "change-list";
    trail.forEach((c) => {
      const li = document.createElement("li");
      li.appendChild(h("span", { class: "date" }, c.date || "—"));
      const body = document.createElement("div");
      body.appendChild(h("div", { class: "subject" }, c.subject || c.summary || "(no summary)"));
      const tags = document.createElement("div");
      tags.className = "tags";
      (c.tags || []).forEach((t) => tags.appendChild(h("span", { class: "chip" }, t)));
      body.appendChild(tags);
      const row = document.createElement("div");
      row.style.marginTop = "0.25rem";
      row.style.display = "flex";
      row.style.alignItems = "center";
      row.style.gap = "0.5rem";
      row.appendChild(h("span", { class: "change-id" }, `change ID ${c.change_id || ""}`));
      const archive = openArchiveLink(c.archive_url);
      if (archive) row.appendChild(archive);
      body.appendChild(row);
      li.appendChild(body);
      ol.appendChild(li);
    });
    section.appendChild(ol);
    root.appendChild(section);
  }

  container.appendChild(root);
}

function renderDashboard(rule) {
  const wrap = document.createElement("div");
  wrap.className = "dashboards";
  wrap.style.display = "grid";
  wrap.style.gap = "0.75rem";
  wrap.style.gridTemplateColumns = "repeat(auto-fill, minmax(240px, 1fr))";

  const series = (rule.convergence || {});
  const entries = [
    { key: "body_length", label: "Body length over changes" },
    { key: "citation_count", label: "Citations over changes" },
    { key: "churn_plus", label: "Additions per change" },
    { key: "churn_minus", label: "Removals per change" },
  ];
  let anyData = false;
  entries.forEach((e) => {
    const values = series[e.key];
    if (!values || values.length === 0) return;
    anyData = true;
    const dash = document.createElement("div");
    dash.className = "dash";
    dash.appendChild(h("h3", {}, e.label));
    const latest = values[values.length - 1];
    const metric = document.createElement("div");
    metric.className = "metric";
    metric.appendChild(h("span", {}, `Most recent`));
    metric.appendChild(h("strong", {}, String(latest)));
    dash.appendChild(metric);
    dash.appendChild(sparkline(values));
    wrap.appendChild(dash);
  });
  if (!anyData) {
    const empty = document.createElement("div");
    empty.className = "empty-state";
    empty.textContent = "(view in progress — content will appear once data is indexed)";
    wrap.appendChild(empty);
  }
  if (rule.last_change_summary) {
    const last = document.createElement("p");
    last.className = "last-change";
    last.textContent = `Last change: ${rule.last_change_summary}`;
    wrap.appendChild(last);
  }
  return wrap;
}

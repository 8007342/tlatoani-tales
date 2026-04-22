// Convergence tab — all-rules convergence at a glance.
// @trace spec:calmecac

import { escapeHtml, h, sparkline } from "./_util.js";

export function render(container, ctx) {
  const { data } = ctx;
  container.innerHTML = "";

  const root = document.createElement("section");
  root.className = "convergence-view";

  const head = document.createElement("header");
  head.className = "view-header";
  head.innerHTML = `
    <div class="eyebrow">Convergence</div>
    <h1>The project converging toward its own Rules.</h1>
    <p class="boring-preamble">Boring by design — the feedback loop reads this, not you. You can still read it.</p>
  `;
  root.appendChild(head);

  if (data.last_change && data.last_change.subject) {
    const banner = document.createElement("aside");
    banner.className = "teaching";
    banner.innerHTML = `Last change: <em>${escapeHtml(data.last_change.subject)}</em> on ${escapeHtml(data.last_change.date || "—")}.`;
    root.appendChild(banner);
  }

  const dashboards = document.createElement("div");
  dashboards.className = "dashboards";

  (data.rules || []).forEach((rule) => {
    const series = (rule.convergence || {});
    const values = series.body_length || series.citation_count || [];
    const dash = document.createElement("article");
    dash.className = "dash";
    const title = document.createElement("h3");
    const link = document.createElement("a");
    link.href = `#/rule/${rule.name}`;
    link.textContent = rule.name;
    link.style.fontFamily = "var(--font-mono)";
    title.appendChild(link);
    dash.appendChild(title);
    if (rule.role) dash.appendChild(h("p", { class: "meta-row" }, rule.role));
    if (values.length) {
      const metric = document.createElement("div");
      metric.className = "metric";
      metric.appendChild(h("span", {}, series.body_length ? "Body length" : "Citations"));
      metric.appendChild(h("strong", {}, String(values[values.length - 1])));
      dash.appendChild(metric);
      dash.appendChild(sparkline(values));
    } else {
      dash.appendChild(h("p", { class: "meta-row" }, "(no series yet)"));
    }
    dashboards.appendChild(dash);
  });

  if (!dashboards.childNodes.length) {
    dashboards.appendChild(h("div", { class: "empty-state" }, "(view in progress — content will appear once Rules have indexed series)"));
  }

  root.appendChild(dashboards);

  // Repo-wide changes summary (concept-level).
  const changes = data.changes || [];
  if (changes.length) {
    const section = document.createElement("section");
    section.appendChild(h("h2", {}, "Changes across the archive"));
    section.appendChild(h("p", {}, "Each change is shown by its concept-level subject and the tags it carries."));
    const ol = document.createElement("ol");
    ol.style.listStyle = "none";
    ol.style.padding = "0";
    ol.style.display = "grid";
    ol.style.gap = "0.5rem";
    changes.slice(0, 50).forEach((c) => {
      const li = document.createElement("li");
      li.className = "card";
      li.innerHTML = `
        <div class="meta-row"><strong>${escapeHtml(c.date || "—")}</strong></div>
        <p>${escapeHtml(c.subject || c.summary || "(no summary)")}</p>
      `;
      const tags = document.createElement("div");
      tags.className = "tags";
      tags.style.display = "flex";
      tags.style.flexWrap = "wrap";
      tags.style.gap = "0.25rem";
      (c.tags || []).forEach((t) => tags.appendChild(h("span", { class: "chip" }, t)));
      li.appendChild(tags);
      ol.appendChild(li);
    });
    section.appendChild(ol);
    root.appendChild(section);
  }

  container.appendChild(root);
}

// Comic view — the strip with plate hotspots overlaid.
// @trace spec:calmecac, spec:trace-plate

import { escapeHtml, h } from "./_util.js";

export function render(container, ctx, params) {
  const { data } = ctx;
  const episode = params.episode;
  const strip = (data.strips || []).find((s) => String(s.episode) === String(episode));

  container.innerHTML = "";
  if (!strip) {
    container.appendChild(h("div", { class: "empty-state" }, `Strip ${escapeHtml(episode)} has not been indexed yet.`));
    return;
  }

  const section = document.createElement("section");
  section.className = "comic-view";

  const header = document.createElement("header");
  header.className = "view-header";
  header.innerHTML = `
    <div class="eyebrow">Strip ${escapeHtml(strip.episode)}${strip.episode_total ? "/" + escapeHtml(strip.episode_total) : ""}</div>
    <h1>${escapeHtml(strip.title || "")}</h1>
    ${strip.subtitle ? `<p class="caption">${escapeHtml(strip.subtitle)}</p>` : ""}
  `;
  section.appendChild(header);

  const teaching = document.createElement("p");
  teaching.className = "teaching";
  teaching.textContent = "You are looking at a published strip. The three plates are doors: the Lesson, the Rule, and the episode pointer.";
  section.appendChild(teaching);

  // Quick links (plate destinations as buttons — also accessible on keyboard).
  const quickLinks = document.createElement("div");
  quickLinks.className = "quick-links";
  if (strip.primary_lesson) {
    quickLinks.appendChild(h("a", { class: "chip", "data-kind": "lesson", href: `#/lesson/${strip.primary_lesson}` }, `@Lesson ${strip.primary_lesson}`));
  }
  if (strip.primary_rule) {
    quickLinks.appendChild(h("a", { class: "chip", "data-kind": "rule", href: `#/rule/${strip.primary_rule}` }, `@trace rule:${strip.primary_rule}`));
  }
  (strip.reinforced_lessons || []).forEach((id) => {
    quickLinks.appendChild(h("a", { class: "chip", "data-kind": "lesson", href: `#/lesson/${id}` }, `reinforces ${id}`));
  });
  section.appendChild(quickLinks);

  // Strip image + hotspot overlay.
  if (strip.image_url) {
    const stripWrap = document.createElement("figure");
    stripWrap.className = "strip";
    const img = document.createElement("img");
    img.src = strip.image_url;
    img.alt = strip.alt_text || `Tlatoāni Tales strip ${strip.episode}: ${strip.title || ""}`;
    img.loading = "lazy";
    stripWrap.appendChild(img);

    if (strip.plate_regions && strip.image_width && strip.image_height) {
      const hotspots = document.createElement("div");
      hotspots.className = "hotspots";
      const W = strip.image_width, H = strip.image_height;
      const pct = (r) => ({
        left: (r.x / W * 100) + "%",
        top: (r.y / H * 100) + "%",
        width: (r.w / W * 100) + "%",
        height: (r.h / H * 100) + "%",
      });
      const lessonLine = strip.plate_regions.trace_lesson && strip.plate_regions.trace_lesson.lesson_line;
      const traceLine = strip.plate_regions.trace_lesson && strip.plate_regions.trace_lesson.trace_line;
      const titlePlate = strip.plate_regions.title;
      const episodePlate = strip.plate_regions.episode;

      if (titlePlate && strip.title_linkable && strip.primary_lesson) {
        hotspots.appendChild(buildHotspot(pct(titlePlate), `#/lesson/${strip.primary_lesson}`, `Open Lesson ${strip.primary_lesson}`));
      }
      if (lessonLine && strip.primary_lesson) {
        hotspots.appendChild(buildHotspot(pct(lessonLine), `#/lesson/${strip.primary_lesson}`, `Open Lesson ${strip.primary_lesson}`));
      }
      if (traceLine && strip.primary_rule) {
        hotspots.appendChild(buildHotspot(pct(traceLine), `#/rule/${strip.primary_rule}`, `Open Rule ${strip.primary_rule}`));
      }
      if (episodePlate) {
        // Episode plate stays on the public strip page — it leaves Calmecac.
        // We render it as a non-navigating span to keep the reader in the
        // viewer unless they explicitly ask.
      }
      stripWrap.appendChild(hotspots);
    }
    section.appendChild(stripWrap);
  } else {
    section.appendChild(h("div", { class: "empty-state" }, "(view in progress — content will appear once the strip is rendered and indexed)"));
  }

  // Strip metadata.
  const meta = document.createElement("div");
  meta.className = "strip-meta";
  const dl = document.createElement("dl");
  const metaRows = [];
  if (strip.date) metaRows.push(["First published", strip.date]);
  if (strip.primary_lesson) metaRows.push(["Primary Lesson", strip.primary_lesson]);
  if (strip.primary_rule) metaRows.push(["Primary Rule", strip.primary_rule]);
  if ((strip.reinforced_lessons || []).length) metaRows.push(["Reinforces", strip.reinforced_lessons.join(", ")]);
  metaRows.forEach(([k, v]) => {
    dl.appendChild(h("dt", {}, k));
    dl.appendChild(h("dd", {}, String(v)));
  });
  meta.appendChild(dl);
  section.appendChild(meta);

  // Argument trail (changes touching this strip).
  const trail = (data.changes || []).filter((c) => (c.touches_strips || []).some((e) => String(e) === String(episode)));
  if (trail.length) {
    const trailSection = document.createElement("section");
    trailSection.innerHTML = `<h2>How this strip arrived at this state</h2>`;
    trailSection.appendChild(renderChangeList(trail));
    section.appendChild(trailSection);
  }

  container.appendChild(section);
}

function buildHotspot(pct, href, label) {
  const a = document.createElement("a");
  a.className = "hotspot";
  a.href = href;
  a.style.left = pct.left;
  a.style.top = pct.top;
  a.style.width = pct.width;
  a.style.height = pct.height;
  a.setAttribute("aria-label", label);
  const lbl = document.createElement("span");
  lbl.className = "label";
  lbl.textContent = label;
  a.appendChild(lbl);
  return a;
}

function renderChangeList(changes) {
  const ol = document.createElement("ol");
  ol.className = "change-list";
  ol.style.listStyle = "none";
  ol.style.padding = "0";
  changes.forEach((c) => {
    const li = document.createElement("li");
    li.className = "card";
    li.style.marginBottom = "0.5rem";
    const date = c.date || "—";
    const subject = c.subject || c.summary || "(no summary)";
    li.innerHTML = `
      <div class="meta-row"><strong>${escapeHtml(date)}</strong> — change ID <code>${escapeHtml(c.change_id || "")}</code></div>
      <p>${escapeHtml(subject)}</p>
    `;
    const tags = document.createElement("div");
    tags.className = "tags";
    tags.style.display = "flex";
    tags.style.flexWrap = "wrap";
    tags.style.gap = "0.25rem";
    (c.tags || []).forEach((t) => {
      const chip = document.createElement("span");
      chip.className = "chip";
      chip.textContent = t;
      tags.appendChild(chip);
    });
    li.appendChild(tags);
    if (c.archive_url) {
      const btn = document.createElement("a");
      btn.className = "btn btn-archive";
      btn.href = c.archive_url;
      btn.target = "_blank";
      btn.rel = "noopener";
      btn.textContent = "Open on archive";
      li.appendChild(btn);
    }
    ol.appendChild(li);
  });
  return ol;
}

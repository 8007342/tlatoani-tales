// Tiny DOM helpers shared by views. Pure vanilla.
// @trace spec:calmecac

export function escapeHtml(s) {
  return String(s == null ? "" : s).replace(/[&<>"']/g, (c) => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;"
  }[c]));
}

export function h(tag, attrs = {}, children) {
  const el = document.createElement(tag);
  for (const k in attrs) {
    if (k === "class") el.className = attrs[k];
    else if (k === "html") el.innerHTML = attrs[k];
    else if (k.startsWith("on") && typeof attrs[k] === "function") el.addEventListener(k.slice(2), attrs[k]);
    else if (attrs[k] !== false && attrs[k] != null) el.setAttribute(k, attrs[k]);
  }
  if (children != null) {
    if (Array.isArray(children)) children.forEach((c) => { if (c != null) el.append(c); });
    else el.append(children);
  }
  return el;
}

export function openArchiveLink(url, label = "Open on archive") {
  if (!url) return null;
  const a = document.createElement("a");
  a.className = "btn btn-archive";
  a.href = url;
  a.target = "_blank";
  a.rel = "noopener";
  a.textContent = label;
  return a;
}

// Build a tiny sparkline SVG from a numeric series.
export function sparkline(values, { width = 240, height = 50, pad = 4 } = {}) {
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("class", "spark");
  svg.setAttribute("viewBox", `0 0 ${width} ${height}`);
  svg.setAttribute("role", "img");
  svg.setAttribute("aria-label", `Series of ${values.length} values`);
  if (!values || values.length === 0) return svg;
  const min = Math.min(...values), max = Math.max(...values);
  const range = Math.max(1, max - min);
  const step = (width - pad * 2) / Math.max(1, values.length - 1);
  const y = (v) => height - pad - ((v - min) / range) * (height - pad * 2);
  const d = values.map((v, i) => `${i === 0 ? "M" : "L"}${pad + i * step},${y(v)}`).join(" ");
  const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
  path.setAttribute("d", d);
  svg.appendChild(path);
  const last = values[values.length - 1];
  const dot = document.createElementNS("http://www.w3.org/2000/svg", "circle");
  dot.setAttribute("cx", pad + (values.length - 1) * step);
  dot.setAttribute("cy", y(last));
  dot.setAttribute("r", 2.5);
  svg.appendChild(dot);
  return svg;
}

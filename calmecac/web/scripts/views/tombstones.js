// Tombstones view — greyed entries with successor arrows, never hidden.
// @trace spec:calmecac, spec:tombstones

import { escapeHtml, h } from "./_util.js";

export function render(container, ctx) {
  const { data } = ctx;
  container.innerHTML = "";

  const root = document.createElement("section");
  root.className = "tombstones-view";

  const head = document.createElement("header");
  head.className = "view-header";
  head.innerHTML = `
    <div class="eyebrow">Tombstones</div>
    <h1>Retired entries — still legible.</h1>
  `;
  root.appendChild(head);

  const preamble = document.createElement("p");
  preamble.className = "preamble";
  preamble.textContent = "A retired identifier is not a deleted one. It stays readable so every past citation can still find its way forward.";
  root.appendChild(preamble);

  const entries = data.tombstones || [];
  if (!entries.length) {
    root.appendChild(h("div", { class: "empty-state" }, "No tombstones in this index yet."));
    container.appendChild(root);
    return;
  }

  const table = document.createElement("table");
  table.innerHTML = `
    <thead>
      <tr>
        <th scope="col">Retired ID</th>
        <th scope="col">Date</th>
        <th scope="col">Reason</th>
        <th scope="col">Successor</th>
      </tr>
    </thead>
  `;
  const tbody = document.createElement("tbody");
  entries.forEach((t) => {
    const tr = document.createElement("tr");
    tr.appendChild(h("td", { class: "retired-id" }, t.id || "—"));
    tr.appendChild(h("td", { class: "date" }, t.date || "—"));
    tr.appendChild(h("td", { class: "reason" }, t.reason || ""));
    const successorTd = document.createElement("td");
    successorTd.className = "successor";
    if (t.successor) {
      const href = t.successor_kind === "rule" ? `#/rule/${t.successor}`
        : t.successor_kind === "lesson" ? `#/lesson/${t.successor}`
        : null;
      if (href) {
        const a = document.createElement("a");
        a.href = href;
        a.textContent = `→ ${t.successor}`;
        successorTd.appendChild(a);
      } else {
        successorTd.textContent = `→ ${t.successor}`;
      }
    } else {
      successorTd.innerHTML = `<span class="decline">declined — no successor</span>`;
    }
    tr.appendChild(successorTd);
    tbody.appendChild(tr);
  });
  table.appendChild(tbody);
  root.appendChild(table);

  container.appendChild(root);
}

// Breadcrumb rendering — never lose the reader.
// @trace spec:calmecac

export function renderBreadcrumbs(container, trail) {
  if (!container) return;
  container.innerHTML = "";
  if (!trail || trail.length === 0) return;
  const ol = document.createElement("ol");
  trail.forEach((entry, i) => {
    const li = document.createElement("li");
    if (entry.href && i !== trail.length - 1) {
      const a = document.createElement("a");
      a.href = entry.href;
      a.textContent = entry.label;
      li.appendChild(a);
    } else {
      const span = document.createElement("span");
      span.textContent = entry.label;
      if (i === trail.length - 1) span.setAttribute("aria-current", "page");
      li.appendChild(span);
    }
    ol.appendChild(li);
  });
  container.appendChild(ol);
}

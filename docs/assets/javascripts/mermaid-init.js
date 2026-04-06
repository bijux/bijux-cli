window.mermaidConfig = {
  startOnLoad: false,
  securityLevel: "loose",
};

document$.subscribe(() => {
  if (typeof mermaid === "undefined") {
    return;
  }

  mermaid.initialize(window.mermaidConfig);
  mermaid.run({
    querySelector: "pre.mermaid code, div.mermaid, code.mermaid",
  });
});

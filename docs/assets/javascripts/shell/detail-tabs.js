(function () {
  const shell = (window.bijuxShell = window.bijuxShell || {});

  function syncDetailStripVisibility() {
    const navState = shell.navState;
    if (!navState) {
      return;
    }

    const activeSitePath = navState.syncSiteTabActiveState();
    const strips = document.querySelectorAll("[data-bijux-detail-strip]");

    for (const strip of strips) {
      const rootPath = navState.normalizePath(
        strip.getAttribute("data-bijux-detail-root-path") || "/"
      );
      strip.hidden = rootPath !== activeSitePath;
    }

    const header = document.querySelector("[data-md-component='header']");
    if (!header) {
      return;
    }
    const hasVisibleDetailStrip = document.querySelector(
      "[data-bijux-detail-strip]:not([hidden])"
    );
    header.setAttribute(
      "data-bijux-detail-visible",
      hasVisibleDetailStrip ? "true" : "false"
    );
  }

  function syncDetailStripActiveState() {
    const navState = shell.navState;
    if (!navState) {
      return;
    }

    const activeStrip = document.querySelector(
      "[data-bijux-detail-strip]:not([hidden])"
    );
    const currentPath = navState.normalizePath(window.location.pathname);

    if (!activeStrip) {
      return;
    }

    const authoredActiveLink = activeStrip.querySelector(
      "a[data-bijux-detail-path][aria-current='page'], .bijux-tabs__item--active a[data-bijux-detail-path]"
    );

    for (const item of activeStrip.querySelectorAll(".bijux-tabs__item")) {
      item.classList.remove("bijux-tabs__item--active");
    }

    for (const link of activeStrip.querySelectorAll("a[data-bijux-detail-path]")) {
      link.removeAttribute("aria-current");
    }

    const matchedLink = navState.bestMatchingLink(
      activeStrip,
      "data-bijux-detail-path",
      currentPath,
      "a[data-bijux-detail-path]"
    );

    let activeLink = matchedLink;

    if (!activeLink && authoredActiveLink) {
      activeLink = {
        path: navState.normalizePath(
          authoredActiveLink.getAttribute("data-bijux-detail-path") || "/"
        ),
        node: authoredActiveLink,
      };
    }

    if (!activeLink) {
      const rootPath = navState.normalizePath(
        activeStrip.getAttribute("data-bijux-detail-root-path") || "/"
      );
      const homeLink = activeStrip.querySelector(
        `a[data-bijux-detail-path="${rootPath}"]`
      );
      if (homeLink) {
        activeLink = {
          path: rootPath,
          node: homeLink,
        };
      }
    }

    if (!activeLink) {
      const firstLink = activeStrip.querySelector("a[data-bijux-detail-path]");
      if (firstLink) {
        activeLink = {
          path: navState.normalizePath(
            firstLink.getAttribute("data-bijux-detail-path") || "/"
          ),
          node: firstLink,
        };
      }
    }

    if (activeLink) {
      activeLink.node.closest(".bijux-tabs__item")?.classList.add("bijux-tabs__item--active");
      activeLink.node.setAttribute("aria-current", "page");
    }
  }

  function bindDetailSelectNavigation() {
    for (const select of document.querySelectorAll("[data-bijux-detail-select]")) {
      if (select.dataset.bijuxDetailSelectBound === "true") {
        continue;
      }

      select.dataset.bijuxDetailSelectBound = "true";
      select.addEventListener("change", () => {
        if (!select.value) {
          return;
        }
        window.location.href = select.value;
      });
    }
  }

  function syncDetailSelectState() {
    const navState = shell.navState;
    if (!navState) {
      return;
    }

    const activeStrip = document.querySelector(
      "[data-bijux-detail-strip]:not([hidden])"
    );
    if (!activeStrip) {
      return;
    }

    const activeLink = activeStrip.querySelector(
      "a[data-bijux-detail-path][aria-current='page']"
    );

    if (!activeLink) {
      return;
    }

    const activePath = navState.normalizePath(
      activeLink.getAttribute("data-bijux-detail-path") || "/"
    );

    const select = activeStrip.querySelector("[data-bijux-detail-select]");
    if (!select) {
      return;
    }

    for (const option of select.options) {
      const optionPath = navState.normalizePath(
        option.getAttribute("data-bijux-detail-path") || option.value || "/"
      );
      option.selected = optionPath === activePath;
    }
  }

  function runDetailTabsSync() {
    syncDetailStripVisibility();
    syncDetailStripActiveState();
    syncDetailSelectState();
    bindDetailSelectNavigation();
  }

  shell.detailTabs = {
    syncDetailStripVisibility,
    syncDetailStripActiveState,
    syncDetailSelectState,
    bindDetailSelectNavigation,
    runDetailTabsSync,
  };
})();

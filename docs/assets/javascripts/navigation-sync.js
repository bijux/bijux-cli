function bijuxSiteBasePath() {
  const scopePath = window.__md_scope?.pathname;
  if (!scopePath) {
    return "";
  }
  const path = scopePath.replace(/\/+$/, "");
  return path === "/" ? "" : path;
}

function bijuxNormalizeNavPath(target) {
  const url = new URL(target, window.location.href);
  const path = url.pathname.replace(/\/+$/, "");
  return path || "/";
}

function bijuxNormalizePath(target) {
  const url = new URL(target, window.location.href);
  const basePath = bijuxSiteBasePath();
  let path = url.pathname.replace(/\/+$/, "");
  if (basePath && (path === basePath || path.startsWith(`${basePath}/`))) {
    path = path.slice(basePath.length) || "/";
  }
  return path || "/";
}

const BIJUX_DETAIL_SELECTION_KEY = "bijux.detailSelectionBySite";

function bijuxReadDetailSelectionMap() {
  try {
    return JSON.parse(window.sessionStorage.getItem(BIJUX_DETAIL_SELECTION_KEY) || "{}");
  } catch (_error) {
    return {};
  }
}

function bijuxReadDetailSelection(sitePath) {
  if (!sitePath) {
    return null;
  }
  const state = bijuxReadDetailSelectionMap();
  return typeof state[sitePath] === "string" ? state[sitePath] : null;
}

function bijuxWriteDetailSelection(sitePath, detailPath) {
  if (!sitePath || !detailPath) {
    return;
  }
  const state = bijuxReadDetailSelectionMap();
  state[sitePath] = detailPath;
  try {
    window.sessionStorage.setItem(BIJUX_DETAIL_SELECTION_KEY, JSON.stringify(state));
  } catch (_error) {
    // Ignore persistence failures and keep navigation behavior functional.
  }
}

function bijuxBestMatchingLink(links, pathAttribute) {
  const currentPath = bijuxNormalizePath(window.location.pathname);
  let activeLink = null;

  for (const link of links) {
    const linkPath = bijuxNormalizePath(link.getAttribute(pathAttribute) || "/");
    const isMatch =
      currentPath === linkPath ||
      (linkPath !== "/" && currentPath.startsWith(`${linkPath}/`));

    if (isMatch && (!activeLink || linkPath.length > activeLink.path.length)) {
      activeLink = { path: linkPath, node: link };
    }
  }

  return activeLink;
}

function bijuxResolveLinkPath(link, pathAttribute) {
  const href = link.getAttribute("href");
  if (href) {
    return bijuxNormalizeNavPath(href);
  }
  return bijuxNormalizeNavPath(link.getAttribute(pathAttribute) || "/");
}

function bijuxResolveDetailStripRootPath(strip) {
  const firstLink = strip.querySelector("[data-bijux-detail-path]");
  if (firstLink?.getAttribute("href")) {
    return bijuxNormalizeNavPath(firstLink.getAttribute("href"));
  }
  return bijuxNormalizeNavPath(
    strip.getAttribute("data-bijux-detail-root-path") || "/"
  );
}

function bijuxBestSitePath() {
  const currentPath = bijuxNormalizeNavPath(window.location.pathname);
  let activeLink = null;

  for (const link of document.querySelectorAll(
    ".bijux-site-tabs [data-bijux-site-path]"
  )) {
    const linkPath = bijuxResolveLinkPath(link, "data-bijux-site-path");
    const isMatch =
      currentPath === linkPath ||
      (linkPath !== "/" && currentPath.startsWith(`${linkPath}/`));

    if (isMatch && (!activeLink || linkPath.length > activeLink.path.length)) {
      activeLink = { path: linkPath, node: link };
    }
  }

  if (activeLink) {
    return activeLink.path;
  }

  const authoredActiveLink = document.querySelector(
    ".bijux-site-tabs [data-bijux-site-path][aria-current='page'], .bijux-site-tabs .bijux-tabs__item--active [data-bijux-site-path]"
  );
  if (authoredActiveLink) {
    return bijuxResolveLinkPath(authoredActiveLink, "data-bijux-site-path");
  }
  return null;
}

function bijuxSyncSiteTabActiveState() {
  const activeSitePath = bijuxBestSitePath();

  for (const item of document.querySelectorAll(".bijux-site-tabs .bijux-tabs__item")) {
    item.classList.remove("md-tabs__item--active", "bijux-tabs__item--active");
  }
  for (const link of document.querySelectorAll(
    ".bijux-site-tabs [data-bijux-site-path]"
  )) {
    link.removeAttribute("aria-current");
  }

  if (!activeSitePath) {
    return null;
  }

  for (const link of document.querySelectorAll(
    ".bijux-site-tabs [data-bijux-site-path]"
  )) {
    const linkPath = bijuxResolveLinkPath(link, "data-bijux-site-path");
    if (linkPath === activeSitePath) {
      link.closest(".bijux-tabs__item")?.classList.add(
        "md-tabs__item--active",
        "bijux-tabs__item--active"
      );
      link.setAttribute("aria-current", "page");
    }
  }

  return activeSitePath;
}

function bijuxSyncDetailStripVisibility() {
  const activeSitePath = bijuxSyncSiteTabActiveState();
  const strips = document.querySelectorAll("[data-bijux-detail-strip]");

  for (const strip of strips) {
    const rootPath = bijuxResolveDetailStripRootPath(strip);
    strip.hidden = rootPath !== activeSitePath;
  }
}

function bijuxSyncDetailStripActiveState() {
  const activeStrip = document.querySelector("[data-bijux-detail-strip]:not([hidden])");
  const currentPath = bijuxNormalizeNavPath(window.location.pathname);
  const sitePath = activeStrip ? bijuxResolveDetailStripRootPath(activeStrip) : "/";
  const preferredPath = bijuxReadDetailSelection(sitePath);
  const authoredActiveLink = activeStrip?.querySelector(
    "[data-bijux-detail-path][aria-current='page'], .bijux-tabs__item--active [data-bijux-detail-path]"
  );

  for (const item of document.querySelectorAll(
    "[data-bijux-detail-strip] .bijux-tabs__item"
  )) {
    item.classList.remove("bijux-tabs__item--active");
  }
  for (const link of document.querySelectorAll(
    "[data-bijux-detail-strip] [data-bijux-detail-path]"
  )) {
    link.removeAttribute("aria-current");
  }

  if (!activeStrip) {
    return;
  }

  let activeLink = null;
  if (preferredPath) {
    const preferredNode = activeStrip.querySelector(
      `[data-bijux-detail-path="${preferredPath}"]`
    );
    if (preferredNode) {
      activeLink = { path: preferredPath, node: preferredNode };
    }
  }

  if (!activeLink) {
    for (const link of activeStrip.querySelectorAll("[data-bijux-detail-path]")) {
      const linkPath = bijuxResolveLinkPath(link, "data-bijux-detail-path");
      const isMatch =
        currentPath === linkPath ||
        (linkPath !== "/" && currentPath.startsWith(`${linkPath}/`));

      if (isMatch && (!activeLink || linkPath.length > activeLink.path.length)) {
        activeLink = { path: linkPath, node: link };
      }
    }
  }

  if (!activeLink && authoredActiveLink) {
    activeLink = {
      path: bijuxResolveLinkPath(authoredActiveLink, "data-bijux-detail-path"),
      node: authoredActiveLink,
    };
  }

  if (activeLink) {
    activeLink.node.closest(".bijux-tabs__item")?.classList.add(
      "bijux-tabs__item--active"
    );
    activeLink.node.setAttribute("aria-current", "page");
    bijuxWriteDetailSelection(sitePath, activeLink.path);
  }
}

function bijuxRevealActiveNavigationTarget() {
  const activeHubLink = document.querySelector(
    ".bijux-hub-strip .bijux-tabs__item--active a"
  );
  const activeSiteLink = document.querySelector(
    ".bijux-site-tabs .bijux-tabs__item--active a"
  );
  const activeDetailLink = document.querySelector(
    "[data-bijux-detail-strip]:not([hidden]) .bijux-tabs__item--active a"
  );
  const activeSidebarLink = document.querySelector(
    ".md-sidebar--primary .md-nav__link--active"
  );

  activeHubLink?.scrollIntoView({
    block: "nearest",
    inline: "center",
  });
  activeSiteLink?.scrollIntoView({
    block: "nearest",
    inline: "center",
  });
  activeDetailLink?.scrollIntoView({
    block: "nearest",
    inline: "center",
  });
  activeSidebarLink?.scrollIntoView({
    block: "nearest",
    inline: "nearest",
  });
}

function bijuxRevealMobileDrawerContext() {
  if (!window.matchMedia("(max-width: 76.2344em)").matches) {
    return;
  }

  const activeMobileLink = document.querySelector(
    ".md-sidebar--primary .bijux-nav--mobile .md-nav__link--active, " +
      ".md-sidebar--primary .bijux-nav--mobile .md-nav__item--active > .md-nav__container > .md-nav__link, " +
      ".md-sidebar--primary .bijux-nav--mobile .md-nav__item--active > .md-nav__link"
  );

  activeMobileLink?.scrollIntoView({
    behavior: "auto",
    block: "center",
    inline: "nearest",
  });
}

function bijuxBindMobileDrawerReveal() {
  const drawerToggle = document.querySelector("#__drawer");
  if (!drawerToggle || drawerToggle.dataset.bijuxRevealBound === "true") {
    return;
  }

  drawerToggle.dataset.bijuxRevealBound = "true";
  drawerToggle.addEventListener("change", () => {
    if (!drawerToggle.checked) {
      return;
    }

    window.setTimeout(() => {
      bijuxRevealMobileDrawerContext();
    }, 180);
  });
}

function bijuxBindDetailStripSelectionPersistence() {
  for (const link of document.querySelectorAll("[data-bijux-detail-strip] [data-bijux-detail-path]")) {
    if (link.dataset.bijuxDetailSelectionBound === "true") {
      continue;
    }
    link.dataset.bijuxDetailSelectionBound = "true";
    link.addEventListener("click", () => {
      const strip = link.closest("[data-bijux-detail-strip]");
      const sitePath = strip ? bijuxResolveDetailStripRootPath(strip) : "/";
      const detailPath = bijuxResolveLinkPath(link, "data-bijux-detail-path");
      bijuxWriteDetailSelection(sitePath, detailPath);
    });
  }
}

document$.subscribe(() => {
  bijuxBindDetailStripSelectionPersistence();
  bijuxSyncDetailStripVisibility();
  bijuxSyncDetailStripActiveState();
  bijuxRevealActiveNavigationTarget();
  bijuxBindMobileDrawerReveal();
});

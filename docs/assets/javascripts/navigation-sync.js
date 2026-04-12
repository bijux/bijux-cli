function bijuxSiteBasePath() {
  const scopePath = window.__md_scope?.pathname;
  if (!scopePath) {
    return "";
  }
  const path = scopePath.replace(/\/+$/, "");
  return path === "/" ? "" : path;
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

function bijuxBestSitePath() {
  const activeLink = bijuxBestMatchingLink(
    document.querySelectorAll(".bijux-site-tabs [data-bijux-site-path]"),
    "data-bijux-site-path"
  );
  if (activeLink) {
    return activeLink.path;
  }

  const authoredActiveLink = document.querySelector(
    ".bijux-site-tabs [data-bijux-site-path][aria-current='page'], .bijux-site-tabs .bijux-tabs__item--active [data-bijux-site-path]"
  );
  if (authoredActiveLink) {
    return bijuxNormalizePath(
      authoredActiveLink.getAttribute("data-bijux-site-path") || "/"
    );
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
    const linkPath = bijuxNormalizePath(
      link.getAttribute("data-bijux-site-path") || "/"
    );
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
    const rootPath = bijuxNormalizePath(
      strip.getAttribute("data-bijux-detail-root-path") || "/"
    );
    strip.hidden = rootPath !== activeSitePath;
  }
}

function bijuxSyncDetailStripActiveState() {
  const activeStrip = document.querySelector("[data-bijux-detail-strip]:not([hidden])");
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

  let activeLink = bijuxBestMatchingLink(
    activeStrip.querySelectorAll("[data-bijux-detail-path]"),
    "data-bijux-detail-path"
  );

  if (!activeLink && authoredActiveLink) {
    activeLink = {
      path: bijuxNormalizePath(
        authoredActiveLink.getAttribute("data-bijux-detail-path") || "/"
      ),
      node: authoredActiveLink,
    };
  }

  if (activeLink) {
    activeLink.node.closest(".bijux-tabs__item")?.classList.add(
      "bijux-tabs__item--active"
    );
    activeLink.node.setAttribute("aria-current", "page");
  }
}

function bijuxActiveDetailPath() {
  const activeStrip = document.querySelector("[data-bijux-detail-strip]:not([hidden])");
  const authoredActiveLink = activeStrip?.querySelector(
    "[data-bijux-detail-path][aria-current='page'], .bijux-tabs__item--active [data-bijux-detail-path]"
  );

  if (!activeStrip) {
    return null;
  }

  const activeLink = bijuxBestMatchingLink(
    activeStrip.querySelectorAll("[data-bijux-detail-path]"),
    "data-bijux-detail-path"
  );
  if (activeLink) {
    return activeLink.path;
  }
  if (authoredActiveLink) {
    return bijuxNormalizePath(
      authoredActiveLink.getAttribute("data-bijux-detail-path") || "/"
    );
  }
  return null;
}

function bijuxSyncCourseStripVisibility() {
  const activeDetailPath = bijuxActiveDetailPath();
  const strips = document.querySelectorAll("[data-bijux-course-strip]");

  for (const strip of strips) {
    const rootPath = bijuxNormalizePath(
      strip.getAttribute("data-bijux-course-root-path") || "/"
    );
    const activeCourseLink = bijuxBestMatchingLink(
      strip.querySelectorAll("[data-bijux-course-path]"),
      "data-bijux-course-path"
    );
    strip.hidden = rootPath !== activeDetailPath && !activeCourseLink;
  }
}

function bijuxSyncCourseStripActiveState() {
  const activeStrip = document.querySelector("[data-bijux-course-strip]:not([hidden])");
  const authoredActiveLink = activeStrip?.querySelector(
    "[data-bijux-course-path][aria-current='page'], .bijux-tabs__item--active [data-bijux-course-path]"
  );

  for (const item of document.querySelectorAll(
    "[data-bijux-course-strip] .bijux-tabs__item"
  )) {
    item.classList.remove("bijux-tabs__item--active");
  }
  for (const link of document.querySelectorAll(
    "[data-bijux-course-strip] [data-bijux-course-path]"
  )) {
    link.removeAttribute("aria-current");
  }

  if (!activeStrip) {
    return;
  }

  let activeLink = bijuxBestMatchingLink(
    activeStrip.querySelectorAll("[data-bijux-course-path]"),
    "data-bijux-course-path"
  );

  if (!activeLink && authoredActiveLink) {
    activeLink = {
      path: bijuxNormalizePath(
        authoredActiveLink.getAttribute("data-bijux-course-path") || "/"
      ),
      node: authoredActiveLink,
    };
  }

  if (activeLink) {
    activeLink.node.closest(".bijux-tabs__item")?.classList.add(
      "bijux-tabs__item--active"
    );
    activeLink.node.setAttribute("aria-current", "page");
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
  const activeCourseLink = document.querySelector(
    "[data-bijux-course-strip] .bijux-tabs__item--active a"
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
  activeCourseLink?.scrollIntoView({
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

document$.subscribe(() => {
  bijuxSyncDetailStripVisibility();
  bijuxSyncDetailStripActiveState();
  bijuxSyncCourseStripVisibility();
  bijuxSyncCourseStripActiveState();
  bijuxRevealActiveNavigationTarget();
  bijuxBindMobileDrawerReveal();
});

(function () {
  const shell = (window.bijuxShell = window.bijuxShell || {});
  let viewportRevealBound = false;

  function centerLinkInScrollableTabs(link) {
    if (!link) {
      return;
    }

    const list = link.closest(".bijux-tabs__list");
    if (!list) {
      link.scrollIntoView({ block: "nearest", inline: "center" });
      return;
    }

    const listRect = list.getBoundingClientRect();
    const linkRect = link.getBoundingClientRect();
    const target =
      list.scrollLeft + (linkRect.left - listRect.left) - listRect.width / 2 + linkRect.width / 2;

    list.scrollTo({
      left: Math.max(0, target),
      behavior: "auto",
    });
  }

  function revealActiveNavigationTarget() {
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
      "[data-bijux-course-strip]:not([hidden]) .bijux-tabs__item--active a"
    );
    const activeSidebarLink = document.querySelector(
      ".md-sidebar--primary .md-nav__link--active"
    );

    centerLinkInScrollableTabs(activeHubLink);
    centerLinkInScrollableTabs(activeSiteLink);
    centerLinkInScrollableTabs(activeDetailLink);
    centerLinkInScrollableTabs(activeCourseLink);
    activeSidebarLink?.scrollIntoView({ block: "nearest", inline: "nearest" });
  }

  function revealAfterLayoutSettles() {
    revealActiveNavigationTarget();
    window.setTimeout(revealActiveNavigationTarget, 120);
    window.setTimeout(revealActiveNavigationTarget, 320);
  }

  function runDesktopNavigationSync() {
    revealAfterLayoutSettles();
  }

  function runPhoneNavigationSync() {
    bindViewportReveal();
    revealMobileDrawerContext();
  }

  function resolveViewportMode() {
    if (window.bijuxViewportProfile && typeof window.bijuxViewportProfile.current === "function") {
      return window.bijuxViewportProfile.current();
    }

    const fallbackWidth =
      window.visualViewport && typeof window.visualViewport.width === "number"
        ? window.visualViewport.width
        : window.innerWidth;

    if (
      window.bijuxViewportProfile &&
      typeof window.bijuxViewportProfile.classifyWidth === "function"
    ) {
      return window.bijuxViewportProfile.classifyWidth(fallbackWidth);
    }

    if (typeof window.matchMedia === "function" && window.matchMedia("(max-width: 47.9375em)").matches) {
      return "phone";
    }

    return "normal";
  }

  function revealMobileDrawerContext() {
    const viewportMode = resolveViewportMode();

    if (viewportMode !== "phone") {
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

  function bindMobileDrawerReveal() {
    bindViewportReveal();
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
        revealMobileDrawerContext();
      }, 180);
    });
  }

  function bindViewportReveal() {
    if (viewportRevealBound) {
      return;
    }
    viewportRevealBound = true;
    window.addEventListener("bijux:viewport-change", () => {
      revealMobileDrawerContext();
    });
  }

  shell.navReveal = {
    revealActiveNavigationTarget,
    revealAfterLayoutSettles,
    runDesktopNavigationSync,
    runPhoneNavigationSync,
    revealMobileDrawerContext,
    bindMobileDrawerReveal,
    bindViewportReveal,
  };
})();

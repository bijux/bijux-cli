(function () {
  const shell = (window.bijuxShell = window.bijuxShell || {});

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
    revealMobileDrawerContext();
  }

  function revealMobileDrawerContext() {
    const viewportMode =
      window.bijuxViewportProfile &&
      typeof window.bijuxViewportProfile.current === "function"
        ? window.bijuxViewportProfile.current()
        : window.matchMedia("(max-width: 76.2344em)").matches
          ? "phone"
          : "normal";

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

  shell.navReveal = {
    revealActiveNavigationTarget,
    revealAfterLayoutSettles,
    runDesktopNavigationSync,
    runPhoneNavigationSync,
    revealMobileDrawerContext,
    bindMobileDrawerReveal,
  };
})();

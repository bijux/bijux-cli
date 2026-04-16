(function () {
  const PHONE_MAX_MEDIA = "(max-width: 76.2344em)";
  const WIDE_MIN_MEDIA = "(min-width: 120em)";

  function resolveViewportProfile() {
    if (window.matchMedia(PHONE_MAX_MEDIA).matches) {
      return "phone";
    }

    if (window.matchMedia(WIDE_MIN_MEDIA).matches) {
      return "wide";
    }

    return "normal";
  }

  function applyViewportProfile() {
    const profile = resolveViewportProfile();
    document.documentElement.setAttribute("data-bijux-viewport", profile);
    document.body?.setAttribute("data-bijux-viewport", profile);
    return profile;
  }

  function bindViewportUpdates() {
    if (window.__bijuxViewportBound === true) {
      return;
    }

    window.__bijuxViewportBound = true;

    let rafId = 0;
    const onResize = () => {
      if (rafId !== 0) {
        return;
      }
      rafId = window.requestAnimationFrame(() => {
        rafId = 0;
        applyViewportProfile();
      });
    };

    window.addEventListener("resize", onResize, { passive: true });
    window.addEventListener("orientationchange", onResize, { passive: true });
  }

  function init() {
    applyViewportProfile();
    bindViewportUpdates();
  }

  window.bijuxViewportProfile = {
    current: resolveViewportProfile,
    apply: applyViewportProfile,
  };

  document$.subscribe(init);
})();

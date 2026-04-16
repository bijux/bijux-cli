# Bijux Shell JavaScript Ownership

- `theme-persistence.js`: cross-project theme durability via configured `extra.bijux.theme_key` and palette synchronization.
- `viewport-profile.js`: viewport profile state (`phone|normal|wide`) and resize/orientation updates.
- `nav-state.js`: site-tab and canonical-path active state decisions.
- `detail-tabs.js`: detail-strip visibility, active detail-tab state, and phone section-selector synchronization.
- `nav-reveal.js`: scroll-reveal behavior for active tabs and mobile drawer context.
- `bootstrap.js`: shell navigation wiring on `document$.subscribe(...)`.
- `nav-sync.js`: compatibility entrypoint consumed by MkDocs `extra_javascript`.

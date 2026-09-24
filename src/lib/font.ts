// Keep the legacy setting key readable, while rendering every surface in MiSans.
export function applyAppFont(_fontId?: string) {
  document.documentElement.style.setProperty("--app-font", '"MiSans"');
}

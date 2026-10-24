import { afterAll, beforeAll, describe, expect, it, vi } from "vitest";
import { render } from "svelte/server";
import MediaProgress from "./MediaProgress.svelte";

describe("MediaProgress inline seek feedback", () => {
  beforeAll(() => vi.stubGlobal("cancelAnimationFrame", () => 0));
  afterAll(() => vi.unstubAllGlobals());

  it.each(["IslandSurface", "FloatingWindow"])("renders an accessible alert for %s", () => {
    const { body } = render(MediaProgress, {
      props: { position: 12_000, duration: 180_000, seekable: true, seekError: "[4102] CDP 未确认跳转" },
    });

    expect(body).toContain('role="alert"');
    expect(body).toContain("[4102] CDP 未确认跳转");
    expect(body).toContain('type="range"');
  });

  it("omits the alert after the caller clears a previous seek error", () => {
    const { body } = render(MediaProgress, {
      props: { position: 12_000, duration: 180_000, seekable: true, seekError: "" },
    });

    expect(body).not.toContain('role="alert"');
  });

  it("shows unknown time instead of formatting a zero-duration timeline as valid", () => {
    const { body } = render(MediaProgress, {
      props: { position: 0, duration: 0, progressKnown: false, seekable: true },
    });

    expect(body.match(/--:--/g)).toHaveLength(2);
    expect(body).not.toContain("0:00");
    expect(body).toContain('role="status"');
    expect(body).not.toContain('type="range"');
  });

});

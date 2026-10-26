import { loadImage } from "./imageLoad";

export const SMTC_COVER_NETWORK_SKIP_EDGE = 500;

/** Check an SMTC data URI without touching remote artwork sources. */
export async function isHighResolutionSmtcCover(src: string, signal?: AbortSignal): Promise<boolean> {
  if (!src.toLowerCase().startsWith("data:image/")) return false;

  let image: HTMLImageElement | undefined;
  try {
    image = await loadImage(src, signal);
    return image.naturalWidth >= SMTC_COVER_NETWORK_SKIP_EDGE
      && image.naturalHeight >= SMTC_COVER_NETWORK_SKIP_EDGE;
  } catch (error) {
    if (signal?.aborted) throw error;
    return false;
  } finally {
    image?.removeAttribute("src");
  }
}

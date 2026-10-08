/** An absent field is a delta omission; an empty/null field explicitly clears. */
export function artworkValue(payload: Record<string, unknown>): string | undefined {
  for (const key of ["albumArt", "thumbnail", "coverUrl", "api_cover_url", "image"]) {
    if (!Object.hasOwn(payload, key)) continue;
    const value = payload[key];
    if (value === null) return "";
    if (typeof value === "string") return value;
  }
  return undefined;
}

export type ArtworkSnapshotToken = { events: number; artwork: number };
export function snapshotDecision(start: ArtworkSnapshotToken, current: ArtworkSnapshotToken, sameTrack: boolean) {
  if (start.events !== current.events && !sameTrack) return "discard";
  if (start.artwork !== current.artwork && sameTrack) return "ignore-artwork";
  return "accept";
}

export function withoutArtwork<T extends Record<string, unknown>>(payload: T) {
  const copy = { ...payload };
  for (const key of ["albumArt", "thumbnail", "coverUrl", "api_cover_url", "image"]) delete copy[key];
  return copy;
}

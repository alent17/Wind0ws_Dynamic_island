export interface CanvasArtworkRequest {
  url: string;
  track: string;
  pixelated: boolean;
}

interface ArtworkDependencies {
  process: (url: string) => Promise<string>;
  paint: (canvas: HTMLCanvasElement, image: HTMLImageElement) => void;
  createImage?: () => HTMLImageElement;
  onError?: (error: unknown) => void;
}

/** One DOM canvas owns one render generation and at most one loading Image. */
export function createCanvasArtworkRenderer(canvas: HTMLCanvasElement, dependencies: ArtworkDependencies) {
  let target: HTMLCanvasElement | null = canvas;
  let epoch = 0;
  let disposed = false;
  let cancelImage: (() => void) | undefined;

  function releaseImage() {
    cancelImage?.();
    cancelImage = undefined;
  }

  function load(url: string, generation: number, fallback?: string) {
    if (disposed || epoch !== generation || !target?.isConnected) return;
    const image = dependencies.createImage?.() ?? new Image();
    if (url.startsWith("http") && !url.includes("asset.localhost")) image.crossOrigin = "Anonymous";
    const cleanup = () => {
      image.onload = null;
      image.onerror = null;
      image.src = "";
    };
    cancelImage = cleanup;
    image.onload = () => {
      try {
        if (!disposed && epoch === generation && target?.isConnected) dependencies.paint(target, image);
      } finally {
        cleanup();
        if (cancelImage === cleanup) cancelImage = undefined;
      }
    };
    image.onerror = () => {
      cleanup();
      if (cancelImage === cleanup) cancelImage = undefined;
      if (disposed || epoch !== generation || !target?.isConnected) return;
      if (fallback && fallback !== url) load(fallback, generation);
      else dependencies.onError?.(new Error("Canvas artwork failed to load"));
    };
    image.src = url;
  }

  return {
    update(request: CanvasArtworkRequest) {
      if (disposed || !target) return;
      const generation = ++epoch;
      releaseImage();
      if (!request.url) {
        target.width = target.height = 0;
        return;
      }
      // Keep the current pixels until the next image is ready. Late backend
      // results and late image events can never repaint a newer generation.
      if (!request.pixelated) {
        load(request.url, generation);
        return;
      }
      void dependencies.process(request.url).then(
        url => load(url, generation, request.url),
        () => load(request.url, generation),
      );
    },
    destroy() {
      disposed = true;
      epoch++;
      releaseImage();
      // Release the backing store even if another reference retains the node.
      if (target) target.width = target.height = 0;
      // A backend promise can outlive its DOM node; its handlers retain this
      // empty owner, rather than a detached canvas until processing completes.
      target = null;
    },
  };
}

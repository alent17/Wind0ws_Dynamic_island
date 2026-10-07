<script lang="ts">
  import { createCanvasArtworkRenderer, type CanvasArtworkRequest } from "../src/lib/canvasArtwork";
  let request = $state<CanvasArtworkRequest | null>(null);
  const held: (() => void)[] = [];
  (window as any).configureArtwork = (value: CanvasArtworkRequest | null) => request = value;
  (window as any).resolveArtwork = () => held.splice(0).forEach(resolve => resolve());
  (window as any).releasedCanvases = [];
  function artwork(canvas: HTMLCanvasElement, input: CanvasArtworkRequest) {
    const renderer = createCanvasArtworkRenderer(canvas, {
      process: url => new Promise(resolve => held.push(() => resolve(url))),
      paint: (target, image) => {
        target.width = image.naturalWidth;
        target.height = image.naturalHeight;
        target.getContext("2d")!.drawImage(image, 0, 0);
      },
    });
    renderer.update(input);
    return {
      update: renderer.update,
      destroy() {
        renderer.destroy();
        (window as any).releasedCanvases.push({ width: canvas.width, height: canvas.height });
      },
    };
  }
</script>
{#if request}<canvas use:artwork={request}></canvas>{/if}

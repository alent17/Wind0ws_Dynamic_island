// Keep the native renderer on the same Lucide paths as the Svelte UI.
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { resolve } from 'node:path';

const names = ['timer', 'volume-2', 'gallery-horizontal-end', 'settings', 'eye-off',
  'clock', 'cloud-sun', 'arrow-left', 'x', 'check', 'play', 'pause', 'chevron-down',
  'sun', 'cloud', 'cloud-fog', 'cloud-rain', 'cloud-snow', 'cloud-lightning', 'chevron-up'];
const root = resolve(import.meta.dirname, '..');
const out = resolve(root, 'native/assets/icons');
mkdirSync(out, { recursive: true });
for (const name of names) {
  const source = readFileSync(resolve(root, `node_modules/lucide-svelte/dist/icons/${name}.svelte`), 'utf8');
  const nodes = JSON.parse(source.match(/const iconNode = (.*);/)[1]);
  const paths = nodes.map(([tag, attrs]) => `<${tag} ${Object.entries(attrs).map(([key, value]) => `${key}="${String(value).replaceAll('&', '&amp;').replaceAll('"', '&quot;')}"`).join(' ')}/>`).join('');
  const fill = ['play','pause'].includes(name) ? 'currentColor' : 'none';
  writeFileSync(resolve(out, `${name}.svg`), `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="${fill}" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">${paths}</svg>\n`);
}
writeFileSync(resolve(out, 'LICENSE'), readFileSync(resolve(root, 'node_modules/lucide-svelte/LICENSE')));
console.log(`Synced ${names.length} Lucide icons for Direct2D.`);

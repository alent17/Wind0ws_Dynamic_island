import { build, loadConfigFromFile } from 'vite';
const label=process.argv[2]||'before';
if (!/^[a-zA-Z0-9_-]+$/.test(label)) throw new Error('Invalid performance label');
// Replace the app's multi-entry inputs instead of merging and rebuilding the
// main and Studio bundles into a supposedly isolated benchmark fixture.
const { config } = await loadConfigFromFile({command:'build',mode:'production'});
await build({...config,configFile:false,build:{...config.build,outDir:`dist/performance/${label}`,rollupOptions:{input:{fixture:'ui-tests/island-fixture.html'}}}});

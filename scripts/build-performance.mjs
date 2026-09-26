import { build } from 'vite';
const label=process.argv[2]||'before';
await build({build:{outDir:`dist/performance/${label}`,rollupOptions:{input:{fixture:'ui-tests/island-fixture.html'}}}});

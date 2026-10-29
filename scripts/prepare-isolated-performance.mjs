import { cp, mkdir, readFile, writeFile } from "node:fs/promises";
import { resolve, basename } from "node:path";

// This creates a separate, disposable checkout. It never changes the real
// application's settings, registry values, source files, or release binary.
const suffix = process.argv[2] ?? "";
if (suffix && !/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(suffix)) throw new Error("Invalid snapshot name");
const destination = resolve(`dist/performance/isolated-build${suffix ? `-${suffix}` : ""}`);
await mkdir(destination); // Refuse to overwrite an existing snapshot.
await cp("src-tauri", resolve(destination, "src-tauri"), {
  recursive: true,
  filter: path => !["target", "target-idle-fix", "gen"].includes(basename(path)),
});
await cp("build", resolve(destination, "build"), { recursive: true });
const configPath = resolve(destination, "src-tauri/tauri.conf.json");
const config = JSON.parse(await readFile(configPath, "utf8"));
config.identifier = `com.isle-app.performance-test${suffix ? `-${suffix}` : ""}`;
await writeFile(configPath, JSON.stringify(config, null, 2) + "\n");
const entryPath = resolve(destination, "src-tauri/src/lib.rs");
let entry = (await readFile(entryPath, "utf8")).replace(/\r\n/g, "\n");
const startup = `            if let Err(error) = set_auto_start(initial_settings.auto_start) {
                tracing::warn!("[Setup] 同步开机启动失败: {}", error);
            }`;
const maintenance = "            commands::start_netease_auto_restore(app.handle().clone());";
if (!entry.includes(startup) || !entry.includes(maintenance)) {
  throw new Error("Startup hooks changed; review the isolated build before running it.");
}
entry = entry.replace(startup, "            // Isolated benchmark: never modify the user's Run registry entries.");
entry = entry.replace(maintenance, "            // Isolated benchmark: never repair the user's player launch entries.");
await writeFile(entryPath, entry);
console.log(`Prepared ${destination}. Build this snapshot with the custom-protocol feature; verify its actual data path before starting it.`);

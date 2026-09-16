import sharp from "sharp";
import { mkdir } from "node:fs/promises";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const output = resolve(root, "src-tauri", "windows");
const icon = resolve(root, "src-tauri", "app-icon.png");
await mkdir(output, { recursive: true });

const headerBackground = Buffer.from(`
  <svg width="150" height="57" xmlns="http://www.w3.org/2000/svg">
    <rect width="150" height="57" fill="#0f172a"/>
    <circle cx="14" cy="12" r="25" fill="#55c6df" opacity=".13"/>
    <path d="M0 55 C38 38 78 69 150 31 V57 H0Z" fill="#1e293b"/>
    <path d="M83 42 C102 29 124 28 150 16" fill="none" stroke="#55c6df" stroke-width="1" opacity=".7"/>
  </svg>`);
await sharp(headerBackground)
  .composite([{ input: await sharp(icon).resize(40, 40).png().toBuffer(), left: 101, top: 8 }])
  .png()
  .toFile(resolve(output, "installer-header.png"));

const sidebarBackground = Buffer.from(`
  <svg width="164" height="314" xmlns="http://www.w3.org/2000/svg">
    <defs><linearGradient id="g" x1="0" y1="0" x2="0" y2="1"><stop stop-color="#111a30"/><stop offset="1" stop-color="#070b14"/></linearGradient></defs>
    <rect width="164" height="314" fill="url(#g)"/>
    <circle cx="82" cy="103" r="70" fill="#55c6df" opacity=".06"/>
    <circle cx="82" cy="103" r="52" fill="none" stroke="#55c6df" stroke-width="1" stroke-dasharray="2 7" opacity=".42"/>
    <text x="82" y="224" text-anchor="middle" fill="#fff" font-family="Segoe UI, sans-serif" font-size="25" font-weight="650" letter-spacing="6">ISLE</text>
    <text x="82" y="246" text-anchor="middle" fill="#8fa2bd" font-family="Segoe UI, sans-serif" font-size="8" letter-spacing="2">MUSIC AT A GLANCE</text>
    <rect x="53" y="273" width="58" height="3" rx="1.5" fill="#55c6df" opacity=".8"/>
  </svg>`);
await sharp(sidebarBackground)
  .composite([{ input: await sharp(icon).resize(82, 82).png().toBuffer(), left: 41, top: 62 }])
  .png()
  .toFile(resolve(output, "installer-sidebar.png"));

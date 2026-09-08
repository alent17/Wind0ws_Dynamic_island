import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import pngToIco from 'png-to-ico';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.join(__dirname, '..');
const iconsDir = path.join(projectRoot, 'src-tauri', 'icons');

console.log('开始生成 ICO 图标...');

async function generateIco() {
  try {
    const pngPath = path.join(iconsDir, '256x256.png');
    const icoPath = path.join(iconsDir, 'icon.ico');
    
    const pngBuffer = fs.readFileSync(pngPath);
    const icoBuffer = await pngToIco(pngBuffer);
    
    fs.writeFileSync(icoPath, icoBuffer);
    
    console.log('✅ 成功生成 icon.ico');
    console.log('\n🎉 ICO 图标生成完成！');
    
  } catch (error) {
    console.error('❌ 生成 ICO 图标失败:', error);
    process.exit(1);
  }
}

generateIco();

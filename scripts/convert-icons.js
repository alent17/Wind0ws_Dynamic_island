import sharp from 'sharp';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.join(__dirname, '..');

const svgPath = path.join(projectRoot, 'src', 'assets', 'icons', 'tray-icon.svg');
const iconsDir = path.join(projectRoot, 'src-tauri', 'icons');

console.log('开始转换图标...');
console.log('SVG 源文件:', svgPath);
console.log('输出目录:', iconsDir);

const sizes = [32, 128, 256, 512];

async function convertSvgToPng() {
  try {
    const svgBuffer = fs.readFileSync(svgPath);
    
    for (const size of sizes) {
      const outputPath = path.join(iconsDir, `${size}x${size}.png`);
      
      await sharp(svgBuffer)
        .resize(size, size)
        .png()
        .toFile(outputPath);
      
      console.log(`✅ 生成 ${size}x${size}.png`);
    }
    
    // 生成 128x128@2x.png (256x256)
    await sharp(svgBuffer)
      .resize(256, 256)
      .png()
      .toFile(path.join(iconsDir, '128x128@2x.png'));
    console.log('✅ 生成 128x128@2x.png');
    
    // 生成 icon.png (512x512)
    await sharp(svgBuffer)
      .resize(512, 512)
      .png()
      .toFile(path.join(iconsDir, 'icon.png'));
    console.log('✅ 生成 icon.png');
    
    // 生成 Windows Store 图标尺寸
    const storeSizes = [30, 44, 71, 89, 107, 142, 150, 284, 310];
    for (const size of storeSizes) {
      const outputPath = path.join(iconsDir, `Square${size}x${size}Logo.png`);
      
      await sharp(svgBuffer)
        .resize(size, size)
        .png()
        .toFile(outputPath);
      
      console.log(`✅ 生成 Square${size}x${size}Logo.png`);
    }
    
    // 生成 StoreLogo.png (50x50)
    await sharp(svgBuffer)
      .resize(50, 50)
      .png()
      .toFile(path.join(iconsDir, 'StoreLogo.png'));
    console.log('✅ 生成 StoreLogo.png');
    
    console.log('\n🎉 所有图标转换完成！');
    
  } catch (error) {
    console.error('❌ 转换失败:', error);
    process.exit(1);
  }
}

convertSvgToPng();

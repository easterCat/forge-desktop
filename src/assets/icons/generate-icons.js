#!/usr/bin/env node
/* eslint-disable @typescript-eslint/no-require-imports */
/**
 * Node.js Icon Generator for Forge
 * Uses sharp library to convert SVG to PNG
 *
 * Usage: node generate-icons.js
 * Requires: npm install sharp
 */

const fs = require('fs');
const path = require('path');
const sharp = require('sharp');

const DESIGN_DIR = path.join(__dirname, '..', 'src-tauri', 'icons');
const SOURCE_SVG = path.join(__dirname, 'app', 'icon-app-base.svg');

async function generateIcons() {
  console.log('🎨 Forge Icon Generator (Node.js)\n');

  // Check source
  if (!fs.existsSync(SOURCE_SVG)) {
    console.error(`❌ Source SVG not found: ${SOURCE_SVG}`);
    process.exit(1);
  }

  // Read SVG
  const svgBuffer = fs.readFileSync(SOURCE_SVG);

  // Define sizes needed for Tauri
  const sizes = [
    { name: '32x32.png', size: 32 },
    { name: '128x128.png', size: 128 },
    { name: '128x128@2x.png', size: 256 },
  ];

  console.log('📱 Generating app icons...\n');

  for (const { name, size } of sizes) {
    const outputPath = path.join(DESIGN_DIR, name);
    try {
      await sharp(svgBuffer)
        .resize(size, size)
        .png()
        .toFile(outputPath);
      console.log(`  ✓ ${name} (${size}x${size})`);
    } catch (err) {
      console.error(`  ❌ Failed to generate ${name}:`, err.message);
    }
  }

  console.log('\n✅ Icon generation complete!\n');
}

generateIcons().catch(console.error);

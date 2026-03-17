/**
 * Generates minimal placeholder ICO/PNG for Tauri Windows build.
 * Run: node scripts/gen-icons.cjs
 * Replace with real icons later: npm run tauri icon path/to/512.png
 */
const fs = require("fs");
const path = require("path");

const iconsDir = path.join(__dirname, "..", "src-tauri", "icons");
if (!fs.existsSync(iconsDir)) fs.mkdirSync(iconsDir, { recursive: true });

function createMinimalIco() {
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(1, 4);

  const bmpHeaderSize = 40;
  const colorRow = 64;
  const andRow = 4;
  const imageSize = bmpHeaderSize + colorRow * 16 + andRow * 16;
  const dirEntry = Buffer.alloc(16);
  dirEntry[0] = 16;
  dirEntry[1] = 16;
  dirEntry[2] = 0;
  dirEntry[3] = 0;
  dirEntry.writeUInt16LE(1, 4);
  dirEntry.writeUInt16LE(32, 6);
  dirEntry.writeUInt32LE(imageSize, 8);
  dirEntry.writeUInt32LE(22, 12);

  const bmp = Buffer.alloc(imageSize, 0);
  bmp.writeUInt32LE(40, 0);
  bmp.writeInt32LE(16, 4);
  bmp.writeInt32LE(32, 8);
  bmp.writeUInt16LE(1, 12);
  bmp.writeUInt16LE(32, 14);
  bmp.writeUInt32LE(0, 16);
  bmp.writeUInt32LE(colorRow * 16 + andRow * 16, 20);
  let off = 40;
  for (let y = 15; y >= 0; y--) {
    for (let x = 0; x < 16; x++) {
      bmp[off++] = 0x0a;
      bmp[off++] = 0x2e;
      bmp[off++] = 0x5a;
      bmp[off++] = 0xff;
    }
    off += colorRow - 64;
  }

  return Buffer.concat([header, dirEntry, bmp]);
}

const icoPath = path.join(iconsDir, "icon.ico");
fs.writeFileSync(icoPath, createMinimalIco());
console.log("Wrote", icoPath);

const png32 = Buffer.from(
  "iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAYAAABzenr0AAAAEklEQVR4Ae3BAQEAAACCIP+vbkhAAQAAAOD/ARqgAAH2b3Z4AAAAAElFTkSuQmCC",
  "base64"
);
fs.writeFileSync(path.join(iconsDir, "32x32.png"), png32);
fs.writeFileSync(path.join(iconsDir, "128x128.png"), png32);
fs.writeFileSync(path.join(iconsDir, "128x128@2x.png"), png32);
console.log("Wrote 32x32.png, 128x128.png, 128x128@2x.png");

const icnsPath = path.join(iconsDir, "icon.icns");
if (!fs.existsSync(icnsPath)) {
  fs.writeFileSync(icnsPath, Buffer.alloc(0));
  console.log("Wrote empty icon.icns");
}

console.log("Done. Run: npm run tauri:dev");

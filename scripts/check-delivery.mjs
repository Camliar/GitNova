import { existsSync, readFileSync } from "node:fs";
import { inflateSync } from "node:zlib";

const read = (path) => readFileSync(path, "utf8");
const config = JSON.parse(read("apps/desktop/src-tauri/tauri.conf.json"));
const bundleConfig = JSON.parse(read("apps/desktop/src-tauri/tauri.bundle.conf.json"));
const ci = read(".github/workflows/ci.yml");
const release = read(".github/workflows/release.yml");
const appIconPath = "apps/desktop/src-tauri/icons/icon.png";
const macIconPath = "apps/desktop/src-tauri/icons/icon.icns";
const windowsIconPath = "apps/desktop/src-tauri/icons/icon.ico";
const appIcon = existsSync(appIconPath) ? readFileSync(appIconPath) : Buffer.alloc(0);
const macIcon = existsSync(macIconPath) ? readFileSync(macIconPath) : Buffer.alloc(0);
const windowsIcon = existsSync(windowsIconPath) ? readFileSync(windowsIconPath) : Buffer.alloc(0);

const paeth = (left, above, upperLeft) => {
  const estimate = left + above - upperLeft;
  const leftDistance = Math.abs(estimate - left);
  const aboveDistance = Math.abs(estimate - above);
  const upperLeftDistance = Math.abs(estimate - upperLeft);
  return leftDistance <= aboveDistance && leftDistance <= upperLeftDistance
    ? left
    : aboveDistance <= upperLeftDistance
      ? above
      : upperLeft;
};

const readPng = (png) => {
  if (png.length < 33 || !png.subarray(0, 8).equals(Buffer.from("89504e470d0a1a0a", "hex"))) return null;
  let offset = 8;
  let width;
  let height;
  const imageData = [];
  while (offset + 12 <= png.length) {
    const length = png.readUInt32BE(offset);
    const type = png.toString("ascii", offset + 4, offset + 8);
    const data = png.subarray(offset + 8, offset + 8 + length);
    if (type === "IHDR") {
      width = data.readUInt32BE(0);
      height = data.readUInt32BE(4);
      if (data[8] !== 8 || data[9] !== 6 || data[12] !== 0) return null;
    } else if (type === "IDAT") {
      imageData.push(data);
    }
    offset += length + 12;
  }
  if (!width || !height || !imageData.length) return null;
  const encoded = inflateSync(Buffer.concat(imageData));
  const stride = width * 4;
  const pixels = Buffer.alloc(stride * height);
  for (let row = 0; row < height; row += 1) {
    const filter = encoded[row * (stride + 1)];
    for (let column = 0; column < stride; column += 1) {
      const raw = encoded[row * (stride + 1) + column + 1];
      const left = column >= 4 ? pixels[row * stride + column - 4] : 0;
      const above = row ? pixels[(row - 1) * stride + column] : 0;
      const upperLeft = row && column >= 4 ? pixels[(row - 1) * stride + column - 4] : 0;
      const predictor = [0, left, above, Math.floor((left + above) / 2), paeth(left, above, upperLeft)][filter];
      if (predictor === undefined) return null;
      pixels[row * stride + column] = (raw + predictor) & 0xff;
    }
  }
  return { width, height, pixels, stride };
};

const failures = [];
if (config.bundle?.active !== true) failures.push("Tauri bundle must be active");
if (!bundleConfig.bundle?.externalBin?.includes("bin/gitnova-core")) failures.push("Desktop bundle must include the Core sidecar");
if (!config.bundle?.icon?.includes("icons/icon.icns")) failures.push("Tauri bundle must explicitly include the macOS icon");
if (!config.bundle?.icon?.includes("icons/icon.ico")) failures.push("Tauri bundle must explicitly include the Windows icon");
if (macIcon.subarray(0, 4).toString("ascii") !== "icns") failures.push(`macOS icon must be a valid ICNS file: ${macIconPath}`);
let decodedAppIcon;
try {
  decodedAppIcon = readPng(appIcon);
} catch {
  decodedAppIcon = null;
}
if (!decodedAppIcon || decodedAppIcon.width !== 512 || decodedAppIcon.height !== 512) {
  failures.push(`Desktop source icon must be a 512x512 RGBA PNG: ${appIconPath}`);
} else {
  const bottomCenterAlpha = decodedAppIcon.pixels[(decodedAppIcon.height - 12) * decodedAppIcon.stride + Math.floor(decodedAppIcon.width / 2) * 4 + 3];
  if (bottomCenterAlpha === 0) failures.push("Desktop source icon must fill the vertical canvas without a transparent bottom strip");
}
if (!windowsIcon.length) failures.push(`Windows icon is missing: ${windowsIconPath}`);
const iconCount = windowsIcon.length >= 6 ? windowsIcon.readUInt16LE(4) : 0;
const iconSizes = new Set(
  Array.from({ length: iconCount }, (_, index) => windowsIcon[6 + index * 16] || 256),
);
if (
  windowsIcon.length < 6 ||
  windowsIcon.readUInt16LE(0) !== 0 ||
  windowsIcon.readUInt16LE(2) !== 1 ||
  ![16, 32, 48, 256].every((size) => iconSizes.has(size))
) {
  failures.push("Windows icon must be a valid multi-resolution ICO containing 16, 32, 48 and 256 px entries");
}
for (const runner of ["ubuntu-22.04", "windows-latest", "macos-latest"]) {
  if (!ci.includes(runner)) failures.push(`CI runner missing: ${runner}`);
  if (!release.includes(runner)) failures.push(`release runner missing: ${runner}`);
}
if (!ci.includes("contents: read") || /secrets\.[A-Z_]+/.test(ci)) failures.push("CI must be read-only and use no repository secrets");
if (!release.includes("contents: write") || !release.includes("tags:") || !release.includes("- 'v*'")) failures.push("release must be tag-gated with contents write permission");
if (failures.length) {
  console.error(failures.join("\n"));
  process.exit(1);
}
console.log("Delivery configuration is valid");

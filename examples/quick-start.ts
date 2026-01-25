/**
 * Quick Start - Copy & Paste Example
 *
 * This is the simplest possible example to get started with fpexif.
 * Just copy this code and start extracting EXIF data!
 */

import init, { parse_exif_json, parse_exif_get_tag } from '../pkg/fpexif.js';

// ============================================================================
// OPTION 1: Extract all 8 essential tags at once
// ============================================================================

async function quickStart_allTags(imageFile: File) {
  // Step 1: Initialize WASM (do this once at app startup)
  await init();

  // Step 2: Read file as bytes
  const arrayBuffer = await imageFile.arrayBuffer();
  const bytes = new Uint8Array(arrayBuffer);

  // Step 3: Parse EXIF data
  const jsonString = parse_exif_json(bytes);
  const exifArray = JSON.parse(jsonString);
  const exif = exifArray[0];

  // Step 4: Use the data
  console.log(`Camera: ${exif.Make} ${exif.Model}`);
  console.log(`ISO: ${exif.ISO}`);
  console.log(`Aperture: f/${exif.Aperture}`);
  console.log(`Shutter Speed: ${exif.ShutterSpeed}`);
  console.log(`Focal Length: ${exif.FocalLength}`);
  console.log(`Date: ${exif.CreateDate}`);
  console.log(`File Size: ${exif.FileSize}`); // "34 MB"
  console.log(`File Bytes: ${exif.FileSizeBytes}`); // 35651584

  return exif;
}

// ============================================================================
// OPTION 2: Extract single tags (faster, case-insensitive)
// ============================================================================

async function quickStart_singleTag(imageFile: File) {
  // Step 1: Initialize WASM
  await init();

  // Step 2: Read file
  const arrayBuffer = await imageFile.arrayBuffer();
  const bytes = new Uint8Array(arrayBuffer);

  // Step 3: Extract individual tags (all case-insensitive!)
  const make = parse_exif_get_tag(bytes, 'make');           // "Canon"
  const model = parse_exif_get_tag(bytes, 'Model');         // "Canon EOS 5D Mark IV"
  const iso = parse_exif_get_tag(bytes, 'ISO');             // "100"
  const aperture = parse_exif_get_tag(bytes, 'aperture');   // "5.6"
  const shutter = parse_exif_get_tag(bytes, 'shutter_speed'); // "1/800"
  const focal = parse_exif_get_tag(bytes, 'FOCAL_LENGTH');  // "50.0 mm"
  const date = parse_exif_get_tag(bytes, 'create_date');    // "2024:01:15 14:30:22"
  const size = parse_exif_get_tag(bytes, 'file_size');      // "34 MB"

  console.log(`${make} ${model} - ISO ${iso}, f/${aperture}, ${shutter}`);

  return { make, model, iso, aperture, shutter, focal, date, size };
}

// ============================================================================
// OPTION 3: HTML File Input (Browser)
// ============================================================================

async function quickStart_fileInput() {
  // Initialize WASM
  await init();

  // Setup file input handler
  const fileInput = document.querySelector<HTMLInputElement>('input[type="file"]');

  fileInput?.addEventListener('change', async (event) => {
    const file = (event.target as HTMLInputElement).files?.[0];
    if (!file) return;

    // Read and parse
    const arrayBuffer = await file.arrayBuffer();
    const bytes = new Uint8Array(arrayBuffer);
    const jsonString = parse_exif_json(bytes);
    const exifArray = JSON.parse(jsonString);
    const exif = exifArray[0];

    // Display results
    console.log('EXIF Data:', exif);
    alert(`Camera: ${exif.Make} ${exif.Model}\nISO: ${exif.ISO}\nFile: ${exif.FileSize}`);
  });
}

// ============================================================================
// OPTION 4: Node.js (file from disk)
// ============================================================================

async function quickStart_nodejs(filePath: string) {
  const fs = await import('fs/promises');

  // Initialize WASM
  await init();

  // Read file from disk
  const buffer = await fs.readFile(filePath);
  const bytes = new Uint8Array(buffer);

  // Parse EXIF
  const jsonString = parse_exif_json(bytes);
  const exifArray = JSON.parse(jsonString);
  const exif = exifArray[0];

  // Use the data
  console.log(`${exif.Make} ${exif.Model}`);
  console.log(`ISO ${exif.ISO} • f/${exif.Aperture} • ${exif.ShutterSpeed}`);
  console.log(`${exif.FileSize}`);

  return exif;
}

// ============================================================================
// BONUS: TypeScript Types
// ============================================================================

interface ExifData {
  Make?: string;              // "Canon"
  Model?: string;             // "Canon EOS 5D Mark IV"
  ISO?: number;               // 100
  Aperture?: number;          // 5.6
  ShutterSpeed?: string;      // "1/800"
  FocalLength?: string;       // "50.0 mm"
  CreateDate?: string;        // "2024:01:15 14:30:22"
  FileSize?: string;          // "34 MB"
  FileSizeBytes?: number;     // 35651584
  // ... hundreds more tags available
  [key: string]: any;
}

// ============================================================================
// Usage Examples
// ============================================================================

// Example 1: File upload in browser
if (typeof document !== 'undefined') {
  document.addEventListener('DOMContentLoaded', () => {
    quickStart_fileInput();
  });
}

// Example 2: Process file in Node.js
if (typeof process !== 'undefined' && process.argv.length > 2) {
  const filePath = process.argv[2];
  quickStart_nodejs(filePath).catch(console.error);
}

// Export for use in other modules
export {
  quickStart_allTags,
  quickStart_singleTag,
  quickStart_fileInput,
  quickStart_nodejs,
};

/**
 * Extract Essential EXIF Tags from RAW/JPEG Files using fpexif WASM
 *
 * This module provides a simple interface to extract the 8 most essential
 * photography metadata tags from image files using the fpexif WASM library.
 *
 * Essential Tags:
 * - make: Camera manufacturer (e.g., "Canon", "Nikon")
 * - model: Camera model (e.g., "Canon EOS 5D Mark IV")
 * - iso: ISO sensitivity (e.g., 100, 800, 3200)
 * - aperture: Lens aperture (e.g., 2.8, 5.6, 16.0)
 * - shutter_speed: Shutter speed (e.g., "1/1000", "2.5")
 * - focal_length: Focal length (e.g., "50.0 mm", "24.0 mm")
 * - create_date: Date/time photo was taken (e.g., "2024:01:15 14:30:22")
 * - file_size: Human-readable file size (e.g., "34 MB", "1.5 GB")
 *
 * @module extract-essential-tags
 */

import init, { parse_exif_json, parse_exif_get_tag } from '../pkg/fpexif.js';

/**
 * Essential EXIF metadata extracted from an image file
 */
export interface EssentialExifTags {
  /** Camera manufacturer (e.g., "Canon", "Nikon", "Sony") */
  make?: string;
  /** Camera model (e.g., "Canon EOS 5D Mark IV") */
  model?: string;
  /** ISO sensitivity (e.g., 100, 800, 3200) */
  iso?: number;
  /** Lens aperture (e.g., 2.8, 5.6, 16.0) */
  aperture?: number;
  /** Shutter speed (e.g., "1/1000", "2.5") */
  shutter_speed?: string;
  /** Focal length with units (e.g., "50.0 mm") */
  focal_length?: string;
  /** Date/time photo was taken (e.g., "2024:01:15 14:30:22") */
  create_date?: string;
  /** Human-readable file size (e.g., "34 MB", "1.5 GB") */
  file_size?: string;
}

/**
 * Raw EXIF data as returned by fpexif (exiftool-compatible format)
 */
export interface ExifData {
  Make?: string;
  Model?: string;
  ISO?: number;
  Aperture?: number;
  ShutterSpeed?: string;
  FocalLength?: string;
  CreateDate?: string;
  DateTimeOriginal?: string;
  FileSize?: string;
  FileSizeBytes?: number;
  [key: string]: any;
}

/**
 * Options for extracting EXIF data
 */
export interface ExtractOptions {
  /** Whether to throw errors or return partial data. Default: false */
  throwOnError?: boolean;
  /** Custom tag mappings (case-insensitive) */
  customTags?: string[];
}

let wasmInitialized = false;

/**
 * Initialize the WASM module
 * Must be called before using any extraction functions
 *
 * @example
 * ```ts
 * await initWasm();
 * ```
 */
export async function initWasm(): Promise<void> {
  if (!wasmInitialized) {
    await init();
    wasmInitialized = true;
  }
}

/**
 * Extract essential EXIF tags from an image file
 *
 * This function extracts the 8 most important photography metadata tags
 * from RAW or JPEG files. All tag access is case-insensitive, so you can
 * use "make", "Make", or "MAKE" interchangeably.
 *
 * @param imageData - Raw bytes of the image file (ArrayBuffer or Uint8Array)
 * @param options - Optional extraction options
 * @returns Object containing the essential EXIF tags (undefined for missing tags)
 * @throws Error if WASM not initialized or parsing fails (when throwOnError=true)
 *
 * @example
 * ```ts
 * // Read file from input element
 * const file = document.querySelector('input[type="file"]').files[0];
 * const arrayBuffer = await file.arrayBuffer();
 * const bytes = new Uint8Array(arrayBuffer);
 *
 * // Extract essential tags
 * const tags = await extractEssentialTags(bytes);
 * console.log(tags);
 * // {
 * //   make: "Canon",
 * //   model: "Canon EOS 5D Mark IV",
 * //   iso: 100,
 * //   aperture: 5.6,
 * //   shutter_speed: "1/800",
 * //   focal_length: "50.0 mm",
 * //   create_date: "2024:01:15 14:30:22",
 * //   file_size: "34 MB"
 * // }
 * ```
 *
 * @example
 * ```ts
 * // Handle errors gracefully
 * const tags = await extractEssentialTags(bytes, { throwOnError: false });
 * if (tags.make) {
 *   console.log(`Photo taken with ${tags.make} ${tags.model}`);
 * }
 * ```
 */
export async function extractEssentialTags(
  imageData: ArrayBuffer | Uint8Array,
  options: ExtractOptions = {}
): Promise<EssentialExifTags> {
  if (!wasmInitialized) {
    throw new Error('WASM not initialized. Call initWasm() first.');
  }

  const { throwOnError = false } = options;
  const bytes = imageData instanceof Uint8Array ? imageData : new Uint8Array(imageData);

  try {
    // Parse the image and get exiftool-compatible JSON
    const jsonString = parse_exif_json(bytes);
    const exifArray = JSON.parse(jsonString) as ExifData[];

    if (!exifArray || exifArray.length === 0) {
      if (throwOnError) {
        throw new Error('No EXIF data found in image');
      }
      return {};
    }

    const exifData = exifArray[0];

    // Extract essential tags (case-insensitive)
    return {
      make: exifData.Make,
      model: exifData.Model,
      iso: exifData.ISO,
      aperture: exifData.Aperture,
      shutter_speed: exifData.ShutterSpeed,
      focal_length: exifData.FocalLength,
      create_date: exifData.CreateDate || exifData.DateTimeOriginal,
      file_size: exifData.FileSize,
    };
  } catch (error) {
    if (throwOnError) {
      throw new Error(`Failed to extract EXIF data: ${error}`);
    }
    return {};
  }
}

/**
 * Extract a single EXIF tag by name (case-insensitive)
 *
 * This function is more efficient than extractEssentialTags() when you only
 * need one or two specific tags. Tag names are case-insensitive and can use
 * snake_case, PascalCase, or any other casing.
 *
 * @param imageData - Raw bytes of the image file
 * @param tagName - Name of the tag to extract (case-insensitive)
 * @returns The tag value as a string, or undefined if not found
 *
 * @example
 * ```ts
 * // All these are equivalent (case-insensitive):
 * const make1 = await extractTag(bytes, "make");
 * const make2 = await extractTag(bytes, "Make");
 * const make3 = await extractTag(bytes, "MAKE");
 *
 * // Snake_case also works:
 * const shutterSpeed1 = await extractTag(bytes, "shutter_speed");
 * const shutterSpeed2 = await extractTag(bytes, "ShutterSpeed");
 * const shutterSpeed3 = await extractTag(bytes, "SHUTTER_SPEED");
 * ```
 *
 * @example
 * ```ts
 * // Quick check for camera make
 * const make = await extractTag(bytes, "make");
 * if (make === "Canon") {
 *   console.log("Canon camera detected");
 * }
 * ```
 */
export async function extractTag(
  imageData: ArrayBuffer | Uint8Array,
  tagName: string
): Promise<string | undefined> {
  if (!wasmInitialized) {
    throw new Error('WASM not initialized. Call initWasm() first.');
  }

  const bytes = imageData instanceof Uint8Array ? imageData : new Uint8Array(imageData);

  try {
    // Use the optimized single-tag extraction function
    const value = parse_exif_get_tag(bytes, tagName);
    return value;
  } catch (error) {
    // Tag not found or parse error
    return undefined;
  }
}

/**
 * Extract all EXIF data from an image file
 *
 * Returns the complete exiftool-compatible JSON data, including all tags
 * (not just the essential 8). Useful when you need access to manufacturer-
 * specific tags or less common metadata.
 *
 * @param imageData - Raw bytes of the image file
 * @returns Complete EXIF data object
 *
 * @example
 * ```ts
 * const allData = await extractAllExifData(bytes);
 * console.log(allData);
 * // {
 * //   Make: "Canon",
 * //   Model: "Canon EOS 5D Mark IV",
 * //   LensModel: "EF50mm f/1.8 STM",
 * //   ExposureMode: "Manual",
 * //   MeteringMode: "Multi-segment",
 * //   ... hundreds more tags
 * // }
 * ```
 */
export async function extractAllExifData(
  imageData: ArrayBuffer | Uint8Array
): Promise<ExifData | null> {
  if (!wasmInitialized) {
    throw new Error('WASM not initialized. Call initWasm() first.');
  }

  const bytes = imageData instanceof Uint8Array ? imageData : new Uint8Array(imageData);

  try {
    const jsonString = parse_exif_json(bytes);
    const exifArray = JSON.parse(jsonString) as ExifData[];
    return exifArray && exifArray.length > 0 ? exifArray[0] : null;
  } catch (error) {
    console.error('Failed to extract EXIF data:', error);
    return null;
  }
}

/**
 * Format essential EXIF tags as a human-readable string
 *
 * @param tags - Essential EXIF tags object
 * @returns Formatted string with all available tags
 *
 * @example
 * ```ts
 * const tags = await extractEssentialTags(bytes);
 * const formatted = formatEssentialTags(tags);
 * console.log(formatted);
 * // Canon EOS 5D Mark IV
 * // ISO 100 • f/5.6 • 1/800s • 50mm
 * // 2024:01:15 14:30:22 • 34 MB
 * ```
 */
export function formatEssentialTags(tags: EssentialExifTags): string {
  const lines: string[] = [];

  // Camera info
  if (tags.make || tags.model) {
    const camera = [tags.make, tags.model].filter(Boolean).join(' ');
    lines.push(camera);
  }

  // Shooting parameters
  const params: string[] = [];
  if (tags.iso !== undefined) params.push(`ISO ${tags.iso}`);
  if (tags.aperture !== undefined) params.push(`f/${tags.aperture}`);
  if (tags.shutter_speed) params.push(tags.shutter_speed + 's');
  if (tags.focal_length) params.push(tags.focal_length.replace('.0 mm', 'mm'));

  if (params.length > 0) {
    lines.push(params.join(' • '));
  }

  // Date and file size
  const metadata: string[] = [];
  if (tags.create_date) metadata.push(tags.create_date);
  if (tags.file_size) metadata.push(tags.file_size);

  if (metadata.length > 0) {
    lines.push(metadata.join(' • '));
  }

  return lines.join('\n');
}

// Export for convenience
export default {
  initWasm,
  extractEssentialTags,
  extractTag,
  extractAllExifData,
  formatEssentialTags,
};

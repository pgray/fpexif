/**
 * Usage Examples for fpexif WASM Library
 *
 * This file contains comprehensive examples showing how to use the
 * extract-essential-tags module in various contexts.
 */

import {
  initWasm,
  extractEssentialTags,
  extractTag,
  extractAllExifData,
  formatEssentialTags,
  type EssentialExifTags,
} from './extract-essential-tags.js';

// ============================================================================
// Example 1: Basic Usage - Extract Essential Tags from File Input
// ============================================================================

/**
 * Extract EXIF data from a file input element
 * Use this in a browser context with an HTML file input
 */
export async function example1_basicFileInput() {
  // Initialize WASM (do this once at app startup)
  await initWasm();

  // Get file from input element
  const fileInput = document.querySelector<HTMLInputElement>('input[type="file"]');
  if (!fileInput?.files?.[0]) {
    console.log('No file selected');
    return;
  }

  const file = fileInput.files[0];

  // Read file as ArrayBuffer
  const arrayBuffer = await file.arrayBuffer();
  const bytes = new Uint8Array(arrayBuffer);

  // Extract essential tags
  const tags = await extractEssentialTags(bytes);

  console.log('Essential EXIF Tags:');
  console.log(`Camera: ${tags.make} ${tags.model}`);
  console.log(`ISO: ${tags.iso}`);
  console.log(`Aperture: f/${tags.aperture}`);
  console.log(`Shutter Speed: ${tags.shutter_speed}`);
  console.log(`Focal Length: ${tags.focal_length}`);
  console.log(`Date Taken: ${tags.create_date}`);
  console.log(`File Size: ${tags.file_size}`);
}

// ============================================================================
// Example 2: Case-Insensitive Single Tag Extraction
// ============================================================================

/**
 * Extract individual tags using case-insensitive names
 * More efficient when you only need one or two specific tags
 */
export async function example2_singleTagExtraction(imageData: Uint8Array) {
  await initWasm();

  // All these work (case-insensitive):
  const make = await extractTag(imageData, 'make');
  const makeUpper = await extractTag(imageData, 'MAKE');
  const makePascal = await extractTag(imageData, 'Make');

  console.log(make === makeUpper && makeUpper === makePascal); // true

  // Snake_case works too:
  const shutterSpeed1 = await extractTag(imageData, 'shutter_speed');
  const shutterSpeed2 = await extractTag(imageData, 'ShutterSpeed');
  const shutterSpeed3 = await extractTag(imageData, 'SHUTTER_SPEED');

  console.log(shutterSpeed1 === shutterSpeed2 && shutterSpeed2 === shutterSpeed3); // true

  // Quick camera detection
  if (make === 'Canon') {
    console.log('Canon camera detected!');
  }
}

// ============================================================================
// Example 3: React Component - Photo Metadata Display
// ============================================================================

/**
 * React component that displays EXIF metadata from uploaded photos
 */
export function Example3_ReactPhotoMetadata() {
  // This is TypeScript/JSX pseudo-code for a React component

  const [exifData, setExifData] = React.useState<EssentialExifTags | null>(null);
  const [loading, setLoading] = React.useState(false);

  React.useEffect(() => {
    // Initialize WASM on component mount
    initWasm();
  }, []);

  const handleFileChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    setLoading(true);
    try {
      const arrayBuffer = await file.arrayBuffer();
      const bytes = new Uint8Array(arrayBuffer);
      const tags = await extractEssentialTags(bytes);
      setExifData(tags);
    } catch (error) {
      console.error('Failed to extract EXIF data:', error);
    } finally {
      setLoading(false);
    }
  };

  return `
    <div>
      <input type="file" accept="image/*,.cr2,.nef,.arw,.raf,.dng" onChange={handleFileChange} />

      {loading && <p>Extracting EXIF data...</p>}

      {exifData && (
        <div className="exif-metadata">
          <h3>{exifData.make} {exifData.model}</h3>
          <div className="shooting-params">
            <span>ISO {exifData.iso}</span>
            <span>f/{exifData.aperture}</span>
            <span>{exifData.shutter_speed}</span>
            <span>{exifData.focal_length}</span>
          </div>
          <p>{exifData.create_date}</p>
          <p>{exifData.file_size}</p>
        </div>
      )}
    </div>
  `;
}

// ============================================================================
// Example 4: Batch Processing - Multiple Files
// ============================================================================

/**
 * Process multiple image files and extract EXIF data from each
 * Useful for photo gallery applications or batch imports
 */
export async function example4_batchProcessing(files: File[]) {
  await initWasm();

  const results = await Promise.all(
    files.map(async (file) => {
      try {
        const arrayBuffer = await file.arrayBuffer();
        const bytes = new Uint8Array(arrayBuffer);
        const tags = await extractEssentialTags(bytes);

        return {
          filename: file.name,
          success: true,
          tags,
        };
      } catch (error) {
        return {
          filename: file.name,
          success: false,
          error: String(error),
        };
      }
    })
  );

  // Filter by camera make
  const canonPhotos = results.filter((r) => r.success && r.tags?.make === 'Canon');
  const nikonPhotos = results.filter((r) => r.success && r.tags?.make === 'Nikon');

  console.log(`Processed ${results.length} files`);
  console.log(`Canon: ${canonPhotos.length}, Nikon: ${nikonPhotos.length}`);

  return results;
}

// ============================================================================
// Example 5: Photo Organization - Sort by Camera/Settings
// ============================================================================

/**
 * Organize photos by camera model and shooting parameters
 */
export async function example5_photoOrganization(files: File[]) {
  await initWasm();

  interface PhotoWithMetadata {
    file: File;
    tags: EssentialExifTags;
  }

  // Extract metadata from all files
  const photosWithMetadata: PhotoWithMetadata[] = [];

  for (const file of files) {
    const arrayBuffer = await file.arrayBuffer();
    const bytes = new Uint8Array(arrayBuffer);
    const tags = await extractEssentialTags(bytes, { throwOnError: false });

    if (tags.make || tags.model) {
      photosWithMetadata.push({ file, tags });
    }
  }

  // Group by camera model
  const byCamera = photosWithMetadata.reduce(
    (acc, photo) => {
      const cameraKey = `${photo.tags.make} ${photo.tags.model}`.trim();
      if (!acc[cameraKey]) {
        acc[cameraKey] = [];
      }
      acc[cameraKey].push(photo);
      return acc;
    },
    {} as Record<string, PhotoWithMetadata[]>
  );

  // Find all photos shot at specific ISO
  const highISOPhotos = photosWithMetadata.filter((p) => (p.tags.iso ?? 0) >= 1600);

  // Find all photos shot with wide aperture
  const wideAperturePhotos = photosWithMetadata.filter((p) => (p.tags.aperture ?? Infinity) <= 2.8);

  console.log('Photos by camera:', byCamera);
  console.log('High ISO photos:', highISOPhotos.length);
  console.log('Wide aperture photos:', wideAperturePhotos.length);

  return { byCamera, highISOPhotos, wideAperturePhotos };
}

// ============================================================================
// Example 6: Node.js - File System Integration
// ============================================================================

/**
 * Extract EXIF data from files using Node.js fs module
 * Requires Node.js 18+ for native fetch support
 */
export async function example6_nodeJsFileSystem() {
  // This would require Node.js environment
  const fs = await import('fs/promises');
  const path = await import('path');

  await initWasm();

  // Read RAW file from filesystem
  const filePath = '/path/to/photo.CR2';
  const buffer = await fs.readFile(filePath);
  const bytes = new Uint8Array(buffer);

  // Extract tags
  const tags = await extractEssentialTags(bytes);

  console.log(`File: ${path.basename(filePath)}`);
  console.log(formatEssentialTags(tags));

  // Process entire directory
  const directory = '/path/to/photos';
  const files = await fs.readdir(directory);

  for (const filename of files) {
    if (filename.match(/\.(cr2|nef|arw|raf|dng)$/i)) {
      const fullPath = path.join(directory, filename);
      const fileBuffer = await fs.readFile(fullPath);
      const fileBytes = new Uint8Array(fileBuffer);
      const fileTags = await extractEssentialTags(fileBytes);

      console.log(`\n${filename}:`);
      console.log(formatEssentialTags(fileTags));
    }
  }
}

// ============================================================================
// Example 7: Error Handling
// ============================================================================

/**
 * Proper error handling when extracting EXIF data
 */
export async function example7_errorHandling(imageData: Uint8Array) {
  await initWasm();

  // Option 1: Graceful degradation (default)
  const tags = await extractEssentialTags(imageData, { throwOnError: false });

  // Check if tags are available
  if (tags.make) {
    console.log(`Camera: ${tags.make}`);
  } else {
    console.log('Camera make not available');
  }

  // Option 2: Throw errors for debugging
  try {
    const tagsStrict = await extractEssentialTags(imageData, { throwOnError: true });
    console.log('All tags extracted successfully');
  } catch (error) {
    console.error('Failed to extract EXIF data:', error);
  }

  // Option 3: Single tag extraction with error handling
  const make = await extractTag(imageData, 'make');
  if (make === undefined) {
    console.log('Make tag not found');
  } else {
    console.log(`Make: ${make}`);
  }
}

// ============================================================================
// Example 8: Advanced - Custom Tag Extraction
// ============================================================================

/**
 * Extract manufacturer-specific or uncommon tags
 */
export async function example8_advancedTags(imageData: Uint8Array) {
  await initWasm();

  // Get all EXIF data for access to less common tags
  const allData = await extractAllExifData(imageData);

  if (allData) {
    // Access any tag from the complete dataset
    console.log('Lens Model:', allData.LensModel);
    console.log('Exposure Mode:', allData.ExposureMode);
    console.log('Metering Mode:', allData.MeteringMode);
    console.log('White Balance:', allData.WhiteBalance);
    console.log('Flash:', allData.Flash);

    // Manufacturer-specific tags
    console.log('Canon-specific tags:', {
      SerialNumber: allData.SerialNumber,
      InternalSerialNumber: allData.InternalSerialNumber,
      LensSerialNumber: allData.LensSerialNumber,
    });

    // You can access any tag using case-insensitive extraction too:
    const lensModel = await extractTag(imageData, 'lens_model');
    const lensModelPascal = await extractTag(imageData, 'LensModel');
    console.log(lensModel === lensModelPascal); // true
  }
}

// ============================================================================
// Example 9: Performance - Lazy Loading WASM
// ============================================================================

/**
 * Initialize WASM only when needed to improve initial page load
 */
export class LazyExifExtractor {
  private initialized = false;

  async ensureInitialized() {
    if (!this.initialized) {
      await initWasm();
      this.initialized = true;
    }
  }

  async extractTags(imageData: Uint8Array): Promise<EssentialExifTags> {
    await this.ensureInitialized();
    return extractEssentialTags(imageData);
  }

  async extractSingleTag(imageData: Uint8Array, tagName: string): Promise<string | undefined> {
    await this.ensureInitialized();
    return extractTag(imageData, tagName);
  }
}

// Usage:
export async function example9_lazyLoading() {
  const extractor = new LazyExifExtractor();

  // WASM is initialized on first use
  const fileInput = document.querySelector<HTMLInputElement>('input[type="file"]');
  fileInput?.addEventListener('change', async (event) => {
    const file = (event.target as HTMLInputElement).files?.[0];
    if (!file) return;

    const arrayBuffer = await file.arrayBuffer();
    const bytes = new Uint8Array(arrayBuffer);

    // WASM initializes automatically here
    const tags = await extractor.extractTags(bytes);
    console.log(tags);
  });
}

// ============================================================================
// Example 10: Photo Gallery - Display Metadata
// ============================================================================

/**
 * Create a photo gallery with EXIF metadata tooltips/overlays
 */
export async function example10_photoGallery(photoFiles: File[]) {
  await initWasm();

  interface PhotoInfo {
    url: string;
    filename: string;
    exif: EssentialExifTags;
    formattedExif: string;
  }

  const gallery: PhotoInfo[] = [];

  for (const file of photoFiles) {
    // Create preview URL
    const url = URL.createObjectURL(file);

    // Extract EXIF
    const arrayBuffer = await file.arrayBuffer();
    const bytes = new Uint8Array(arrayBuffer);
    const exif = await extractEssentialTags(bytes);
    const formattedExif = formatEssentialTags(exif);

    gallery.push({
      url,
      filename: file.name,
      exif,
      formattedExif,
    });
  }

  // Example: Render gallery with metadata
  gallery.forEach((photo) => {
    console.log(`
      Photo: ${photo.filename}
      ${photo.formattedExif}
      ---
    `);
  });

  // Filter gallery by criteria
  const canonPhotos = gallery.filter((p) => p.exif.make === 'Canon');
  const portraitAperture = gallery.filter((p) => (p.exif.aperture ?? Infinity) <= 2.8);

  return { gallery, canonPhotos, portraitAperture };
}

// ============================================================================
// Export all examples
// ============================================================================

export default {
  example1_basicFileInput,
  example2_singleTagExtraction,
  example3_ReactPhotoMetadata: Example3_ReactPhotoMetadata,
  example4_batchProcessing,
  example5_photoOrganization,
  example6_nodeJsFileSystem,
  example7_errorHandling,
  example8_advancedTags,
  example9_lazyLoading,
  example10_photoGallery,
  LazyExifExtractor,
};

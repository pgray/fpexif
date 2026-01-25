# fpexif WASM Examples - Essential EXIF Tags Extraction

This directory contains comprehensive examples for using the fpexif WASM library to extract essential photography metadata from RAW and JPEG files.

## Quick Start

### 1. Build the WASM Module

```bash
# From the project root
wasm-pack build --target web
```

This creates the WASM module in `pkg/` directory.

### 2. Run the Demo

```bash
# Serve the examples directory
cd examples
python3 -m http.server 8080
# Or use any static file server
```

Open http://localhost:8080/demo.html in your browser.

## Files Overview

### 📄 `extract-essential-tags.ts`

**Main library module** - Provides a clean, typed interface for extracting the 8 essential EXIF tags:

- `make` - Camera manufacturer (e.g., "Canon", "Nikon")
- `model` - Camera model (e.g., "Canon EOS 5D Mark IV")
- `iso` - ISO sensitivity (e.g., 100, 800, 3200)
- `aperture` - Lens aperture (e.g., 2.8, 5.6, 16.0)
- `shutter_speed` - Shutter speed (e.g., "1/1000", "2.5")
- `focal_length` - Focal length (e.g., "50.0 mm", "24.0 mm")
- `create_date` - Date/time photo was taken
- `file_size` - Human-readable file size (e.g., "34 MB", "1.5 GB")

**Key Features:**
- ✅ Case-insensitive tag access (`"make"`, `"Make"`, `"MAKE"` all work)
- ✅ Snake_case support (`"shutter_speed"` = `"ShutterSpeed"`)
- ✅ TypeScript types included
- ✅ Error handling options
- ✅ Single-tag extraction for efficiency

### 📄 `usage-examples.ts`

**10 comprehensive usage examples** covering:

1. Basic file input extraction
2. Case-insensitive single tag extraction
3. React component integration
4. Batch processing multiple files
5. Photo organization by camera/settings
6. Node.js file system integration
7. Error handling patterns
8. Advanced manufacturer-specific tags
9. Performance optimization with lazy loading
10. Photo gallery with metadata display

### 📄 `demo.html`

**Interactive browser demo** - Upload any RAW or JPEG file and see the extracted EXIF data displayed in a clean UI.

## Usage

### Import as a Module

```typescript
import {
  initWasm,
  extractEssentialTags,
  extractTag,
  extractAllExifData,
  formatEssentialTags,
} from './extract-essential-tags.js';

// Initialize WASM (do this once at app startup)
await initWasm();

// Extract essential tags
const file = document.querySelector('input[type="file"]').files[0];
const arrayBuffer = await file.arrayBuffer();
const bytes = new Uint8Array(arrayBuffer);

const tags = await extractEssentialTags(bytes);
console.log(tags);
// {
//   make: "Canon",
//   model: "Canon EOS 5D Mark IV",
//   iso: 100,
//   aperture: 5.6,
//   shutter_speed: "1/800",
//   focal_length: "50.0 mm",
//   create_date: "2024:01:15 14:30:22",
//   file_size: "34 MB"
// }
```

### Extract Single Tags (More Efficient)

```typescript
// All these work (case-insensitive):
const make = await extractTag(bytes, 'make');
const makeUpper = await extractTag(bytes, 'MAKE');
const makePascal = await extractTag(bytes, 'Make');

// Snake_case works too:
const shutterSpeed = await extractTag(bytes, 'shutter_speed');
const shutterSpeedPascal = await extractTag(bytes, 'ShutterSpeed');
```

### React Integration

```typescript
import { initWasm, extractEssentialTags } from './extract-essential-tags.js';

function PhotoUploader() {
  const [exifData, setExifData] = useState(null);

  useEffect(() => {
    initWasm(); // Initialize on mount
  }, []);

  const handleFileChange = async (event) => {
    const file = event.target.files[0];
    const arrayBuffer = await file.arrayBuffer();
    const bytes = new Uint8Array(arrayBuffer);
    const tags = await extractEssentialTags(bytes);
    setExifData(tags);
  };

  return (
    <div>
      <input type="file" onChange={handleFileChange} />
      {exifData && (
        <div>
          <h3>{exifData.make} {exifData.model}</h3>
          <p>ISO {exifData.iso} • f/{exifData.aperture} • {exifData.shutter_speed}</p>
        </div>
      )}
    </div>
  );
}
```

### Node.js Integration

```typescript
import { readFile } from 'fs/promises';
import { initWasm, extractEssentialTags } from './extract-essential-tags.js';

await initWasm();

const buffer = await readFile('/path/to/photo.CR2');
const bytes = new Uint8Array(buffer);
const tags = await extractEssentialTags(bytes);

console.log(`Camera: ${tags.make} ${tags.model}`);
console.log(`Settings: ISO ${tags.iso}, f/${tags.aperture}, ${tags.shutter_speed}`);
```

## API Reference

### `initWasm(): Promise<void>`

Initialize the WASM module. Must be called once before using any extraction functions.

### `extractEssentialTags(imageData, options?): Promise<EssentialExifTags>`

Extract the 8 essential EXIF tags from an image file.

**Parameters:**
- `imageData`: `ArrayBuffer | Uint8Array` - Raw image file bytes
- `options`: `ExtractOptions` (optional)
  - `throwOnError`: `boolean` - Throw errors instead of returning partial data (default: false)

**Returns:** `Promise<EssentialExifTags>`

### `extractTag(imageData, tagName): Promise<string | undefined>`

Extract a single EXIF tag by name (case-insensitive). More efficient than extracting all tags.

**Parameters:**
- `imageData`: `ArrayBuffer | Uint8Array` - Raw image file bytes
- `tagName`: `string` - Tag name (case-insensitive, e.g., "make", "Make", "shutter_speed")

**Returns:** `Promise<string | undefined>`

### `extractAllExifData(imageData): Promise<ExifData | null>`

Extract all EXIF data from an image file (hundreds of tags, including manufacturer-specific).

**Parameters:**
- `imageData`: `ArrayBuffer | Uint8Array` - Raw image file bytes

**Returns:** `Promise<ExifData | null>`

### `formatEssentialTags(tags): string`

Format essential EXIF tags as a human-readable string.

**Parameters:**
- `tags`: `EssentialExifTags` - Tags object from extractEssentialTags()

**Returns:** Formatted string

## Supported Formats

✅ **RAW Formats (26+):**
Canon (CR2, CR3, CRW), Nikon (NEF, NRW), Sony (ARW, SR2, SRF), Fujifilm (RAF), Olympus (ORF), Panasonic (RW2), Pentax (PEF, DNG), Minolta (MRW), Sigma (X3F), Samsung (SRW), Hasselblad (3FR), Phase One (IIQ), Leaf (MOS), Mamiya (MEF), Kodak (DCR, KDC, ERF), Leica (RWL, DNG)

✅ **Standard Formats:**
JPEG, TIFF, DNG

## Performance Tips

1. **Lazy Load WASM**: Initialize WASM only when needed (see Example 9)
2. **Single Tag Extraction**: Use `extractTag()` when you only need 1-2 tags
3. **Batch Processing**: Use `Promise.all()` for parallel processing (see Example 4)
4. **Web Workers**: Process files in a Web Worker for large batches

## Case-Insensitive Access

All tag names are **case-insensitive** and support multiple naming conventions:

```typescript
// All these are equivalent:
await extractTag(bytes, 'make')           // ✅
await extractTag(bytes, 'Make')           // ✅
await extractTag(bytes, 'MAKE')           // ✅

await extractTag(bytes, 'shutter_speed')  // ✅
await extractTag(bytes, 'ShutterSpeed')   // ✅
await extractTag(bytes, 'SHUTTER_SPEED')  // ✅
await extractTag(bytes, 'shutterspeed')   // ✅
```

## Browser Compatibility

- ✅ Chrome 89+
- ✅ Firefox 89+
- ✅ Safari 15+
- ✅ Edge 89+

Requires WebAssembly support (available in all modern browsers).

## TypeScript Types

```typescript
interface EssentialExifTags {
  make?: string;
  model?: string;
  iso?: number;
  aperture?: number;
  shutter_speed?: string;
  focal_length?: string;
  create_date?: string;
  file_size?: string;
}

interface ExifData {
  Make?: string;
  Model?: string;
  ISO?: number;
  Aperture?: number;
  ShutterSpeed?: string;
  FocalLength?: string;
  CreateDate?: string;
  FileSize?: string;
  FileSizeBytes?: number;
  LensModel?: string;
  ExposureMode?: string;
  WhiteBalance?: string;
  // ... hundreds more tags
  [key: string]: any;
}
```

## Examples

See `usage-examples.ts` for 10 comprehensive examples including:
- File upload handling
- React components
- Batch processing
- Photo organization
- Error handling
- Node.js integration
- And more!

## License

Same as fpexif library - MIT OR Apache-2.0

## Questions?

See the main fpexif README or open an issue on GitHub.

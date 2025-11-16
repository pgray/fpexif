#!/bin/bash
# Script to download free sample images for testing
# All samples are from public domain or freely redistributable sources

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Downloading free sample images for fpexif testing..."
echo ""

# Function to download and verify
download_sample() {
    local url="$1"
    local filename="$2"
    local description="$3"

    if [ -f "$filename" ]; then
        echo "✓ $filename already exists (skipping)"
        return
    fi

    echo "Downloading $description..."
    if command -v curl &> /dev/null; then
        curl -L -o "$filename" "$url" || {
            echo "✗ Failed to download $filename"
            rm -f "$filename"
            return 1
        }
    elif command -v wget &> /dev/null; then
        wget -O "$filename" "$url" || {
            echo "✗ Failed to download $filename"
            rm -f "$filename"
            return 1
        }
    else
        echo "✗ Neither curl nor wget found. Please install one of them."
        return 1
    fi

    echo "✓ Downloaded $filename"
}

# Download samples from RawSamples.ch (public domain)
# Note: These are examples - you'll need to visit the site to get actual download URLs
# The site provides downloads but doesn't have direct download links we can script

echo "To download RAW samples, please visit:"
echo "  https://www.rawsamples.ch/"
echo ""
echo "Recommended samples to download:"
echo "  - Canon CRW (Canon Raw v1) - for legacy Canon format"
echo "  - Canon CR2 (Canon Raw 2) - for modern Canon DSLR"
echo "  - Nikon NEF - for Nikon format"
echo "  - Sony ARW - for Sony format"
echo "  - DNG - for Adobe Digital Negative"
echo "  - Olympus ORF - for Olympus format"
echo "  - Pentax PEF - for Pentax format"
echo ""
echo "Place downloaded files in this directory (test-data/) and run:"
echo "  cargo test test_discover_available_files -- --nocapture"
echo ""

# Example: Download a small public domain PNG with EXIF
echo "Downloading sample PNG with EXIF (if available)..."
# Note: This is a placeholder - you'd need an actual URL to a PNG with EXIF
# download_sample "https://example.com/sample.png" "sample.png" "PNG with EXIF"

# Example: Download sample JPEG
echo "You can also create small test files manually:"
echo "  - Take a photo with a smartphone (for HEIC/JPEG)"
echo "  - Export to different formats using GIMP/Photoshop"
echo "  - Use online converters for WebP/AVIF"
echo ""

echo "====================================="
echo "Download instructions complete!"
echo "====================================="
echo ""
echo "Next steps:"
echo "1. Visit https://www.rawsamples.ch/ and download sample files"
echo "2. Place them in this directory (test-data/)"
echo "3. Run: cargo test -- --nocapture"
echo "4. Un-ignore tests in tests/real_file_test.rs as files are added"

#!/usr/bin/env -S deno run --allow-read

/**
 * Tag Diff Tool
 *
 * Compares ExifTool PrintConv mappings against our Rust implementations
 * and generates ready-to-paste Rust code for missing decode functions.
 *
 * Usage: deno run --allow-read scripts/tag-diff.ts <manufacturer>
 * Example: deno run --allow-read scripts/tag-diff.ts canon
 */

interface PrintConvMapping {
  tagId: string;
  tagName: string;
  values: Map<string, string>;
  sourceFile: string;
  lineNumber: number;
}

interface ExistingDecode {
  functionName: string;
  tagName: string;
}

const MANUFACTURER_CONFIG: Record<string, { pmFiles: string[]; rsFile: string }> = {
  canon: {
    pmFiles: ["exiftool/lib/Image/ExifTool/Canon.pm"],
    rsFile: "src/makernotes/canon.rs",
  },
  nikon: {
    pmFiles: ["exiftool/lib/Image/ExifTool/Nikon.pm"],
    rsFile: "src/makernotes/nikon.rs",
  },
  sony: {
    pmFiles: ["exiftool/lib/Image/ExifTool/Sony.pm"],
    rsFile: "src/makernotes/sony.rs",
  },
  fuji: {
    pmFiles: ["exiftool/lib/Image/ExifTool/FujiFilm.pm"],
    rsFile: "src/makernotes/fuji.rs",
  },
  panasonic: {
    pmFiles: ["exiftool/lib/Image/ExifTool/Panasonic.pm"],
    rsFile: "src/makernotes/panasonic.rs",
  },
  olympus: {
    pmFiles: ["exiftool/lib/Image/ExifTool/Olympus.pm"],
    rsFile: "src/makernotes/olympus.rs",
  },
};

function toSnakeCase(name: string): string {
  return name
    .replace(/([A-Z])/g, "_$1")
    .toLowerCase()
    .replace(/^_/, "")
    .replace(/__+/g, "_");
}

function parseExifToolPrintConv(content: string, fileName: string): PrintConvMapping[] {
  const mappings: PrintConvMapping[] = [];
  const lines = content.split("\n");

  let currentTagId = "";
  let currentTagName = "";
  let inPrintConv = false;
  let braceDepth = 0;
  let currentValues = new Map<string, string>();
  let printConvStartLine = 0;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    // Match tag definition: 0x0001 => { Name => 'TagName', ...
    const tagMatch = line.match(/^\s*(0x[0-9a-fA-F]+|\d+)\s*=>\s*\{/);
    if (tagMatch) {
      currentTagId = tagMatch[1];
    }

    // Match tag name: Name => 'TagName'
    const nameMatch = line.match(/Name\s*=>\s*['"]([^'"]+)['"]/);
    if (nameMatch) {
      currentTagName = nameMatch[1];
    }

    // Start of PrintConv block
    if (line.includes("PrintConv") && line.includes("=>") && line.includes("{")) {
      inPrintConv = true;
      braceDepth = 1;
      currentValues = new Map();
      printConvStartLine = i + 1;

      // Check if there are values on the same line
      const sameLine = line.substring(line.indexOf("{") + 1);
      parseValuesFromLine(sameLine, currentValues);
      continue;
    }

    if (inPrintConv) {
      // Count braces
      for (const char of line) {
        if (char === "{") braceDepth++;
        if (char === "}") braceDepth--;
      }

      // Parse value mappings: 0 => 'Value', or 0 => "Value"
      parseValuesFromLine(line, currentValues);

      // End of PrintConv block
      if (braceDepth <= 0) {
        if (currentTagName && currentValues.size > 0) {
          mappings.push({
            tagId: currentTagId,
            tagName: currentTagName,
            values: currentValues,
            sourceFile: fileName,
            lineNumber: printConvStartLine,
          });
        }
        inPrintConv = false;
        currentValues = new Map();
      }
    }
  }

  return mappings;
}

function parseValuesFromLine(line: string, values: Map<string, string>): void {
  // Match patterns like: 0 => 'Value', -1 => "Value", 0x0001 => 'Value'
  const regex = /(-?\d+|0x[0-9a-fA-F]+)\s*=>\s*['"]([^'"]+)['"]/g;
  let match;
  while ((match = regex.exec(line)) !== null) {
    values.set(match[1], match[2]);
  }
}

function parseExistingDecodes(content: string): ExistingDecode[] {
  const decodes: ExistingDecode[] = [];
  const regex = /pub fn (decode_([a-z_0-9]+)_exiftool)/g;
  let match;
  while ((match = regex.exec(content)) !== null) {
    decodes.push({
      functionName: match[1],
      tagName: match[2],
    });
  }
  return decodes;
}

function generateRustDecode(mapping: PrintConvMapping, manufacturer: string): string {
  const fnName = toSnakeCase(mapping.tagName);
  const lines: string[] = [];

  lines.push(`/// Decode ${mapping.tagName} - ExifTool format`);
  lines.push(`/// Source: ${mapping.sourceFile}:${mapping.lineNumber}`);
  lines.push(`pub fn decode_${fnName}_exiftool(value: u16) -> &'static str {`);
  lines.push(`    match value {`);

  for (const [key, val] of mapping.values) {
    const numKey = key.startsWith("0x") ? key : parseInt(key, 10);
    lines.push(`        ${numKey} => "${val}",`);
  }

  lines.push(`        _ => "Unknown",`);
  lines.push(`    }`);
  lines.push(`}`);
  lines.push(``);
  lines.push(`/// Decode ${mapping.tagName} - exiv2 format`);
  lines.push(`pub fn decode_${fnName}_exiv2(value: u16) -> &'static str {`);
  lines.push(`    decode_${fnName}_exiftool(value)`);
  lines.push(`}`);

  return lines.join("\n");
}

async function main() {
  const manufacturer = Deno.args[0]?.toLowerCase();

  if (!manufacturer || !MANUFACTURER_CONFIG[manufacturer]) {
    console.log("Usage: deno run --allow-read scripts/tag-diff.ts <manufacturer>");
    console.log("Available manufacturers:", Object.keys(MANUFACTURER_CONFIG).join(", "));
    Deno.exit(1);
  }

  const config = MANUFACTURER_CONFIG[manufacturer];

  // Parse ExifTool files
  const allMappings: PrintConvMapping[] = [];
  for (const pmFile of config.pmFiles) {
    try {
      const content = await Deno.readTextFile(pmFile);
      const mappings = parseExifToolPrintConv(content, pmFile);
      allMappings.push(...mappings);
    } catch (e) {
      console.error(`Error reading ${pmFile}:`, e);
    }
  }

  // Parse existing Rust implementations
  let existingDecodes: ExistingDecode[] = [];
  try {
    const rsContent = await Deno.readTextFile(config.rsFile);
    existingDecodes = parseExistingDecodes(rsContent);
  } catch (e) {
    console.error(`Error reading ${config.rsFile}:`, e);
  }

  const existingNames = new Set(existingDecodes.map(d => d.tagName));

  // Find missing tags
  const missingMappings = allMappings.filter(m => {
    const snakeName = toSnakeCase(m.tagName);
    return !existingNames.has(snakeName);
  });

  // Output
  console.log(`\n=== ${manufacturer.toUpperCase()} Tag Analysis ===\n`);
  console.log(`ExifTool PrintConv mappings found: ${allMappings.length}`);
  console.log(`Already implemented: ${existingDecodes.length}`);
  console.log(`Missing: ${missingMappings.length}\n`);

  if (missingMappings.length === 0) {
    console.log("All tags implemented!");
    return;
  }

  // Group by value count (prioritize simple enums)
  const simpleEnums = missingMappings.filter(m => m.values.size >= 2 && m.values.size <= 20);
  simpleEnums.sort((a, b) => a.values.size - b.values.size);

  console.log(`=== Top ${Math.min(10, simpleEnums.length)} Missing Tags (simplest first) ===\n`);

  for (const mapping of simpleEnums.slice(0, 10)) {
    console.log(`--- ${mapping.tagName} (${mapping.values.size} values) ---`);
    console.log(`Tag ID: ${mapping.tagId}`);
    console.log(`Source: ${mapping.sourceFile}:${mapping.lineNumber}`);
    console.log(`Values:`);
    for (const [k, v] of mapping.values) {
      console.log(`  ${k} => "${v}"`);
    }
    console.log(`\nRust code:\n`);
    console.log(generateRustDecode(mapping, manufacturer));
    console.log(`\n${"=".repeat(60)}\n`);
  }
}

main();

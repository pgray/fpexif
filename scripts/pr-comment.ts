#!/usr/bin/env -S deno run --allow-read --allow-write

/**
 * PR Comment Generator
 *
 * Reads JSON test result files and generates a markdown report for PR comments.
 * Usage: deno run --allow-read --allow-write scripts/pr-comment.ts [output-file]
 */

interface TestIssue {
  category: string;
  message: string;
  field?: string;
  expected?: string;
  actual?: string;
}

interface FileTestResult {
  file_path: string;
  format: string;
  success: boolean;
  fpexif_tag_count: number;
  reference_tag_count: number;
  matching_tags: number;
  mismatched_tags: number;
  missing_tags: number;
  extra_tags: number;
  issues: TestIssue[];
}

interface FormatTestResult {
  format: string;
  test_name: string;
  reference_tool: string;
  files_tested: number;
  files_passed: number;
  total_issues: number;
  unknown_tags: number;
  missing_fields: number;
  value_mismatches: number;
  critical_issues: number;
  total_matching_tags: number;
  total_mismatched_tags: number;
  total_missing_tags: number;
  total_extra_tags: number;
  file_results: FileTestResult[];
}

const RESULTS_DIR = "test-results";

async function loadResults(): Promise<FormatTestResult[]> {
  const results: FormatTestResult[] = [];

  try {
    for await (const entry of Deno.readDir(RESULTS_DIR)) {
      if (entry.isFile && entry.name.endsWith(".json")) {
        const content = await Deno.readTextFile(`${RESULTS_DIR}/${entry.name}`);
        const result = JSON.parse(content) as FormatTestResult;
        results.push(result);
      }
    }
  } catch (e) {
    if (e instanceof Deno.errors.NotFound) {
      console.error(`No results directory found at ${RESULTS_DIR}`);
    } else {
      throw e;
    }
  }

  return results.sort((a, b) => a.format.localeCompare(b.format));
}

function getFileName(path: string): string {
  return path.split("/").pop() || path;
}

function generateReport(results: FormatTestResult[]): string {
  const lines: string[] = [];

  // Header
  lines.push("## Test Issues Report");
  lines.push("");

  if (results.length === 0) {
    lines.push("### All Clear!");
    lines.push("");
    lines.push("No test result files found.");
    return lines.join("\n");
  }

  // Calculate totals
  const totalUnknown = results.reduce((sum, r) => sum + r.unknown_tags, 0);
  const totalMissing = results.reduce((sum, r) => sum + r.missing_fields, 0);
  const totalMismatches = results.reduce((sum, r) => sum + r.value_mismatches, 0);
  const totalCritical = results.reduce((sum, r) => sum + r.critical_issues, 0);
  const totalIssues = totalUnknown + totalMissing + totalMismatches;
  const totalFiles = results.reduce((sum, r) => sum + r.files_tested, 0);
  const totalPassed = results.reduce((sum, r) => sum + r.files_passed, 0);
  const totalMatching = results.reduce((sum, r) => sum + (r.total_matching_tags || 0), 0);
  const totalMismatchedTags = results.reduce((sum, r) => sum + (r.total_mismatched_tags || 0), 0);
  const totalMissingTags = results.reduce((sum, r) => sum + (r.total_missing_tags || 0), 0);
  const totalExtraTags = results.reduce((sum, r) => sum + (r.total_extra_tags || 0), 0);
  const totalComparableTags = totalMatching + totalMismatchedTags + totalMissingTags;
  const matchRate = totalComparableTags > 0 ? ((totalMatching / totalComparableTags) * 100).toFixed(1) : "N/A";

  // Summary
  lines.push("### Summary");
  lines.push("");
  lines.push(`| Metric | Count |`);
  lines.push(`|--------|-------|`);
  lines.push(`| Files Tested | ${totalFiles} (${totalPassed} passed) |`);
  lines.push(`| **Match Rate** | **${matchRate}%** |`);
  lines.push(`| Matching Tags | ${totalMatching} |`);
  lines.push(`| Mismatched Tags | ${totalMismatchedTags} |`);
  lines.push(`| Missing Tags | ${totalMissingTags} |`);
  lines.push(`| Extra Tags | ${totalExtraTags} |`);
  if (totalCritical > 0) {
    lines.push(`| **Critical Issues** | **${totalCritical}** |`);
  }
  lines.push("");

  // Summary table by format
  lines.push("### Results by Format");
  lines.push("");
  lines.push("| Format | Files | Match % | ✓ Match | ✗ Mismatch | ⚠ Missing | + Extra |");
  lines.push("|--------|-------|---------|---------|------------|-----------|---------|");

  for (const r of results) {
    const matching = r.total_matching_tags || 0;
    const mismatched = r.total_mismatched_tags || 0;
    const missing = r.total_missing_tags || 0;
    const extra = r.total_extra_tags || 0;
    const comparable = matching + mismatched + missing;
    const rate = comparable > 0 ? ((matching / comparable) * 100).toFixed(1) : "N/A";
    lines.push(
      `| ${r.format} | ${r.files_tested} | ${rate}% | ${matching} | ${mismatched} | ${missing} | ${extra} |`
    );
  }
  lines.push("");

  // Per-file details for each format
  lines.push("### Per-File Results");
  lines.push("");

  for (const r of results) {
    if (r.file_results.length === 0) continue;

    lines.push("<details>");
    const formatMatching = r.total_matching_tags || 0;
    const formatMismatched = r.total_mismatched_tags || 0;
    const formatMissing = r.total_missing_tags || 0;
    const formatComparable = formatMatching + formatMismatched + formatMissing;
    const formatRate = formatComparable > 0 ? ((formatMatching / formatComparable) * 100).toFixed(1) : "N/A";
    lines.push(
      `<summary><b>${r.format}</b> - ${r.files_tested} files, ${formatRate}% match rate</summary>`
    );
    lines.push("");

    // Per-file table
    lines.push("| File | ✓ Match | ✗ Mismatch | ⚠ Missing | + Extra | Status |");
    lines.push("|------|---------|------------|-----------|---------|--------|");

    for (const f of r.file_results) {
      const fileName = getFileName(f.file_path);
      const matching = f.matching_tags || 0;
      const mismatched = f.mismatched_tags || 0;
      const missing = f.missing_tags || 0;
      const extra = f.extra_tags || 0;
      const status = f.success ? "✓" : "✗";
      lines.push(
        `| \`${fileName}\` | ${matching} | ${mismatched} | ${missing} | ${extra} | ${status} |`
      );
    }
    lines.push("");

    // Show detailed issues for this format
    const mismatchIssues = r.file_results.flatMap((f) =>
      f.issues.filter((i) => i.category === "value_mismatch")
    );

    if (mismatchIssues.length > 0) {
      lines.push("**Value Mismatches:**");
      lines.push("```");
      for (const issue of mismatchIssues.slice(0, 10)) {
        lines.push(`  ${issue.message}`);
      }
      if (mismatchIssues.length > 10) {
        lines.push(`  ... and ${mismatchIssues.length - 10} more`);
      }
      lines.push("```");
      lines.push("");
    }

    const missingIssues = r.file_results.flatMap((f) =>
      f.issues.filter((i) => i.category === "missing_field")
    );

    if (missingIssues.length > 0) {
      lines.push("**Missing Fields:**");
      lines.push("```");
      const uniqueFields = [...new Set(missingIssues.map((i) => i.field).filter(Boolean))];
      for (const field of uniqueFields.slice(0, 15)) {
        lines.push(`  ${field}`);
      }
      if (uniqueFields.length > 15) {
        lines.push(`  ... and ${uniqueFields.length - 15} more`);
      }
      lines.push("```");
      lines.push("");
    }

    lines.push("</details>");
    lines.push("");
  }

  // All clear message if no issues
  if (totalIssues === 0) {
    lines.push("### All Clear!");
    lines.push("");
    lines.push("All tests passed with no issues detected.");
  }

  return lines.join("\n");
}

async function main() {
  const outputFile = Deno.args[0] || "test-issues-report.md";

  console.log(`Loading test results from ${RESULTS_DIR}...`);
  const results = await loadResults();
  console.log(`Found ${results.length} result files`);

  const report = generateReport(results);

  await Deno.writeTextFile(outputFile, report);
  console.log(`Report written to ${outputFile}`);

  // Print summary to stdout
  const totalCritical = results.reduce((sum, r) => sum + r.critical_issues, 0);
  if (totalCritical > 0) {
    console.log(`\nWARNING: Found ${totalCritical} critical issues`);
    Deno.exit(1);
  }
}

main();

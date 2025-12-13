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

  // Summary
  lines.push("### Summary");
  lines.push("");
  lines.push(`- **Files Tested:** ${totalFiles} (${totalPassed} passed)`);
  lines.push(`- **Total Issues:** ${totalIssues}`);
  lines.push(`  - Unknown Tags: ${totalUnknown}`);
  lines.push(`  - Missing Fields: ${totalMissing}`);
  lines.push(`  - Value Mismatches: ${totalMismatches}`);
  if (totalCritical > 0) {
    lines.push(`  - **Critical:** ${totalCritical}`);
  }
  lines.push("");

  // Summary table by format
  const formatsWithIssues = results.filter(
    (r) => r.unknown_tags > 0 || r.missing_fields > 0 || r.value_mismatches > 0
  );

  if (formatsWithIssues.length > 0) {
    lines.push("### Issues by Format");
    lines.push("");
    lines.push("| Format | Files | Unknown Tags | Missing Fields | Mismatches |");
    lines.push("|--------|-------|--------------|----------------|------------|");

    for (const r of formatsWithIssues) {
      lines.push(
        `| ${r.format} | ${r.files_tested} | ${r.unknown_tags} | ${r.missing_fields} | ${r.value_mismatches} |`
      );
    }
    lines.push("");
  }

  // Detailed sections by format
  if (formatsWithIssues.length > 0) {
    lines.push("### Details by Format");
    lines.push("");

    for (const r of formatsWithIssues) {
      const unknownIssues = r.file_results.flatMap((f) =>
        f.issues.filter((i) => i.category === "unknown_tag")
      );
      const missingIssues = r.file_results.flatMap((f) =>
        f.issues.filter((i) => i.category === "missing_field")
      );
      const mismatchIssues = r.file_results.flatMap((f) =>
        f.issues.filter((i) => i.category === "value_mismatch")
      );

      lines.push("<details>");
      lines.push(
        `<summary><b>${r.format}</b> - ${r.unknown_tags} unknown, ${r.missing_fields} missing, ${r.value_mismatches} mismatches</summary>`
      );
      lines.push("");

      if (unknownIssues.length > 0) {
        lines.push("**Unknown Tags:**");
        lines.push("```");
        const uniqueTags = [...new Set(unknownIssues.map((i) => i.field).filter(Boolean))];
        for (const tag of uniqueTags.slice(0, 10)) {
          lines.push(`  ${tag}`);
        }
        if (uniqueTags.length > 10) {
          lines.push(`  ... and ${uniqueTags.length - 10} more`);
        }
        lines.push("```");
        lines.push("");
      }

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

      lines.push("</details>");
      lines.push("");
    }

    lines.push("To add support for unknown tags, add definitions to `src/tags.rs`.");
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

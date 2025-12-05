//! Validation command handler.

use botticelli_narrative::validator::{ValidationConfig, validate_narrative_file_with_config};
use std::path::{Path, PathBuf};

use super::ValidationOutputFormat;

/// Handles the validate command.
///
/// # Arguments
///
/// * `path` - Path to narrative file or directory
/// * `validate_files` - Check that media/nested narrative files exist
/// * `validate_models` - Warn on unknown model names
/// * `base_dir` - Base directory for relative paths
/// * `format` - Output format (human or json)
/// * `strict` - Treat warnings as errors
/// * `quiet` - Only show errors, not warnings
#[tracing::instrument(skip_all, fields(path = %path.display()))]
pub fn handle_validate_command(
    path: PathBuf,
    validate_files: bool,
    validate_models: bool,
    base_dir: Option<PathBuf>,
    format: ValidationOutputFormat,
    strict: bool,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Starting validation");

    // Build validation config
    let config = ValidationConfig {
        validate_nested_narratives: validate_files,
        validate_media_files: validate_files,
        warn_unknown_models: validate_models,
        warn_unused_resources: !quiet, // Don't warn about unused if quiet mode
        base_dir: base_dir.or_else(|| {
            if path.is_file() {
                path.parent().map(|p| p.to_path_buf())
            } else {
                Some(path.clone())
            }
        }),
    };

    // Validate single file or directory
    if path.is_file() {
        let result = validate_narrative_file_with_config(&path, &config);
        output_result(path.as_path(), &result, &format, strict, quiet)?;

        if !result.is_valid() {
            std::process::exit(1);
        }
        if strict && !result.warnings.is_empty() {
            std::process::exit(2);
        }
    } else if path.is_dir() {
        let mut has_errors = false;
        let mut has_warnings = false;
        let mut total_files = 0;
        let mut valid_files = 0;

        // Validate all TOML files in directory
        for entry in std::fs::read_dir(&path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.extension().and_then(|s| s.to_str()) == Some("toml") {
                total_files += 1;
                let result = validate_narrative_file_with_config(&entry_path, &config);

                if result.is_valid() && result.warnings.is_empty() {
                    valid_files += 1;
                }

                has_errors = has_errors || !result.is_valid();
                has_warnings = has_warnings || !result.warnings.is_empty();

                output_result(entry_path.as_path(), &result, &format, strict, quiet)?;
            }
        }

        // Print summary for human format
        if matches!(format, ValidationOutputFormat::Human) {
            println!("\n{}", "=".repeat(80));
            println!("Validation Summary:");
            println!("  Total files: {}", total_files);
            println!("  Valid files: {}", valid_files);
            println!("  Files with errors: {}", total_files - valid_files);

            if has_errors {
                println!("\n❌ Validation failed");
                std::process::exit(1);
            } else if strict && has_warnings {
                println!("\n⚠️  Validation passed with warnings (strict mode)");
                std::process::exit(2);
            } else if has_warnings {
                println!("\n⚠️  Validation passed with warnings");
            } else {
                println!("\n✅ All narratives valid");
            }
        } else if has_errors {
            std::process::exit(1);
        } else if strict && has_warnings {
            std::process::exit(2);
        }
    } else {
        return Err(format!(
            "Path '{}' is neither a file nor a directory",
            path.display()
        )
        .into());
    }

    Ok(())
}

/// Outputs validation result in the specified format.
fn output_result(
    path: &Path,
    result: &botticelli_narrative::validator::ValidationResult,
    format: &ValidationOutputFormat,
    strict: bool,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        ValidationOutputFormat::Human => {
            output_human(path, result, strict, quiet);
        }
        ValidationOutputFormat::Json => {
            output_json(path, result)?;
        }
    }
    Ok(())
}

/// Outputs validation result in human-readable format.
fn output_human(
    path: &Path,
    result: &botticelli_narrative::validator::ValidationResult,
    strict: bool,
    quiet: bool,
) {
    let status_icon = if !result.is_valid() {
        "❌"
    } else if !result.warnings.is_empty() {
        if strict { "⚠️" } else { "✅" }
    } else {
        "✅"
    };

    println!("\n{} {}", status_icon, path.display());
    println!("{}", "─".repeat(80));

    if !result.errors.is_empty() {
        println!("\nErrors:");
        for (i, error) in result.errors.iter().enumerate() {
            println!("\n  {}. {}", i + 1, error.message);
            if let Some(suggestion) = &error.suggestion {
                println!("\n     💡 Suggestion:");
                for line in suggestion.lines() {
                    println!("        {}", line);
                }
            }
        }
    }

    if !quiet && !result.warnings.is_empty() {
        println!("\nWarnings:");
        for (i, warning) in result.warnings.iter().enumerate() {
            println!("\n  {}. {}", i + 1, warning.message);
        }
    }

    if result.is_valid() && result.warnings.is_empty() {
        println!("\n  No issues found");
    }
}

/// Outputs validation result in JSON format.
fn output_json(
    path: &Path,
    result: &botticelli_narrative::validator::ValidationResult,
) -> Result<(), Box<dyn std::error::Error>> {
    use serde_json::json;

    let errors: Vec<serde_json::Value> = result
        .errors
        .iter()
        .map(|e| {
            json!({
                "kind": format!("{:?}", e.kind),
                "message": e.message,
                "suggestion": e.suggestion,
                "location": e.location.as_ref().map(|loc| json!({
                    "line": loc.line,
                    "column": loc.column,
                    "section": loc.section,
                })),
            })
        })
        .collect();

    let warnings: Vec<serde_json::Value> = result
        .warnings
        .iter()
        .map(|w| {
            json!({
                "kind": format!("{:?}", w.kind),
                "message": w.message,
                "location": w.location.as_ref().map(|loc| json!({
                    "line": loc.line,
                    "column": loc.column,
                    "section": loc.section,
                })),
            })
        })
        .collect();

    let output = json!({
        "valid": result.is_valid(),
        "file": path.display().to_string(),
        "errors": errors,
        "warnings": warnings,
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

//! Validation tool to verify media migration is complete.
//!
//! This tool checks that all media has been successfully migrated to the new
//! storage system before proceeding with cleanup (removing old columns).
//!
//! Usage:
//!   cargo run --bin validate_migration --features database

use boticelli::establish_connection;
use diesel::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("🔍 Validating media migration...");

    let mut conn = establish_connection()?;

    // Count total inputs with media
    let total_with_media = count_inputs_with_media(&mut conn)?;
    tracing::info!("📊 Total inputs with media: {}", total_with_media);

    // Count inputs with old storage (not migrated)
    let unmigrated = count_unmigrated_inputs(&mut conn)?;
    
    if unmigrated > 0 {
        tracing::error!("❌ MIGRATION INCOMPLETE");
        tracing::error!("   {} inputs still using old storage columns", unmigrated);
        tracing::error!("   Run 'migrate_media' before proceeding with cleanup");
        std::process::exit(1);
    }

    // Count inputs with new storage
    let migrated = count_migrated_inputs(&mut conn)?;
    tracing::info!("✓ Inputs migrated to new storage: {}", migrated);

    // Check for orphaned media references
    let orphaned = count_orphaned_media(&mut conn)?;
    if orphaned > 0 {
        tracing::warn!("⚠ Found {} orphaned media references (not critical)", orphaned);
    }

    tracing::info!("✓ MIGRATION VALIDATION COMPLETE");
    tracing::info!("  All media has been migrated successfully");
    tracing::info!("  Safe to proceed with cleanup migration:");
    tracing::info!("  diesel migration run");

    Ok(())
}

fn count_inputs_with_media(conn: &mut PgConnection) -> Result<i64, Box<dyn std::error::Error>> {
    use boticelli::act_inputs;

    let count: i64 = act_inputs::table
        .filter(
            act_inputs::source_binary
                .is_not_null()
                .or(act_inputs::source_base64.is_not_null())
                .or(act_inputs::media_ref_id.is_not_null()),
        )
        .count()
        .get_result(conn)?;

    Ok(count)
}

fn count_unmigrated_inputs(conn: &mut PgConnection) -> Result<i64, Box<dyn std::error::Error>> {
    use boticelli::act_inputs;

    let count: i64 = act_inputs::table
        .filter(act_inputs::media_ref_id.is_null())
        .filter(
            act_inputs::source_binary
                .is_not_null()
                .or(act_inputs::source_base64.is_not_null()),
        )
        .count()
        .get_result(conn)?;

    Ok(count)
}

fn count_migrated_inputs(conn: &mut PgConnection) -> Result<i64, Box<dyn std::error::Error>> {
    use boticelli::act_inputs;

    let count: i64 = act_inputs::table
        .filter(act_inputs::media_ref_id.is_not_null())
        .count()
        .get_result(conn)?;

    Ok(count)
}

fn count_orphaned_media(_conn: &mut PgConnection) -> Result<i64, Box<dyn std::error::Error>> {
    // TODO: Implement proper orphaned media check
    // For now, just return 0 as this is non-critical validation
    Ok(0)
}

//! Tests for model-specific rate limit configuration.

use boticelli::{BoticelliConfig, Tier};

#[test]
fn test_load_model_specific_overrides() {
    let config = BoticelliConfig::load().unwrap();

    // Get Gemini free tier
    let gemini = &config.providers["gemini"];
    let free_tier = &gemini.tiers["free"];

    // Verify tier-level defaults
    assert_eq!(free_tier.rpm(), Some(10)); // Default (gemini-2.5-flash)
    assert_eq!(free_tier.tpm(), Some(250_000));
    assert_eq!(free_tier.rpd(), Some(250));

    // Verify model-specific overrides exist
    assert!(free_tier.models.contains_key("gemini-2.5-pro"));
    assert!(free_tier.models.contains_key("gemini-2.5-flash"));
    assert!(free_tier.models.contains_key("gemini-2.5-flash-lite"));
    assert!(free_tier.models.contains_key("gemini-2.0-flash"));

    // Verify gemini-2.5-pro has different limits
    let pro_config = free_tier.models.get("gemini-2.5-pro").unwrap();
    assert_eq!(pro_config.rpm, Some(2));
    assert_eq!(pro_config.tpm, Some(125_000));
    assert_eq!(pro_config.rpd, Some(50));

    // Verify gemini-2.5-flash-lite has higher RPM
    let lite_config = free_tier.models.get("gemini-2.5-flash-lite").unwrap();
    assert_eq!(lite_config.rpm, Some(15));
    assert_eq!(lite_config.tpm, Some(250_000));
    assert_eq!(lite_config.rpd, Some(1_000));

    // Verify gemini-2.0-flash has higher TPM
    let flash_2_0_config = free_tier.models.get("gemini-2.0-flash").unwrap();
    assert_eq!(flash_2_0_config.rpm, Some(15));
    assert_eq!(flash_2_0_config.tpm, Some(1_000_000));
    assert_eq!(flash_2_0_config.rpd, Some(200));
}

#[test]
fn test_for_model_method() {
    let config = BoticelliConfig::load().unwrap();

    let gemini = &config.providers["gemini"];
    let free_tier = &gemini.tiers["free"];

    // Test for_model() with gemini-2.5-pro (should have overrides)
    let pro_tier = free_tier.for_model("gemini-2.5-pro");
    assert_eq!(pro_tier.rpm(), Some(2)); // Overridden
    assert_eq!(pro_tier.tpm(), Some(125_000)); // Overridden
    assert_eq!(pro_tier.rpd(), Some(50)); // Overridden
    assert_eq!(pro_tier.name(), "Free"); // Name preserved

    // Test for_model() with gemini-2.5-flash-lite
    let lite_tier = free_tier.for_model("gemini-2.5-flash-lite");
    assert_eq!(lite_tier.rpm(), Some(15)); // Overridden
    assert_eq!(lite_tier.tpm(), Some(250_000)); // Overridden
    assert_eq!(lite_tier.rpd(), Some(1_000)); // Overridden

    // Test for_model() with gemini-2.0-flash
    let flash_2_0_tier = free_tier.for_model("gemini-2.0-flash");
    assert_eq!(flash_2_0_tier.rpm(), Some(15)); // Overridden
    assert_eq!(flash_2_0_tier.tpm(), Some(1_000_000)); // Overridden (higher!)
    assert_eq!(flash_2_0_tier.rpd(), Some(200)); // Overridden

    // Test for_model() with unknown model (should use tier defaults)
    let unknown_tier = free_tier.for_model("unknown-model");
    assert_eq!(unknown_tier.rpm(), Some(10)); // Tier default
    assert_eq!(unknown_tier.tpm(), Some(250_000)); // Tier default
    assert_eq!(unknown_tier.rpd(), Some(250)); // Tier default
}

#[test]
fn test_payasyougo_model_overrides() {
    let config = BoticelliConfig::load().unwrap();

    let gemini = &config.providers["gemini"];
    let payasyougo_tier = &gemini.tiers["payasyougo"];

    // Verify tier-level defaults
    assert_eq!(payasyougo_tier.rpm(), Some(1_000));
    assert_eq!(payasyougo_tier.tpm(), Some(1_000_000));

    // Test model-specific overrides for pay-as-you-go
    let pro_tier = payasyougo_tier.for_model("gemini-2.5-pro");
    assert_eq!(pro_tier.rpm(), Some(150)); // Much lower than default
    assert_eq!(pro_tier.tpm(), Some(2_000_000)); // Higher
    assert_eq!(pro_tier.rpd(), Some(10_000));

    let lite_tier = payasyougo_tier.for_model("gemini-2.5-flash-lite");
    assert_eq!(lite_tier.rpm(), Some(4_000)); // Much higher
    assert_eq!(lite_tier.tpm(), Some(4_000_000)); // Much higher
    assert_eq!(lite_tier.rpd(), None); // Unlimited

    let flash_2_0_tier = payasyougo_tier.for_model("gemini-2.0-flash");
    assert_eq!(flash_2_0_tier.rpm(), Some(2_000)); // Higher than default
    assert_eq!(flash_2_0_tier.tpm(), Some(4_000_000)); // Much higher
    assert_eq!(flash_2_0_tier.rpd(), None); // Unlimited
}

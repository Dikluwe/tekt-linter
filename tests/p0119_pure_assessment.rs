use crystalline_lint::entities::config_init::{decide_init, ConfigDestinationState, InitPlan};

// These tests intentionally name only the public L1 surface frozen by the P0119
// contract.  They do not import a serializer, filesystem adapter, CLI projection, or
// implementation-owned template.

#[test]
fn present_refusal_does_not_require_topology() {
    assert!(matches!(
        decide_init(ConfigDestinationState::Present, None),
        InitPlan::RefusePresent
    ));
}

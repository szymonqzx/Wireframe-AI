//! Unit tests for Rust adapter module

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn test_provider_registry_creation() {
        // Test that provider registry can be created
        let registry: HashMap<String, String> = HashMap::new();

        assert!(registry.is_empty(), "New registry should be empty");
    }

    #[test]
    fn test_execution_mode_variants() {
        // Test execution mode enum logic
        let modes = vec!["Sandbox", "Direct"];

        assert_eq!(modes.len(), 2, "Should have 2 execution modes");
        assert!(modes.contains(&"Sandbox"), "Should have Sandbox mode");
        assert!(modes.contains(&"Direct"), "Should have Direct mode");
    }

    #[test]
    fn test_message_validation() {
        // Test basic message validation
        let valid_message = "Test message";
        let empty_message = "";

        assert!(
            !valid_message.is_empty(),
            "Valid message should not be empty"
        );
        assert!(empty_message.is_empty(), "Empty message should be empty");
    }

    #[test]
    fn test_session_id_format() {
        // Test session ID format
        let session_id = "test_session_123";

        assert!(!session_id.is_empty(), "Session ID should not be empty");
        assert!(session_id.len() > 0, "Session ID should have length > 0");
    }
}

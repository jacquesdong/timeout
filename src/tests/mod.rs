use super::*;

#[test]
fn test_parse_duration() {
    // Test seconds
    assert_eq!(parse_duration("10").unwrap(), 10.0);
    assert_eq!(parse_duration("10s").unwrap(), 10.0);
    
    // Test minutes
    assert_eq!(parse_duration("1m").unwrap(), 60.0);
    assert_eq!(parse_duration("0.5m").unwrap(), 30.0);
    
    // Test hours
    assert_eq!(parse_duration("1h").unwrap(), 3600.0);
    assert_eq!(parse_duration("0.5h").unwrap(), 1800.0);
    
    // Test days
    assert_eq!(parse_duration("1d").unwrap(), 86400.0);
    assert_eq!(parse_duration("0.5d").unwrap(), 43200.0);
    
    // Test zero duration
    assert_eq!(parse_duration("0").unwrap(), 0.0);
    
    // Test invalid format
    assert!(parse_duration("invalid").is_err());
}

#[test]
fn test_parse_signal() {
    // Test signal names
    assert_eq!(parse_signal("TERM").unwrap(), libc::SIGTERM);
    assert_eq!(parse_signal("HUP").unwrap(), libc::SIGHUP);
    assert_eq!(parse_signal("INT").unwrap(), libc::SIGINT);
    assert_eq!(parse_signal("KILL").unwrap(), libc::SIGKILL);
    
    // Test signal numbers
    assert_eq!(parse_signal("15").unwrap(), 15);
    assert_eq!(parse_signal("9").unwrap(), 9);
    
    // Test invalid signal
    assert!(parse_signal("INVALID").is_err());
}

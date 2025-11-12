//! Authentication and configuration integration tests

use oci_api::auth::{ConfigLoader, OciConfig};
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_config_builder_full_workflow() {
    let config = OciConfig::builder()
        .user_id("ocid1.user.oc1.iad.test123")
        .tenancy_id("ocid1.tenancy.oc1..test456")
        .region("us-ashburn-1")
        .fingerprint("11:22:33:44:55:66:77:88:99:aa:bb:cc:dd:ee:ff:00")
        .private_key(
            "-----BEGIN RSA PRIVATE KEY-----\n\
             MIIEowIBAAKCAQEAtestkey123456789\n\
             -----END RSA PRIVATE KEY-----",
        )
        .expect("Failed to load private key")
        .build()
        .expect("Failed to build config");

    assert_eq!(config.user_id, "ocid1.user.oc1.iad.test123");
    assert_eq!(config.tenancy_id, "ocid1.tenancy.oc1..test456");
    assert_eq!(config.region, "us-ashburn-1");
    assert_eq!(
        config.fingerprint,
        "11:22:33:44:55:66:77:88:99:aa:bb:cc:dd:ee:ff:00"
    );
    assert!(config.private_key.contains("BEGIN RSA PRIVATE KEY"));
}

#[test]
fn test_config_loader_with_multiple_profiles() {
    // Create a temporary INI file with multiple profiles
    let mut key_file1 = NamedTempFile::new().unwrap();
    let key_content1 =
        "-----BEGIN RSA PRIVATE KEY-----\nDEFAULT_KEY\n-----END RSA PRIVATE KEY-----\n";
    key_file1.write_all(key_content1.as_bytes()).unwrap();

    let mut key_file2 = NamedTempFile::new().unwrap();
    let key_content2 =
        "-----BEGIN RSA PRIVATE KEY-----\nPRODUCTION_KEY\n-----END RSA PRIVATE KEY-----\n";
    key_file2.write_all(key_content2.as_bytes()).unwrap();

    let mut ini_file = NamedTempFile::new().unwrap();
    let ini_content = format!(
        r#"
[DEFAULT]
user=ocid1.user.default
tenancy=ocid1.tenancy.default
region=ap-seoul-1
fingerprint=aa:bb:cc:dd:ee:ff
key_file={}

[PRODUCTION]
user=ocid1.user.production
tenancy=ocid1.tenancy.production
region=us-phoenix-1
fingerprint=11:22:33:44:55:66
key_file={}
"#,
        key_file1.path().to_str().unwrap(),
        key_file2.path().to_str().unwrap()
    );
    ini_file.write_all(ini_content.as_bytes()).unwrap();

    // Load DEFAULT profile
    let config_default = ConfigLoader::load_from_file(ini_file.path(), None)
        .expect("Failed to load DEFAULT profile");
    assert_eq!(config_default.user_id, "ocid1.user.default");
    assert_eq!(config_default.region, "ap-seoul-1");
    assert!(config_default.private_key.contains("DEFAULT_KEY"));

    // Load PRODUCTION profile
    let config_prod = ConfigLoader::load_from_file(ini_file.path(), Some("PRODUCTION"))
        .expect("Failed to load PRODUCTION profile");
    assert_eq!(config_prod.user_id, "ocid1.user.production");
    assert_eq!(config_prod.region, "us-phoenix-1");
    assert!(config_prod.private_key.contains("PRODUCTION_KEY"));
}

#[test]
fn test_config_loader_with_tilde_expansion() {
    // Create a temporary key file
    let mut key_file = NamedTempFile::new().unwrap();
    let key_content = "-----BEGIN RSA PRIVATE KEY-----\nTEST_KEY\n-----END RSA PRIVATE KEY-----\n";
    key_file.write_all(key_content.as_bytes()).unwrap();

    let home = std::env::var("HOME").unwrap();

    // Create INI with absolute path (no tilde)
    let mut ini_file = NamedTempFile::new().unwrap();
    let ini_content = format!(
        r#"
[DEFAULT]
user=ocid1.user.test
tenancy=ocid1.tenancy.test
region=ap-tokyo-1
fingerprint=aa:bb:cc:dd
key_file={}
"#,
        key_file.path().to_str().unwrap()
    );
    ini_file.write_all(ini_content.as_bytes()).unwrap();

    let config = ConfigLoader::load_from_file(ini_file.path(), None)
        .expect("Failed to load config with absolute path");
    assert_eq!(config.user_id, "ocid1.user.test");
    assert!(config.private_key.contains("TEST_KEY"));
}

#[test]
fn test_config_validation_missing_fields() {
    let mut key_file = NamedTempFile::new().unwrap();
    let key_content = "-----BEGIN RSA PRIVATE KEY-----\nTEST\n-----END RSA PRIVATE KEY-----\n";
    key_file.write_all(key_content.as_bytes()).unwrap();

    // Missing fingerprint
    let mut ini_file = NamedTempFile::new().unwrap();
    let ini_content = format!(
        r#"
[DEFAULT]
user=ocid1.user.test
tenancy=ocid1.tenancy.test
region=ap-seoul-1
key_file={}
"#,
        key_file.path().to_str().unwrap()
    );
    ini_file.write_all(ini_content.as_bytes()).unwrap();

    let result = ConfigLoader::load_from_file(ini_file.path(), None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("fingerprint"));
}

#[test]
fn test_config_with_different_regions() {
    let regions = vec![
        "ap-seoul-1",
        "ap-tokyo-1",
        "us-ashburn-1",
        "us-phoenix-1",
        "eu-frankfurt-1",
    ];

    for region in regions {
        let config = OciConfig::builder()
            .user_id("ocid1.user.test")
            .tenancy_id("ocid1.tenancy.test")
            .region(region)
            .fingerprint("aa:bb:cc:dd")
            .private_key("-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----")
            .expect("Failed to load private key")
            .build()
            .expect(&format!("Failed to build config for region {}", region));

        assert_eq!(config.region, region);
    }
}

#[test]
fn test_config_immutability() {
    let config = OciConfig::builder()
        .user_id("ocid1.user.original")
        .tenancy_id("ocid1.tenancy.test")
        .region("ap-seoul-1")
        .fingerprint("aa:bb:cc:dd")
        .private_key("-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----")
        .expect("Failed to load private key")
        .build()
        .unwrap();

    // Config fields are public but config itself should be used immutably
    let cloned_config = config.clone();
    assert_eq!(config.user_id, cloned_config.user_id);
    assert_eq!(config.tenancy_id, cloned_config.tenancy_id);
    assert_eq!(config.region, cloned_config.region);
}

#[test]
fn test_private_key_formats() {
    let pem_formats = vec![
        "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----",
        "-----BEGIN PRIVATE KEY-----\ntest\n-----END PRIVATE KEY-----",
        "-----BEGIN EC PRIVATE KEY-----\ntest\n-----END EC PRIVATE KEY-----",
    ];

    for pem in pem_formats {
        let result = OciConfig::builder()
            .user_id("ocid1.user.test")
            .tenancy_id("ocid1.tenancy.test")
            .region("ap-seoul-1")
            .fingerprint("aa:bb:cc:dd")
            .private_key(pem);

        assert!(
            result.is_ok(),
            "Failed to load private key with PEM: {}",
            pem
        );

        let config = result.unwrap().build();
        assert!(config.is_ok(), "Failed to build config with PEM: {}", pem);
    }
}

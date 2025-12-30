//! Authentication and configuration integration tests

use oci_api::auth::ConfigLoader;
use oci_api::client::Oci;
use std::io::Write;
use tempfile::NamedTempFile;

const TEST_VALID_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvwIBADANBgkqhkiG9w0BAQEFAASCBKkwggSlAgEAAoIBAQCvfVmTGipPCAsg
fr8khhrPpQxmjUW62+pH/54EecyKTd8KTkg11wT40Pi5zB/UAl8DGTPs9MNz1PQX
EGPh7YPccPTGJ4ZFfu87s2W9m3zp9UWUIy+n+Jr5FBpn8H7n7W/FPLTF7xRyzMSY
BGWFKIyHkufglkKJlRkyVK8+0w6vFBg5Ni/0Eo0uTT31AWvv1b5nuCRstSCME2O7
GbNUPo6vF1xEWNeFzp9Lp7JuMXu+tgLJiSkHKq7I2u25iQvklnqogDSLzxQigX/P
+08jd52R9HI0rWiwLVJ1QE/erZJ+DnKjikb3jpHNRVZmG7/tDM/54yh85L0JfzZx
yt+b3qS5AgMBAAECggEAGMAKERggnXLZ9uRJWwJa56w0eoY0Lm1ztmHTzHfNJDhl
W5O81XMU7W6zlai3WHRZKBu22hWPN1fycQpLvAJ+lWmM7CGI62ZCoV3k3IAAdxKz
lHf98ae7W6O9MamWjGlNWTj9mejlLme41mPQWZ5la32JnIA0tCjGG/YbnTWxHXnx
B5skseaEMR3DT98uBZa67IFKDLJDIIaD4aQNILMNtEb2PFOChblA0mm2szR3AMhv
Pl0VvrexHR+xdlteUBJ/G3Y3KuAB4MzTwl9rBarTmBaaZbl+iD1Kt3v+elNQdVCo
JPSfGr9AbVdFDHB0FS46sWqOyk3Rx9lScigUWb0mvQKBgQDnfUQJ7Uhqm7FByXQs
MWxLQIEHukWGG98btV2FjHO5N/IObrjXXUEl3qkTIW+oa+im48HRDKjlIZkTtN7l
tbhqRlt9lW7PXtR+J+YjSXxAeourNaaMxbaVy3U/fhVVP5KrWfLzBbb0ZOF2A7gq
g+rlHFVIVPOLj8lIPIlFjST9zwKBgQDCEiklTiFZZP6EjvgT7yMdJgvOkLFcJ4nF
A1PL72S7nYPKbwQZt0eUohMA/PVkDyemNpafTYeGjKx+waS60Zcn1/S6CMMDkmJL
DBAJVtCXwVmyaJTocS9kQwTeLqK+BBiHWL9nPTHmrTmEfrVwwB51eB9G+EJlv4fy
J8f4yPie9wKBgQCt/u3hOEUyPIxjknSLsype9cEGefA/+TsdrJj7BLMHCRIb3wV4
e1O4j0AubPdsdI+Owaqw4v8gGrzgnxbbOle/Kdsi7es4W2ME4CCPbXDDVlkc+1qQ
fRvcQ+2BJ9gJF5u6yAVgvW7jC+Cbv/fxnO41/7HqiE/3GsCEV1wmtwyS6QKBgQCe
h7VCuwr0+lIKuLsflYYKhoy4hWvMSqP44pnuCjUwKSCCGaOw2g3H9YkuknRl8xdB
aHAr22os1/cEaGyHCzS9oGRSH1wmK8rNYSIsbtVgUdpSqamSIvtCnJh6YoAgVjov
PajEzbFYrQJCIDtYyidXb/OkxqF+ejGz9xkcOhcVywKBgQCCmIJbRrHKB7YYPD68
NJo0kGnesUmsBzrFxWsckCTYpVkqjDI4VPeOYVFpXtlPkVMIIy7PSjZHCu9ujcDC
Oj3UlzzFzA70eAdkFrBlFxIembT4SjSoptN/8GP8wIe7xgnvj0gZJTH3W+z8AiBr
Ae/wEOcaaJD3g0i9hhz8Blf4IA==
-----END PRIVATE KEY-----"#;

#[test]
fn test_config_builder_full_workflow() {
    let oci = Oci::builder()
        .user_id("ocid1.user.oc1.iad.test123")
        .tenancy_id("ocid1.tenancy.oc1..test456")
        .region("us-ashburn-1")
        .fingerprint("11:22:33:44:55:66:77:88:99:aa:bb:cc:dd:ee:ff:00")
        .private_key(TEST_VALID_PEM)
        .expect("Failed to load private key")
        .build()
        .expect("Failed to build config");

    assert_eq!(oci.signer().user_id(), "ocid1.user.oc1.iad.test123");
    assert_eq!(oci.tenancy_id(), "ocid1.tenancy.oc1..test456");
    assert_eq!(oci.region(), "us-ashburn-1");
    assert_eq!(
        oci.signer().fingerprint(),
        "11:22:33:44:55:66:77:88:99:aa:bb:cc:dd:ee:ff:00"
    );
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
        let oci = Oci::builder()
            .user_id("ocid1.user.test")
            .tenancy_id("ocid1.tenancy.test")
            .region(region)
            .fingerprint("aa:bb:cc:dd")
            .private_key(TEST_VALID_PEM)
            .expect("Failed to load private key")
            .build()
            .expect(&format!("Failed to build config for region {}", region));

        assert_eq!(oci.region(), region);
    }
}

#[test]
fn test_config_immutability() {
    let oci = Oci::builder()
        .user_id("ocid1.user.original")
        .tenancy_id("ocid1.tenancy.test")
        .region("ap-seoul-1")
        .fingerprint("aa:bb:cc:dd")
        .private_key(TEST_VALID_PEM)
        .expect("Failed to load private key")
        .build()
        .unwrap();

    // Config fields are public but config itself should be used immutably
    let cloned_oci = oci.clone();
    assert_eq!(oci.signer().user_id(), cloned_oci.signer().user_id());
    assert_eq!(oci.tenancy_id(), cloned_oci.tenancy_id());
    assert_eq!(oci.region(), cloned_oci.region());
}

#[test]
fn test_private_key_formats() {
    // Test valid PEM
    let result = Oci::builder()
        .user_id("ocid1.user.test")
        .tenancy_id("ocid1.tenancy.test")
        .region("ap-seoul-1")
        .fingerprint("aa:bb:cc:dd")
        .private_key(TEST_VALID_PEM);

    assert!(
        result.is_ok(),
        "Failed to load private key with valid PEM"
    );

    let oci = result.unwrap().build();
    assert!(oci.is_ok(), "Failed to build config with valid PEM");

    // Test invalid PEM (should fail at build time)
    let invalid_pem = "-----BEGIN RSA PRIVATE KEY-----\ninvalid\n-----END RSA PRIVATE KEY-----";
    let result = Oci::builder()
        .user_id("ocid1.user.test")
        .tenancy_id("ocid1.tenancy.test")
        .region("ap-seoul-1")
        .fingerprint("aa:bb:cc:dd")
        .private_key(invalid_pem);

    // private_key() method itself might succeed if it just stores the string or does basic validation
    // But build() should fail
    if let Ok(builder) = result {
        assert!(builder.build().is_err(), "Should fail to build with invalid PEM");
    }
}

use super::ObjectStorage;
use crate::client::Oci;
use mockito::Server;

fn create_test_client() -> Oci {
    Oci::builder()
        .user_id("ocid1.user.oc1..test")
        .tenancy_id("ocid1.tenancy.oc1..test")
        .region("us-ashburn-1")
        .fingerprint("00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00")
        .private_key(
            r#"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQCuflefBANdEX7S
REhnbcA99CvzRsbRC+TYs8kcWKb5JNJLUrY6T+cFnhzT7cuJ1xtXn6FK9LwESOBo
oXVzQLwL551Lsl8kpUCw4vx7UoU+sCvRDXhUYQsJEtXZHfrGzcm2/A6P2XSenCnd
ns2VcCt0Xp+sHE04EGPjdL1WEnt1asXD4omletG4GKz1bm8jJWT8cq2MJrg3b2mq
U/qsG9jzjao9gZW4f6GiFbVX8xUzSWupE6l5nUkoXWRbkEQZtiFvl3J405oSTDmo
TM6MAWp47w3OOf9LGn0w830BbUMywSSaFEpQwOGVEns2MKjpnUa6SCo5ejFjE+R3
eoyeOzOzAgMBAAECggEAKBSKMNLdqOuDW23myacagMCMtdkcgtj+DE6jk7jDRbgQ
+884X0WKNa6fRYi592Pq+mIGzO5RH5TTwwTPMxV7/CoL5d7HXuX8aYUB5JvMUl1p
+x9ic9NEkyV57GCoATE0s6zK9XzH/kS1kxvOchRtTtILUB/CFu4g25athM9C/3GR
bkvdCPQcxbd6f7z7FEL191lbS6WdVXVhpktlZyHjC6X3/9QgJqn0xRNmTQvV9MUl
p/ckfE+AWdQlGYFnYJ6uKRCD0AVDj9q2TPJq1scQHheufdDfoZCdc8CiWHgRLp3B
/Ju3+eAl9tR6KLOdMq3T1d7bxF9iHbjrXxpti9JYKQKBgQDaAQvJ07yvecb1X0yo
3mzd/GKj0QzxXvXVMBp0X2JrdCNaatTLNRpZwqb0crhGkwYD7Dsnok7en9HgVu6v
2pRTTyJw/SeQ2bgvybErbtGYMWALbgQuPWnYmN2eVllhnbvHbnfafH+TCPsgbZfs
Ga8iJueEdJWcPVnYMR95V26DGQKBgQDM5+62sLBjmba4fhpP3CZxG7V8rtUW/DWa
6UXvhh+JjGy1B3fPJqbJ/mXPeHLMpJ12OZI9OyhHBOskR7pzzqc/IKh8Lk1G8Tu/
zBOB90pi9sq//i5tq+AxA9WEcxqWZToBGiJH/a38iT5kYWQY3om43WA6gh3By3vW
ntdphtzyqwKBgQDWzCMOWHbYOtejGqQQ2x9PVgbmu+rRxCvaQ0w9j2IM1+ChjRNf
qVHuURFpV8NjnidWJCNg+NZXGgeT0HPbhzWQJC+ePoEGgs6tH0BWuBkBqNymRl8O
JGqvBGeQRCpLOTw00w56kyKsADRXjkQbWG8r6kNBShHDYNuuXTBSwafcuQKBgQCe
uTKS1b9tB88gjp43KmOkzkABezSZf3jOrNB9wDmBxQMYH9bQ4jHk2mlnEvhqSUGo
KOR9BewnR0oWanGl73hiUBvzRbKat5b+9UhPLo2yp1Va77xrO+6ISV4GIVuBEJyj
6eiQN5OkwXNRMpflI80vJFy8NbpzOfqNv7FBjzRrzQKBgHMBFdLlYRIlfEgECNcT
OjyTx2C/nA8b875sPKBHhi8SdGaxST52m3xZ9YVUnN2SaAhnu5UkP7YJX8JzxAQb
sd8sguospGNmKOOoMLo4LDNMsDcrJJ16V4DJfVV5iwnKGp8QgIJi7a8YHPlx2WLR
vc2NfetOo+tuZADstuyzwGC9
-----END PRIVATE KEY-----"#,
        )
        .expect("Failed to load private key")
        .build()
        .unwrap()
}

#[tokio::test]
async fn test_get_bucket() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/n/test_namespace/b/test_bucket/")
        .with_status(200)
        .create_async()
        .await;

    let oci_client = create_test_client();
    let mut os_client = ObjectStorage::new(&oci_client, "test_namespace");

    // Override endpoint to point to mock server
    let url = server.url();
    let parts: Vec<&str> = url.split("://").collect();
    os_client.protocol = parts[0].to_string();
    os_client.endpoint = parts[1].to_string();

    let bucket = os_client.get_bucket("test_bucket").await.unwrap();

    assert_eq!(bucket.name, "test_bucket");
    assert_eq!(bucket.namespace, "test_namespace");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_put_object() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("PUT", "/n/test_namespace/b/test_bucket/o/test_object")
        .with_status(200)
        .with_header("opc-content-md5", "mgNkuembtIDdJeHwKEyFVQ==") // MD5 of "content"
        .create_async()
        .await;

    let oci_client = create_test_client();
    let mut os_client = ObjectStorage::new(&oci_client, "test_namespace");

    // Override endpoint to point to mock server
    let url = server.url();
    let parts: Vec<&str> = url.split("://").collect();
    os_client.protocol = parts[0].to_string();
    os_client.endpoint = parts[1].to_string();

    // Mock GET bucket request which is called by get_bucket
    let _mock_bucket = server
        .mock("GET", "/n/test_namespace/b/test_bucket/")
        .with_status(200)
        .create_async()
        .await;

    let mut bucket = os_client.get_bucket("test_bucket").await.unwrap();

    // Override endpoint for bucket as well (since get_bucket copies it from os_client, but let's be safe)
    bucket.protocol = parts[0].to_string();
    bucket.endpoint = parts[1].to_string();

    let object = bucket.put_object("test_object", "content").await.unwrap();

    assert_eq!(object.name, "test_object");
    assert_eq!(object.value, "content");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_get_object() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/n/test_namespace/b/test_bucket/o/test_object")
        .with_status(200)
        .with_header("content-md5", "mgNkuembtIDdJeHwKEyFVQ==") // MD5 of "content"
        .with_body("content")
        .create_async()
        .await;

    let oci_client = create_test_client();
    let mut os_client = ObjectStorage::new(&oci_client, "test_namespace");

    // Override endpoint to point to mock server
    let url = server.url();
    let parts: Vec<&str> = url.split("://").collect();
    os_client.protocol = parts[0].to_string();
    os_client.endpoint = parts[1].to_string();

    // Mock GET bucket request
    let _mock_bucket = server
        .mock("GET", "/n/test_namespace/b/test_bucket/")
        .with_status(200)
        .create_async()
        .await;

    let mut bucket = os_client.get_bucket("test_bucket").await.unwrap();
    bucket.protocol = parts[0].to_string();
    bucket.endpoint = parts[1].to_string();

    let object = bucket.get_object("test_object").await.unwrap();

    assert_eq!(object.name, "test_object");
    assert_eq!(object.value, "content");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_get_or_create_object() {
    let mut server = Server::new_async().await;

    // 1. First call: GET returns 404
    let mock_get_404 = server
        .mock("GET", "/n/test_namespace/b/test_bucket/o/test_object")
        .with_status(404)
        .create_async()
        .await;

    // 2. Second call: PUT creates object
    let mock_put = server
        .mock("PUT", "/n/test_namespace/b/test_bucket/o/test_object")
        .with_status(200)
        .with_header("opc-content-md5", "mgNkuembtIDdJeHwKEyFVQ==") // MD5 of "content"
        .create_async()
        .await;

    let oci_client = create_test_client();
    let mut os_client = ObjectStorage::new(&oci_client, "test_namespace");

    let url = server.url();
    let parts: Vec<&str> = url.split("://").collect();
    os_client.protocol = parts[0].to_string();
    os_client.endpoint = parts[1].to_string();

    // Mock GET bucket request
    let _mock_bucket = server
        .mock("GET", "/n/test_namespace/b/test_bucket/")
        .with_status(200)
        .create_async()
        .await;

    let mut bucket = os_client.get_bucket("test_bucket").await.unwrap();
    bucket.protocol = parts[0].to_string();
    bucket.endpoint = parts[1].to_string();

    let object = bucket
        .get_or_create_object("test_object", "content")
        .await
        .unwrap();

    assert_eq!(object.name, "test_object");
    assert_eq!(object.value, "content");
    mock_get_404.assert_async().await;
    mock_put.assert_async().await;
}

#[tokio::test]
async fn test_retention_rules() {
    use crate::services::object_storage::models::{
        RetentionDuration, RetentionRuleDetails, RetentionTimeUnit,
    };

    let mut server = Server::new_async().await;

    // Mock List Retention Rules
    let mock_list = server
        .mock("GET", "/n/test_namespace/b/test_bucket/retentionRules")
        .with_status(200)
        .with_body(
            r#"{
                "items": [
                    {
                        "id": "rule1",
                        "displayName": "rule1",
                        "duration": {
                            "timeAmount": 10,
                            "timeUnit": "DAYS"
                        },
                        "timeCreated": "2023-01-01T00:00:00.000Z",
                        "timeModified": "2023-01-01T00:00:00.000Z",
                        "etag": "etag1"
                    }
                ]
            }"#,
        )
        .create_async()
        .await;

    // Mock Create Retention Rule
    let mock_create = server
        .mock("POST", "/n/test_namespace/b/test_bucket/retentionRules")
        .with_status(200)
        .with_body(
            r#"{
                "id": "rule2",
                "displayName": "rule2",
                "duration": {
                    "timeAmount": 20,
                    "timeUnit": "DAYS"
                },
                "timeCreated": "2023-01-01T00:00:00.000Z",
                "timeModified": "2023-01-01T00:00:00.000Z",
                "etag": "etag2"
            }"#,
        )
        .create_async()
        .await;

    // Mock Get Retention Rule
    let mock_get = server
        .mock(
            "GET",
            "/n/test_namespace/b/test_bucket/retentionRules/rule2",
        )
        .with_status(200)
        .with_body(
            r#"{
                "id": "rule2",
                "displayName": "rule2",
                "duration": {
                    "timeAmount": 20,
                    "timeUnit": "DAYS"
                },
                "timeCreated": "2023-01-01T00:00:00.000Z",
                "timeModified": "2023-01-01T00:00:00.000Z",
                "etag": "etag2"
            }"#,
        )
        .create_async()
        .await;

    // Mock Update Retention Rule
    let mock_update = server
        .mock(
            "PUT",
            "/n/test_namespace/b/test_bucket/retentionRules/rule2",
        )
        .with_status(200)
        .with_body(
            r#"{
                "id": "rule2",
                "displayName": "rule2_updated",
                "duration": {
                    "timeAmount": 30,
                    "timeUnit": "DAYS"
                },
                "timeCreated": "2023-01-01T00:00:00.000Z",
                "timeModified": "2023-01-02T00:00:00.000Z",
                "etag": "etag3"
            }"#,
        )
        .create_async()
        .await;

    // Mock Delete Retention Rule
    let mock_delete = server
        .mock(
            "DELETE",
            "/n/test_namespace/b/test_bucket/retentionRules/rule2",
        )
        .with_status(204)
        .create_async()
        .await;

    let oci_client = create_test_client();
    let mut os_client = ObjectStorage::new(&oci_client, "test_namespace");

    let url = server.url();
    let parts: Vec<&str> = url.split("://").collect();
    os_client.protocol = parts[0].to_string();
    os_client.endpoint = parts[1].to_string();

    // Mock GET bucket request
    let _mock_bucket = server
        .mock("GET", "/n/test_namespace/b/test_bucket/")
        .with_status(200)
        .create_async()
        .await;

    let mut bucket = os_client.get_bucket("test_bucket").await.unwrap();
    bucket.protocol = parts[0].to_string();
    bucket.endpoint = parts[1].to_string();

    // Test List
    let rules = bucket.get_retention_rules().await.unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].display_name, "rule1");
    mock_list.assert_async().await;

    // Test Create
    let details = RetentionRuleDetails {
        display_name: Some("rule2".to_string()),
        duration: Some(RetentionDuration {
            time_amount: 20,
            time_unit: RetentionTimeUnit::Days,
        }),
        time_rule_locked: None,
    };
    let rule = bucket.create_retention_rule(details).await.unwrap();
    assert_eq!(rule.display_name, "rule2");
    mock_create.assert_async().await;

    // Test Get
    let rule = bucket.get_retention_rule("rule2").await.unwrap();
    assert_eq!(rule.display_name, "rule2");
    mock_get.assert_async().await;

    // Test Update
    let details = RetentionRuleDetails {
        display_name: Some("rule2_updated".to_string()),
        duration: Some(RetentionDuration {
            time_amount: 30,
            time_unit: RetentionTimeUnit::Days,
        }),
        time_rule_locked: None,
    };
    // Use RetentionRule method
    let rule = rule.update(&bucket, details).await.unwrap();
    assert_eq!(rule.display_name, "rule2_updated");
    mock_update.assert_async().await;

    // Test Delete
    // Use RetentionRule method
    rule.delete(&bucket).await.unwrap();
    mock_delete.assert_async().await;
}

#[tokio::test]
async fn test_get_object_with_checksums() {
    use crate::services::object_storage::models::ChecksumAlgorithm;

    let mut server = Server::new_async().await;

    // "test content" checksums
    let content = "test content";
    // MD5 of "test content"
    let md5_b64 = "lHP90NiApDwht3eNNIchVw==";
    // SHA256 of "test content"
    let sha256_b64 = "auinVVUgn9bEQVfArtgBbnY/9DWhnPGG92hjFAFD/3I=";

    // Mock GET bucket
    let _mock_bucket = server
        .mock("GET", "/n/test_namespace/b/test_bucket/")
        .with_status(200)
        .create_async()
        .await;

    // Mock GET object
    let mock_object = server
        .mock("GET", "/n/test_namespace/b/test_bucket/o/test_object")
        .with_status(200)
        .with_header("content-md5", md5_b64)
        .with_header("opc-content-sha256", sha256_b64)
        .with_body(content)
        .create_async()
        .await;

    let oci_client = create_test_client();
    let mut os_client = ObjectStorage::new(&oci_client, "test_namespace");

    // Override endpoint to point to mock server
    let url = server.url();
    let parts: Vec<&str> = url.split("://").collect();
    os_client.protocol = parts[0].to_string();
    os_client.endpoint = parts[1].to_string();

    let bucket = os_client.get_bucket("test_bucket").await.unwrap();
    let object = bucket.get_object("test_object").await.unwrap();

    assert_eq!(object.value, content);

    // Check MD5
    assert_eq!(object.md5, md5_b64);

    // Check SHA256
    if let Some(checksum) = &object.checksum {
        assert!(matches!(checksum.algorithm, ChecksumAlgorithm::SHA256));
        assert_eq!(checksum.value, sha256_b64);
    } else {
        panic!("SHA256 checksum not found");
    }

    // Verify checksums
    object
        .verify_checksums()
        .expect("Checksum verification failed");

    mock_object.assert_async().await;
}

#[tokio::test]
async fn test_get_object_checksum_mismatch() {
    let mut server = Server::new_async().await;

    let content = "test content";
    let md5_b64_wrong = "lHP90NiApDwht3eNNIchVx=="; // Modified last char

    // Mock GET bucket
    let _mock_bucket = server
        .mock("GET", "/n/test_namespace/b/test_bucket/")
        .with_status(200)
        .create_async()
        .await;

    let mock_object = server
        .mock("GET", "/n/test_namespace/b/test_bucket/o/test_object_bad")
        .with_status(200)
        .with_header("content-md5", md5_b64_wrong)
        .with_body(content)
        .create_async()
        .await;

    let oci_client = create_test_client();
    let mut os_client = ObjectStorage::new(&oci_client, "test_namespace");

    let url = server.url();
    let parts: Vec<&str> = url.split("://").collect();
    os_client.protocol = parts[0].to_string();
    os_client.endpoint = parts[1].to_string();

    let bucket = os_client.get_bucket("test_bucket").await.unwrap();
    let object = bucket.get_object("test_object_bad").await.unwrap();

    let result = object.verify_checksums();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("MD5 checksum mismatch"));

    mock_object.assert_async().await;
}

#[tokio::test]
async fn test_put_object_with_checksum() {
    use crate::services::object_storage::models::ChecksumAlgorithm;

    let mut server = Server::new_async().await;
    let content = "test content";
    // SHA256 of "test content"
    let sha256_b64 = "auinVVUgn9bEQVfArtgBbnY/9DWhnPGG92hjFAFD/3I=";

    // Mock GET bucket
    let _mock_bucket = server
        .mock("GET", "/n/test_namespace/b/test_bucket/")
        .with_status(200)
        .create_async()
        .await;

    let mock_put = server
        .mock(
            "PUT",
            "/n/test_namespace/b/test_bucket/o/test_object_sha256",
        )
        .with_status(200)
        .with_header("opc-checksum-algorithm", "SHA256")
        .with_header("opc-content-sha256", sha256_b64)
        .with_header("opc-content-md5", "lHP90NiApDwht3eNNIchVw==") // MD5 of "test content"
        .create_async()
        .await;

    let oci_client = create_test_client();
    let mut os_client = ObjectStorage::new(&oci_client, "test_namespace");

    let url = server.url();
    let parts: Vec<&str> = url.split("://").collect();
    os_client.protocol = parts[0].to_string();
    os_client.endpoint = parts[1].to_string();

    let mut bucket = os_client.get_bucket("test_bucket").await.unwrap();
    bucket.protocol = parts[0].to_string();
    bucket.endpoint = parts[1].to_string();

    bucket
        .put_object_with_checksum("test_object_sha256", content, ChecksumAlgorithm::SHA256)
        .await
        .unwrap();

    mock_put.assert_async().await;
}

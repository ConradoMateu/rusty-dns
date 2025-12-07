//! Provider tests with HTTP mocking.

#[cfg(test)]
mod namecheap_tests {
    use crate::providers::{DdnsProvider, NamecheapProvider};
    use std::net::IpAddr;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_namecheap_update_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/update"))
            .and(query_param("host", "vpn"))
            .and(query_param("domain", "example.com"))
            .and(query_param("password", "secret123"))
            .and(query_param("ip", "1.2.3.4"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"<?xml version="1.0"?>
                <interface-response>
                    <Command>SETDNSHOST</Command>
                    <IP>1.2.3.4</IP>
                    <ErrCount>0</ErrCount>
                    <Done>true</Done>
                </interface-response>"#,
            ))
            .mount(&mock_server)
            .await;

        let provider = NamecheapProvider::with_base_url(
            "example.com".to_string(),
            "vpn".to_string(),
            "secret123".to_string(),
            mock_server.uri(),
        );

        let ip: IpAddr = "1.2.3.4".parse().unwrap();
        let result = provider.update_ip(ip).await.unwrap();

        assert!(result.success);
        assert_eq!(result.ip, Some(ip));
        assert_eq!(result.domain, "vpn.example.com");
    }

    #[tokio::test]
    async fn test_namecheap_update_password_mismatch() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/update"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"<?xml version="1.0"?>
                <interface-response>
                    <ErrCount>1</ErrCount>
                    <Err1>Passwords do not match</Err1>
                </interface-response>"#,
            ))
            .mount(&mock_server)
            .await;

        let provider = NamecheapProvider::with_base_url(
            "example.com".to_string(),
            "vpn".to_string(),
            "wrong".to_string(),
            mock_server.uri(),
        );

        let ip: IpAddr = "1.2.3.4".parse().unwrap();
        let result = provider.update_ip(ip).await.unwrap();

        assert!(!result.success);
        assert_eq!(result.error, Some("Passwords do not match".to_string()));
    }

    #[tokio::test]
    async fn test_namecheap_root_domain() {
        let provider = NamecheapProvider::new(
            "example.com".to_string(),
            "@".to_string(),
            "secret".to_string(),
        );
        assert_eq!(provider.domain(), "example.com");
    }

    #[tokio::test]
    async fn test_namecheap_subdomain() {
        let provider = NamecheapProvider::new(
            "example.com".to_string(),
            "vpn".to_string(),
            "secret".to_string(),
        );
        assert_eq!(provider.domain(), "vpn.example.com");
    }
}

#[cfg(test)]
mod duckdns_tests {
    use crate::providers::{DdnsProvider, DuckDnsProvider};
    use std::net::IpAddr;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_duckdns_update_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/update"))
            .and(query_param("domains", "mysubdomain"))
            .and(query_param("token", "mytoken"))
            .and(query_param("ip", "5.6.7.8"))
            .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
            .mount(&mock_server)
            .await;

        let provider = DuckDnsProvider::with_base_url(
            "mysubdomain".to_string(),
            "mytoken".to_string(),
            mock_server.uri(),
        );

        let ip: IpAddr = "5.6.7.8".parse().unwrap();
        let result = provider.update_ip(ip).await.unwrap();

        assert!(result.success);
        assert_eq!(result.ip, Some(ip));
    }

    #[tokio::test]
    async fn test_duckdns_update_failure() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/update"))
            .respond_with(ResponseTemplate::new(200).set_body_string("KO"))
            .mount(&mock_server)
            .await;

        let provider = DuckDnsProvider::with_base_url(
            "mysubdomain".to_string(),
            "badtoken".to_string(),
            mock_server.uri(),
        );

        let ip: IpAddr = "5.6.7.8".parse().unwrap();
        let result = provider.update_ip(ip).await.unwrap();

        assert!(!result.success);
    }

    #[tokio::test]
    async fn test_duckdns_domain_format() {
        let provider = DuckDnsProvider::new("test".to_string(), "token".to_string());
        assert_eq!(provider.domain(), "test.duckdns.org");
    }

    #[tokio::test]
    async fn test_duckdns_validate_empty_token() {
        let provider = DuckDnsProvider::new("test".to_string(), "".to_string());
        assert!(provider.validate().await.is_err());
    }

    #[tokio::test]
    async fn test_duckdns_validate_empty_domains() {
        let provider = DuckDnsProvider::new("".to_string(), "token".to_string());
        assert!(provider.validate().await.is_err());
    }
}

#[cfg(test)]
mod cloudflare_tests {
    use crate::providers::{CloudflareProvider, DdnsProvider};
    use std::net::IpAddr;
    use wiremock::matchers::{header, method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_cloudflare_update_success() {
        let mock_server = MockServer::start().await;

        let get_response =
            r#"{"success":true,"result":[{"id":"record-123","content":"1.1.1.1"}],"errors":[]}"#;
        let patch_response =
            r#"{"success":true,"result":{"id":"record-123","content":"2.2.2.2"},"errors":[]}"#;

        // Mock GET to find record ID
        Mock::given(method("GET"))
            .and(path_regex(r"/client/v4/zones/.*/dns_records.*"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_string(get_response))
            .expect(2)
            .mount(&mock_server)
            .await;

        // Mock PATCH to update record
        Mock::given(method("PATCH"))
            .and(path_regex(r"/client/v4/zones/.*/dns_records/.*"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_string(patch_response))
            .mount(&mock_server)
            .await;

        let provider = CloudflareProvider::with_base_url(
            "test-token".to_string(),
            "zone-123".to_string(),
            "vpn.example.com".to_string(),
            false,
            mock_server.uri(),
        );

        let ip: IpAddr = "2.2.2.2".parse().unwrap();
        let result = provider.update_ip(ip).await.unwrap();

        assert!(result.success);
        assert_eq!(result.ip, Some(ip));
    }

    #[tokio::test]
    async fn test_cloudflare_record_not_found() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path_regex(r"/client/v4/zones/.*/dns_records.*"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(r#"{"success":true,"result":[],"errors":[]}"#),
            )
            .mount(&mock_server)
            .await;

        let provider = CloudflareProvider::with_base_url(
            "test-token".to_string(),
            "zone-123".to_string(),
            "nonexistent.example.com".to_string(),
            false,
            mock_server.uri(),
        );

        let ip: IpAddr = "2.2.2.2".parse().unwrap();
        let result = provider.update_ip(ip).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cloudflare_auth_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path_regex(r"/client/v4/zones/.*/dns_records.*"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"success":false,"result":null,"errors":[{"message":"Invalid API token"}]}"#,
            ))
            .mount(&mock_server)
            .await;

        let provider = CloudflareProvider::with_base_url(
            "bad-token".to_string(),
            "zone-123".to_string(),
            "vpn.example.com".to_string(),
            false,
            mock_server.uri(),
        );

        let ip: IpAddr = "2.2.2.2".parse().unwrap();
        let result = provider.update_ip(ip).await;

        assert!(result.is_err());
    }
}

#[cfg(test)]
mod godaddy_tests {
    use crate::providers::{DdnsProvider, GoDaddyProvider};
    use std::net::IpAddr;
    use wiremock::matchers::{header, method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_godaddy_update_success() {
        let mock_server = MockServer::start().await;

        // Mock GET current IP
        Mock::given(method("GET"))
            .and(path_regex(r"/v1/domains/.*/records/A/.*"))
            .and(header("Authorization", "sso-key api-key:api-secret"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!([{"data": "1.1.1.1"}])),
            )
            .mount(&mock_server)
            .await;

        // Mock PUT update
        Mock::given(method("PUT"))
            .and(path_regex(r"/v1/domains/.*/records/A/.*"))
            .and(header("Authorization", "sso-key api-key:api-secret"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let provider = GoDaddyProvider::with_base_url(
            "api-key".to_string(),
            "api-secret".to_string(),
            "example.com".to_string(),
            "vpn".to_string(),
            600,
            mock_server.uri(),
        );

        let ip: IpAddr = "3.3.3.3".parse().unwrap();
        let result = provider.update_ip(ip).await.unwrap();

        assert!(result.success);
        assert_eq!(result.ip, Some(ip));
    }

    #[tokio::test]
    async fn test_godaddy_domain_format() {
        let provider = GoDaddyProvider::new(
            "key".to_string(),
            "secret".to_string(),
            "example.com".to_string(),
            "vpn".to_string(),
            600,
        );
        assert_eq!(provider.domain(), "vpn.example.com");
    }

    #[tokio::test]
    async fn test_godaddy_root_domain() {
        let provider = GoDaddyProvider::new(
            "key".to_string(),
            "secret".to_string(),
            "example.com".to_string(),
            "@".to_string(),
            600,
        );
        assert_eq!(provider.domain(), "example.com");
    }
}

#[cfg(test)]
mod env_resolution_tests {
    use crate::providers::resolve_env;

    #[test]
    fn test_resolve_env_with_value() {
        assert_eq!(resolve_env("plain_value"), "plain_value");
    }

    #[test]
    fn test_resolve_env_with_existing_var() {
        std::env::set_var("TEST_RUSTY_DNS_VAR", "resolved_value");
        assert_eq!(resolve_env("$TEST_RUSTY_DNS_VAR"), "resolved_value");
        std::env::remove_var("TEST_RUSTY_DNS_VAR");
    }

    #[test]
    fn test_resolve_env_with_missing_var() {
        let result = resolve_env("$NONEXISTENT_VAR_12345");
        assert_eq!(result, "$NONEXISTENT_VAR_12345");
    }
}

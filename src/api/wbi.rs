//! WBI Signature implementation for Bilibili API
//! See: https://socialsisteryi.github.io/bilibili-API-collect/docs/misc/sign/wbi.html

use std::time::{SystemTime, UNIX_EPOCH};

const MIXIN_KEY_ENC_TAB: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29,
    28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25,
    54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52,
];

/// Generate mixin key from img_key and sub_key
pub fn get_mixin_key(img_key: &str, sub_key: &str) -> String {
    let orig = format!("{}{}", img_key, sub_key);
    let orig_bytes = orig.as_bytes();
    MIXIN_KEY_ENC_TAB
        .iter()
        .take(32)
        .map(|&i| orig_bytes[i] as char)
        .collect::<String>()
}

/// URL encode a string (RFC 3986 compliant)
fn url_encode(s: &str) -> String {
    s.chars()
        .filter_map(|c| {
            if c.is_ascii_alphanumeric() || "-_.~".contains(c) {
                Some(c.to_string())
            } else if "!'()*".contains(c) {
                // Filter these characters
                None
            } else {
                let encoded = c
                    .encode_utf8(&mut [0; 4])
                    .bytes()
                    .fold(String::new(), |acc, b| acc + &format!("%{:02X}", b));
                Some(encoded)
            }
        })
        .collect::<String>()
}

/// Sign request parameters with WBI
/// Returns the signed query string (includes w_rid and wts)
pub fn encode_wbi(params: Vec<(&str, String)>, img_key: &str, sub_key: &str) -> String {
    let cur_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    encode_wbi_with_timestamp(params, img_key, sub_key, cur_time)
}

/// Sign with a specific timestamp (for testing)
pub fn encode_wbi_with_timestamp(
    mut params: Vec<(&str, String)>,
    img_key: &str,
    sub_key: &str,
    timestamp: u64,
) -> String {
    let mixin_key = get_mixin_key(img_key, sub_key);

    // Add timestamp
    params.push(("wts", timestamp.to_string()));

    // Sort by key
    params.sort_by(|a, b| a.0.cmp(b.0));

    // Build query string
    let query = params
        .iter()
        .map(|(k, v)| format!("{}={}", url_encode(k), url_encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    // Calculate w_rid
    let w_rid = format!("{:x}", md5::compute(format!("{}{}", query, mixin_key)));

    // Return final query
    format!("{}&w_rid={}", query, w_rid)
}

/// Extract key from WBI image URL
pub fn extract_key_from_url(url: &str) -> Option<String> {
    url.rsplit_once('/')
        .and_then(|(_, s)| s.rsplit_once('.'))
        .map(|(s, _)| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_mixin_key() {
        let img_key = "7cd084941338484aae1ad9425b84077c";
        let sub_key = "4932caff0ff746eab6f01bf08b70ac45";
        assert_eq!(
            get_mixin_key(img_key, sub_key),
            "ea1db124af3c7062474693fa704f4ff8"
        );
    }

    #[test]
    fn test_encode_wbi() {
        let params = vec![
            ("foo", String::from("114")),
            ("bar", String::from("514")),
            ("zab", String::from("1919810")),
        ];
        let result = encode_wbi_with_timestamp(
            params,
            "7cd084941338484aae1ad9425b84077c",
            "4932caff0ff746eab6f01bf08b70ac45",
            1702204169,
        );
        assert_eq!(
            result,
            "bar=514&foo=114&wts=1702204169&zab=1919810&w_rid=8f6f2b5b3d485fe1886cec6a0be8c5d4"
        );
    }

    #[test]
    fn test_extract_key_from_url() {
        let url = "https://i0.hdslb.com/bfs/wbi/7cd084941338484aae1ad9425b84077c.png";
        assert_eq!(
            extract_key_from_url(url),
            Some("7cd084941338484aae1ad9425b84077c".to_string())
        );
    }
}

use crate::http::Request;

/// Renders one captured request for terminal display: the request
/// line, every header, and the body (decoded as UTF-8 if valid,
/// otherwise a byte count) — everything a person debugging a webhook
/// payload actually wants to see.
pub fn format_request(req: &Request, timestamp: &str) -> String {
    let mut out = format!(
        "── {timestamp} ── {} {} {}\n",
        req.method, req.path, req.version
    );
    for (k, v) in &req.headers {
        out.push_str(&format!("  {k}: {v}\n"));
    }
    if !req.body.is_empty() {
        match std::str::from_utf8(&req.body) {
            Ok(text) => out.push_str(&format!("\n{text}\n")),
            Err(_) => out.push_str(&format!("\n<{} bytes of binary body>\n", req.body.len())),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(method: &str, path: &str, headers: &[(&str, &str)], body: &[u8]) -> Request {
        Request {
            method: method.to_string(),
            path: path.to_string(),
            version: "HTTP/1.1".to_string(),
            headers: headers
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            body: body.to_vec(),
        }
    }

    #[test]
    fn includes_method_path_and_version() {
        let out = format_request(&req("POST", "/hook", &[], b""), "12:00:00");
        assert!(out.contains("POST /hook HTTP/1.1"));
    }

    #[test]
    fn includes_every_header() {
        let out = format_request(&req("GET", "/", &[("X-A", "1"), ("X-B", "2")], b""), "t");
        assert!(out.contains("X-A: 1"));
        assert!(out.contains("X-B: 2"));
    }

    #[test]
    fn decodes_a_utf8_body_as_text() {
        let out = format_request(&req("POST", "/", &[], b"{\"ok\":true}"), "t");
        assert!(out.contains("{\"ok\":true}"));
    }

    #[test]
    fn non_utf8_body_shows_a_byte_count_instead_of_garbage() {
        let out = format_request(&req("POST", "/", &[], &[0xff, 0xfe, 0x00]), "t");
        assert!(out.contains("3 bytes of binary body"));
    }

    #[test]
    fn empty_body_adds_no_extra_blank_section() {
        let out = format_request(&req("GET", "/", &[], b""), "t");
        assert!(
            !out.contains("\n\n"),
            "no body means no blank-line body section, got: {out:?}"
        );
    }
}

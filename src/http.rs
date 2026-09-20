//! A minimal real HTTP/1.1 request parser — just enough of the spec to
//! read a request line, headers, and a `Content-Length`-delimited body
//! off a real socket. No framework: this tool only ever needs to
//! *receive* one request at a time and print it, never route or serve.

use std::io::{BufRead, Read};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

pub fn parse_request<R: BufRead + Read>(reader: &mut R) -> std::io::Result<Request> {
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let request_line = request_line.trim_end();
    let mut parts = request_line.splitn(3, ' ');
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("").to_string();
    let version = parts.next().unwrap_or("").to_string();

    let mut headers = Vec::new();
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        let line = line.trim_end_matches(['\r', '\n']);
        if n == 0 || line.is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            headers.push((k.trim().to_string(), v.trim().to_string()));
        }
    }

    let content_length: usize = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, v)| v.parse().ok())
        .unwrap_or(0);

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }

    Ok(Request {
        method,
        path,
        version,
        headers,
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    fn parse(raw: &str) -> Request {
        let mut reader = BufReader::new(raw.as_bytes());
        parse_request(&mut reader).unwrap()
    }

    #[test]
    fn parses_request_line() {
        let req = parse("GET /path?x=1 HTTP/1.1\r\nHost: localhost\r\n\r\n");
        assert_eq!(req.method, "GET");
        assert_eq!(req.path, "/path?x=1");
        assert_eq!(req.version, "HTTP/1.1");
    }

    #[test]
    fn parses_multiple_headers_in_order() {
        let req = parse("POST / HTTP/1.1\r\nHost: localhost\r\nX-Custom: abc\r\n\r\n");
        assert_eq!(
            req.headers,
            vec![
                ("Host".to_string(), "localhost".to_string()),
                ("X-Custom".to_string(), "abc".to_string())
            ]
        );
    }

    #[test]
    fn parses_body_by_content_length() {
        let req = parse("POST /webhook HTTP/1.1\r\nContent-Length: 13\r\n\r\n{\"ok\":true}\r\n");
        assert_eq!(&req.body[..11], b"{\"ok\":true}");
    }

    #[test]
    fn request_with_no_body_and_no_content_length_has_empty_body() {
        let req = parse("GET / HTTP/1.1\r\nHost: x\r\n\r\n");
        assert!(req.body.is_empty());
    }

    #[test]
    fn header_name_lookup_for_content_length_is_case_insensitive() {
        let req = parse("POST / HTTP/1.1\r\ncontent-length: 5\r\n\r\nhello");
        assert_eq!(req.body, b"hello");
    }

    #[test]
    fn malformed_header_line_with_no_colon_is_skipped() {
        let req = parse("GET / HTTP/1.1\r\nnot-a-header-line\r\nHost: x\r\n\r\n");
        assert_eq!(req.headers, vec![("Host".to_string(), "x".to_string())]);
    }
}

use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    pub max_request_bytes: usize,
    pub max_start_line_bytes: usize,
    pub max_header_count: usize,
    pub max_header_line_bytes: usize,
    pub max_body_bytes: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self { max_request_bytes: 64 * 1024, max_start_line_bytes: 2048, max_header_count: 64,
            max_header_line_bytes: 8192, max_body_bytes: 48 * 1024 }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Method { Get, Options, Announce, Setup, Record, Pause, Flush, Teardown, GetParameter, SetParameter, Other(String) }

impl Method {
    fn parse(s: &str) -> Self {
        match s { "GET" => Self::Get, "OPTIONS" => Self::Options, "ANNOUNCE" => Self::Announce, "SETUP" => Self::Setup,
            "RECORD" => Self::Record, "PAUSE" => Self::Pause, "FLUSH" => Self::Flush,
            "TEARDOWN" => Self::Teardown, "GET_PARAMETER" => Self::GetParameter,
            "SET_PARAMETER" => Self::SetParameter, other => Self::Other(other.to_owned()) }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Headers(Vec<(String, String)>);
impl Headers {
    pub fn get(&self, name: &str) -> Option<&str> { self.0.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str()) }
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> { self.0.iter().map(|(n, v)| (n.as_str(), v.as_str())) }
    pub fn values<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a str> + 'a { self.0.iter().filter(move |(n, _)| n.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str()) }
    pub fn len(&self) -> usize { self.0.len() }
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Request { pub method: Method, pub uri: String, pub version: String, pub headers: Headers, pub body: Vec<u8> }

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    RequestTooLarge, Incomplete, InvalidUtf8, StartLineTooLong, MalformedStartLine,
    UnsupportedVersion, TooManyHeaders, HeaderLineTooLong, MalformedHeader,
    DuplicateContentLength, InvalidContentLength, BodyTooLarge, IncompleteBody, TrailingData,
}
impl fmt::Display for ParseError { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "RTSP parse error: {self:?}") } }
impl std::error::Error for ParseError {}

pub fn parse_request(input: &[u8], limits: Limits) -> Result<Request, ParseError> {
    if input.len() > limits.max_request_bytes { return Err(ParseError::RequestTooLarge); }
    let end = input.windows(4).position(|w| w == b"\r\n\r\n").ok_or(ParseError::Incomplete)?;
    let head = &input[..end];
    let text = std::str::from_utf8(head).map_err(|_| ParseError::InvalidUtf8)?;
    let mut lines = text.split("\r\n");
    let start = lines.next().ok_or(ParseError::MalformedStartLine)?;
    if start.len() > limits.max_start_line_bytes { return Err(ParseError::StartLineTooLong); }
    let mut parts = start.split_whitespace();
    let method = parts.next().ok_or(ParseError::MalformedStartLine)?;
    let uri = parts.next().ok_or(ParseError::MalformedStartLine)?;
    let version = parts.next().ok_or(ParseError::MalformedStartLine)?;
    if parts.next().is_some() || method.is_empty() || uri.is_empty() { return Err(ParseError::MalformedStartLine); }
    if version != "RTSP/1.0" { return Err(ParseError::UnsupportedVersion); }
    let mut values = Vec::new();
    let mut content_length = None;
    for line in lines {
        if line.len() > limits.max_header_line_bytes { return Err(ParseError::HeaderLineTooLong); }
        if values.len() >= limits.max_header_count { return Err(ParseError::TooManyHeaders); }
        if line.starts_with([' ', '\t']) { return Err(ParseError::MalformedHeader); }
        let (name, value) = line.split_once(':').ok_or(ParseError::MalformedHeader)?;
        if name.is_empty() || !name.bytes().all(is_token) { return Err(ParseError::MalformedHeader); }
        let value = value.trim();
        if value.bytes().any(|b| b < 0x20 && b != b'\t') { return Err(ParseError::MalformedHeader); }
        if name.eq_ignore_ascii_case("content-length") {
            if content_length.is_some() { return Err(ParseError::DuplicateContentLength); }
            content_length = Some(value.parse::<usize>().map_err(|_| ParseError::InvalidContentLength)?);
        }
        values.push((name.to_owned(), value.to_owned()));
    }
    let length = content_length.unwrap_or(0);
    if length > limits.max_body_bytes { return Err(ParseError::BodyTooLarge); }
    let body_start = end + 4;
    let available = input.len() - body_start;
    if available < length { return Err(ParseError::IncompleteBody); }
    if available > length { return Err(ParseError::TrailingData); }
    Ok(Request { method: Method::parse(method), uri: uri.to_owned(), version: version.to_owned(), headers: Headers(values), body: input[body_start..].to_vec() })
}

fn is_token(b: u8) -> bool { b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn parses_bounded_request() { let r = parse_request(b"SET_PARAMETER * RTSP/1.0\r\nCSeq: 7\r\nContent-Length: 3\r\n\r\nabc", Limits::default()).unwrap(); assert_eq!(r.method, Method::SetParameter); assert_eq!(r.headers.get("cseq"), Some("7")); assert_eq!(r.body, b"abc"); }
    #[test] fn rejects_incomplete_body() { assert_eq!(parse_request(b"ANNOUNCE * RTSP/1.0\r\nContent-Length: 4\r\n\r\nx", Limits::default()), Err(ParseError::IncompleteBody)); }
    #[test] fn rejects_header_count() { let mut l = Limits::default(); l.max_header_count = 1; assert_eq!(parse_request(b"OPTIONS * RTSP/1.0\r\nA: 1\r\nB: 2\r\n\r\n", l), Err(ParseError::TooManyHeaders)); }
    #[test] fn rejects_duplicate_length() { assert_eq!(parse_request(b"OPTIONS * RTSP/1.0\r\nContent-Length: 0\r\ncontent-length: 0\r\n\r\n", Limits::default()), Err(ParseError::DuplicateContentLength)); }
}

#[cfg(test)]
mod additional_validation_tests {
 use super::*;
 #[test] fn rejects_content_length_overflow_and_trailing_data() {
  assert_eq!(parse_request(b"OPTIONS * RTSP/1.0\r\nContent-Length: 999999999999999999999999999\r\n\r\n",Limits::default()),Err(ParseError::InvalidContentLength));
  assert_eq!(parse_request(b"OPTIONS * RTSP/1.0\r\nContent-Length: 0\r\n\r\nx",Limits::default()),Err(ParseError::TrailingData));
 }
 #[test] fn rejects_body_over_limit() {
  let mut limits=Limits::default(); limits.max_body_bytes=1;
  assert_eq!(parse_request(b"OPTIONS * RTSP/1.0\r\nContent-Length: 2\r\n\r\nxx",limits),Err(ParseError::BodyTooLarge));
 }
}

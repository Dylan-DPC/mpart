pub use tiny_http::Request as TinyHttpRequest;

use super::HttpRequest;

use std::io::Read;

impl<'r> HttpRequest for &'r mut TinyHttpRequest {
    type Body = &'r mut dyn Read;

    fn multipart_boundary(&self) -> Option<&str> {
        const BOUNDARY: &str = "boundary=";

        let content_type = self
            .headers()
            .iter()
            .find(|header| header.field.equiv("Content-Type"))?
        .value
        .as_str();
        let start = content_type.find(BOUNDARY)? + BOUNDARY.len();
        let end = content_type[start..]
            .find(';')
            .map_or(content_type.len(), |end| start + end);

        Some(&content_type[start..end])
    }

    fn body(self) -> Self::Body {
        self.as_reader()
    }
}

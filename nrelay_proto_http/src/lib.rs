use httparse;

pub struct HttpSniffer {
    buffer: Vec<u8>,
    host: Option<String>,
}

impl HttpSniffer {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            host: None,
        }
    }

    pub fn feed(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    pub fn extract_host(&mut self) -> Option<String> {
        if self.host.is_some() {
            return self.host.clone();
        }

        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);

        match req.parse(&self.buffer) {
            Ok(httparse::Status::Complete(_)) => {
                for header in req.headers {
                    if header.name.eq_ignore_ascii_case("host") {
                        if let Ok(host) = std::str::from_utf8(header.value) {
                            self.host = Some(host.to_string());
                            return self.host.clone();
                        }
                    }
                }
            }
            _ => {}
        }

        None
    }

    pub fn consumed_bytes(&self) -> &[u8] {
        &self.buffer
    }
}

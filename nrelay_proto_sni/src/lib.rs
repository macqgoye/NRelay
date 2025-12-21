pub struct TlsSniSniffer {
    buffer: Vec<u8>,
    sni: Option<String>,
}

impl TlsSniSniffer {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            sni: None,
        }
    }

    pub fn feed(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    pub fn extract_sni(&mut self) -> Option<String> {
        if self.sni.is_some() {
            return self.sni.clone();
        }

        if self.buffer.len() < 43 {
            return None;
        }

        if self.buffer[0] != 0x16 {
            return None;
        }

        let mut pos = 43;

        if pos + 1 > self.buffer.len() {
            return None;
        }
        let session_id_len = self.buffer[pos] as usize;
        pos += 1 + session_id_len;

        if pos + 2 > self.buffer.len() {
            return None;
        }
        let cipher_suites_len =
            u16::from_be_bytes([self.buffer[pos], self.buffer[pos + 1]]) as usize;
        pos += 2 + cipher_suites_len;

        if pos + 1 > self.buffer.len() {
            return None;
        }
        let compression_len = self.buffer[pos] as usize;
        pos += 1 + compression_len;

        if pos + 2 > self.buffer.len() {
            return None;
        }
        let extensions_len = u16::from_be_bytes([self.buffer[pos], self.buffer[pos + 1]]) as usize;
        pos += 2;

        let extensions_end = pos + extensions_len;

        while pos + 4 <= extensions_end && pos + 4 <= self.buffer.len() {
            let ext_type = u16::from_be_bytes([self.buffer[pos], self.buffer[pos + 1]]);
            let ext_len = u16::from_be_bytes([self.buffer[pos + 2], self.buffer[pos + 3]]) as usize;
            pos += 4;

            if ext_type == 0 {
                if pos + 5 <= self.buffer.len() {
                    let list_len =
                        u16::from_be_bytes([self.buffer[pos], self.buffer[pos + 1]]) as usize;
                    pos += 2;

                    if pos + 3 <= self.buffer.len() {
                        let name_type = self.buffer[pos];
                        let name_len =
                            u16::from_be_bytes([self.buffer[pos + 1], self.buffer[pos + 2]])
                                as usize;
                        pos += 3;

                        if name_type == 0 && pos + name_len <= self.buffer.len() {
                            if let Ok(hostname) =
                                std::str::from_utf8(&self.buffer[pos..pos + name_len])
                            {
                                self.sni = Some(hostname.to_string());
                                return self.sni.clone();
                            }
                        }
                    }
                }
                break;
            }

            pos += ext_len;
        }

        None
    }

    pub fn consumed_bytes(&self) -> &[u8] {
        &self.buffer
    }
}

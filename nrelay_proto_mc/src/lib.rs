pub struct MinecraftSniffer {
    buffer: Vec<u8>,
    server_address: Option<String>,
}

impl MinecraftSniffer {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            server_address: None,
        }
    }

    pub fn feed(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    pub fn extract_server_address(&mut self) -> Option<String> {
        if self.server_address.is_some() {
            return self.server_address.clone();
        }

        if self.buffer.is_empty() {
            return None;
        }

        let (packet_len, mut pos) = read_varint(&self.buffer, 0)?;

        if self.buffer.len() < pos + packet_len {
            return None;
        }

        let (packet_id, new_pos) = read_varint(&self.buffer, pos)?;
        pos = new_pos;

        if packet_id != 0 {
            return None;
        }

        let (protocol_version, new_pos) = read_varint(&self.buffer, pos)?;
        pos = new_pos;

        let (addr_len, new_pos) = read_varint(&self.buffer, pos)?;
        pos = new_pos;

        if self.buffer.len() < pos + addr_len {
            return None;
        }

        if let Ok(address) = std::str::from_utf8(&self.buffer[pos..pos + addr_len]) {
            self.server_address = Some(address.to_string());
            return self.server_address.clone();
        }

        None
    }

    pub fn consumed_bytes(&self) -> &[u8] {
        &self.buffer
    }
}

fn read_varint(data: &[u8], start: usize) -> Option<(usize, usize)> {
    let mut result = 0;
    let mut shift = 0;
    let mut pos = start;

    for _ in 0..5 {
        if pos >= data.len() {
            return None;
        }

        let byte = data[pos];
        pos += 1;

        result |= ((byte & 0x7F) as usize) << shift;

        if (byte & 0x80) == 0 {
            return Some((result, pos));
        }

        shift += 7;
    }

    None
}

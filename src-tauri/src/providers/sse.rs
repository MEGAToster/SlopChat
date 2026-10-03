#[derive(Default)]
pub struct SseParser {
    buffer: Vec<u8>,
    data: String,
}

impl SseParser {
    // Bytes are buffered and decoded per complete line, so a multi-byte
    // character split across reads isn't mangled by a lossy decode.
    pub fn push(&mut self, bytes: &[u8]) -> Vec<String> {
        self.buffer.extend_from_slice(bytes);
        self.drain(false)
    }

    // Flushes a stream that ended without its trailing blank line.
    pub fn finish(&mut self) -> Vec<String> {
        self.drain(true)
    }

    fn drain(&mut self, eof: bool) -> Vec<String> {
        let mut events = Vec::new();

        while let Some(index) = self.buffer.iter().position(|byte| *byte == b'\n') {
            let line: Vec<u8> = self.buffer.drain(..=index).collect();
            let text = String::from_utf8_lossy(&line[..line.len() - 1]);
            self.absorb(text.trim_end_matches('\r'), &mut events);
        }

        if eof {
            if !self.buffer.is_empty() {
                let text = String::from_utf8_lossy(&self.buffer).trim_end_matches('\r').to_string();
                self.buffer.clear();
                self.absorb(&text, &mut events);
            }
            if !self.data.is_empty() {
                events.push(std::mem::take(&mut self.data));
            }
        }

        events
    }

    fn absorb(&mut self, line: &str, events: &mut Vec<String>) {
        if line.is_empty() {
            if !self.data.is_empty() {
                events.push(std::mem::take(&mut self.data));
            }
        } else if let Some(value) = line.strip_prefix("data:") {
            if !self.data.is_empty() {
                self.data.push('\n');
            }
            self.data.push_str(value.trim_start());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_events_split_across_reads() {
        let mut parser = SseParser::default();
        assert!(parser.push(b"data: {\"a\"").is_empty());
        assert_eq!(
            parser.push(b":1}\n\ndata: [DONE]\n\n"),
            vec!["{\"a\":1}".to_string(), "[DONE]".to_string()],
        );
    }

    #[test]
    fn ignores_comment_lines() {
        let mut parser = SseParser::default();
        assert_eq!(
            parser.push(b": keep-alive\ndata: hi\n\n"),
            vec!["hi".to_string()],
        );
    }

    #[test]
    fn handles_crlf_delimiters() {
        let mut parser = SseParser::default();
        assert_eq!(parser.push(b"data: x\r\n\r\n"), vec!["x".to_string()]);
    }

    #[test]
    fn decodes_multibyte_split_across_reads() {
        let mut parser = SseParser::default();
        let text = "data: grüße 👋\n\n";
        let split = text.find('ü').unwrap() + 1;

        assert!(parser.push(&text.as_bytes()[..split]).is_empty());
        assert_eq!(
            parser.push(&text.as_bytes()[split..]),
            vec!["grüße 👋".to_string()],
        );
    }

    #[test]
    fn flushes_final_event_without_trailing_blank_line() {
        let mut parser = SseParser::default();
        assert!(parser.push(b"data: tail").is_empty());
        assert_eq!(parser.finish(), vec!["tail".to_string()]);
        assert!(parser.finish().is_empty());
    }
}

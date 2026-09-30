#[derive(Default)]
pub struct SseParser {
    buffer: String,
    data: String,
}

impl SseParser {
    pub fn push(&mut self, bytes: &[u8]) -> Vec<String> {
        self.buffer.push_str(&String::from_utf8_lossy(bytes));

        let mut events = Vec::new();
        while let Some(index) = self.buffer.find('\n') {
            let line = self.buffer[..index].trim_end_matches('\r').to_string();
            self.buffer.drain(..=index);

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

        events
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
}

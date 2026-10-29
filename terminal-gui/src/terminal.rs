pub(crate) struct TerminalScreen {
    parser: vt100::Parser,
}

impl TerminalScreen {
    pub(crate) fn new() -> Self {
        Self {
            parser: vt100::Parser::new(24, 80, 1000),
        }
    }

    pub(crate) fn process(&mut self, bytes: &[u8]) {
        self.parser.process(bytes);
    }

    pub(crate) fn screen(&self) -> &vt100::Screen {
        self.parser.screen()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn carriage_return_overwrites_existing_text() {
        let mut parser = vt100::Parser::new(24, 80, 0);

        parser.process(b"hello\rHi");

        assert_eq!(parser.screen().contents(), "Hillo");
    }

    #[test]
    fn cursor_movement_can_arrive_in_separate_chunks() {
        let mut parser = vt100::Parser::new(24, 80, 0);

        parser.process(b"hello");
        parser.process(b"\x1b[");
        parser.process(b"2D");
        parser.process(b"XY");

        assert_eq!(parser.screen().contents(), "helXY");
    }

    #[test]
    fn utf8_character_can_arrive_in_separate_chunks() {
        let mut parser = vt100::Parser::new(24, 80, 0);
        let bytes = "é".as_bytes();

        parser.process(&bytes[..1]);
        parser.process(&bytes[1..]);

        assert_eq!(parser.screen().contents(), "é");
    }
}
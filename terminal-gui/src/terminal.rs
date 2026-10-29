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
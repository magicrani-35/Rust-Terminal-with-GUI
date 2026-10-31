#[derive(Default)]
struct TerminalReplies {
    pending: Vec<u8>,
}

impl vt100::Callbacks for TerminalReplies {
    fn unhandled_csi(
        &mut self,
        screen: &mut vt100::Screen,
        first: Option<u8>,
        second: Option<u8>,
        params: &[&[u16]],
        command: char,
    ) {
        if first.is_none()
            && second.is_none()
            && command == 'n'
            && params == &[&[6_u16][..]]
        {
            let (row, col) = screen.cursor_position();

            let reply = format!(
                "\x1b[{};{}R",
                row + 1,
                col + 1,
            );

            self.pending.extend_from_slice(reply.as_bytes());
        }
    }
}

pub(crate) struct TerminalScreen {
    parser: vt100::Parser<TerminalReplies>,
}

impl TerminalScreen {
    pub(crate) fn new() -> Self {
       Self {
        parser: vt100::Parser::new_with_callbacks(
            24,
            80,
            1000,
            TerminalReplies::default(),
        ),
       }
    }

    pub(crate) fn process(&mut self, bytes: &[u8]) {
        self.parser.process(bytes);
    }

    pub(crate) fn screen(&self) -> &vt100::Screen {
        self.parser.screen()
    }

    pub(crate) fn take_replies(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.parser.callbacks_mut().pending)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn replies_to_split_cursor_position_query() {
        let mut terminal = super::TerminalScreen::new();

        terminal.process(b"hello");
        terminal.process(b"\x1b[");
        terminal.process(b"6n");
        
        assert_eq!(terminal.take_replies(), b"\x1b[1;6R");
        assert!(terminal.take_replies().is_empty());
        assert_eq!(terminal.screen().contents(), "hello");

    }

    #[test]
    fn ignores_other_status_queries() {
        let mut terminal = super::TerminalScreen::new();

        terminal.process(b"\x1b[5n");

        assert!(terminal.take_replies().is_empty());
    }

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
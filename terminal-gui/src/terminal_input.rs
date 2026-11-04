use eframe::egui;

pub(crate) fn encode(
    event: &egui::Event,
    application_cursor: bool,
) -> Option<Vec<u8>> {
    match event {
        egui::Event::Text(text) => Some(text.as_bytes().to_vec()),

        egui::Event::Key {
            key,
            pressed: true,
            modifiers,
            ..
        } if !modifiers.ctrl
            && !modifiers.alt
            && !modifiers.command
            && !modifiers.mac_cmd => 
            {
                match key {
                    egui::Key::Enter => Some(vec![b'\r']),
                    egui::Key::Backspace => Some(vec![0x7f]),
                    egui::Key::ArrowUp => Some(arrow(b'A',
                        application_cursor)),
                    egui::Key::ArrowDown => Some(arrow(b'B',
                        application_cursor)),
                    egui::Key::ArrowRight => Some(arrow(b'C',
                        application_cursor)),
                    egui::Key::ArrowLeft => Some(arrow(b'D',
                        application_cursor)),
                    _ => None,
                }
            }
        _ => None,
    }
}

fn arrow(direction: u8, application_cursor: bool) -> Vec<u8> {
    let prefix = if application_cursor { b'O' } else { b'[' };

    vec![0x1b, prefix, direction]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_event(key: egui::Key, pressed: bool) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        }
    }

    #[test]
    fn preserves_utf8_text() {
        let event = egui::Event::Text(String::from("cafe"));

        assert_eq!(encode(&event, false),
        Some("cafe".as_bytes().to_vec()));
    }

    #[test]
    fn encodes_return_and_backspace() {
        assert_eq!(
            encode(&key_event(egui::Key::Enter, true), false),
            Some(vec![b'\r']),
        );

        assert_eq!(
            encode(&key_event(egui::Key::Backspace, true), false),
            Some(vec![0x7f]),
        );
    }

    #[test]
    fn ignores_releases_and_letter_key_events() {
        assert_eq!(
            encode(&key_event(egui::Key::Enter, false), false),
            None,
        );

        assert_eq!(
            encode(&key_event(egui::Key::A, true), false),
            None,
        );
    }

    #[test]
    fn encodes_arrows_in_both_cursor_modes() {
        let cases = [
            (egui::Key::ArrowUp, b'A'),
            (egui::Key::ArrowDown, b'B'),
            (egui::Key::ArrowRight, b'C'),
            (egui::Key::ArrowLeft, b'D'),
        ];

        for (key, direction) in cases {
            let event = key_event(key, true);

            assert_eq!(
                encode(&event, false),
                Some(vec![0x1b, b'[', direction]),
            );

            assert_eq!(
                encode(&event, true),
                Some(vec![0x1b, b'O', direction]),
            );
        }
    }
}
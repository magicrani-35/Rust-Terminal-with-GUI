use eframe::egui;

pub(crate) fn encode(event: &egui::Event) -> Option<Vec<u8>> {
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
                    _ => None,
                }
            }
        _ => None,
    }
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

        assert_eq!(encode(&event),
        Some("cafe".as_bytes().to_vec()));
    }

    #[test]
    fn encodes_return_and_backspace() {
        assert_eq!(
            encode(&key_event(egui::Key::Enter, true)),
            Some(vec![b'\r']),
        );

        assert_eq!(
            encode(&key_event(egui::Key::Backspace, true)),
            Some(vec![0x7f]),
        );
    }

    #[test]
    fn ignores_releases_and_letter_key_events() {
        assert_eq!(
            encode(&key_event(egui::Key::Enter, false)),
            None,
        );

        assert_eq!(
            encode(&key_event(egui::Key::A, true)),
            None,
        );
    }
}
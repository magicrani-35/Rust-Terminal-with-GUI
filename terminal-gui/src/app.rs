use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, SyncSender};

use crate::shell_path;
use crate::session;
use crate::session::{SessionCommand, SessionEvent};

pub(crate) struct TerminalApp {
    shell_path: Result<PathBuf, String>,
    events: Receiver<SessionEvent>,
    output: Vec<u8>,
    status: String,
    input_sender: SyncSender<SessionCommand>,
    command_input: String,
}

const OUTPUT_LIMIT: usize = 64 * 1024;

impl TerminalApp {
    pub(crate) fn new(ctx: egui::Context) -> Self {
        let (events, input_sender) = session::start(ctx);

        Self {
            shell_path: shell_path::locate()
                .map_err(|error| error.to_string()),
            events,
            input_sender,
            output: Vec::new(),
            status: String::from("Waiting for the worker..."),
            command_input: String::new(),
        }
    }

    fn append_output(&mut self, bytes: &[u8]) {
        if bytes.len() >= OUTPUT_LIMIT {
            self.output.clear();
            self.output.extend_from_slice(
                &bytes[bytes.len() - OUTPUT_LIMIT..],
            );
            return;
        }

        let overflow = self.output.len()
            .saturating_add(bytes.len())
            .saturating_sub(OUTPUT_LIMIT);

        if overflow > 0 {
            self.output.drain(..overflow);
        }

        self.output.extend_from_slice(bytes);
    }

    fn send_command(&mut self) {
        if self.command_input.is_empty() {
            return;
        }

        let mut bytes = self.command_input.as_bytes().to_vec();
        bytes.push(b'\n');

        match self.input_sender.try_send(SessionCommand::Input(bytes)) {
            Ok(()) => {
                self.command_input.clear();
                self.status = String::from("Input queued");
            }
            Err(error) => {
                self.status = format!("Could not queue input: {error}");
            }
        }
    }
}

impl eframe::App for TerminalApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut received = 0;

        for _ in 0..32 {
            let event = match self.events.try_recv() {
                Ok(event) => event,
                Err(_) => break,
            };

            match event {
                SessionEvent::Output(bytes) => {
                    self.append_output(&bytes);
                }
                SessionEvent::Status(message) => {
                    self.status = message;
                }
            }

            received += 1;
        }

        if received == 32 {
            ui.ctx().request_repaint();
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Rust Terminal");

            ui.horizontal(|ui| {
                ui.label("Command:");
                ui.text_edit_singleline(&mut self.command_input);

                if ui.button("Send").clicked() {
                    self.send_command();
                }
            });

            if ui.button("Stop shell").clicked() {
                match self.input_sender.try_send(SessionCommand::Shutdown) {
                    Ok(()) => {
                        self.status = String::from("Shutdown queued");
                    }
                    Err(error) => {
                        self.status = format!("Could not queue shutdown: {error}");
                    }
                }
            }

            if ui.button("Send Ctrl+D").clicked() {
                match self.input_sender.try_send(
                    SessionCommand::Input(vec![0x04]),
                ) {
                    Ok(()) => {
                        self.status = String::from("Ctrl+D queued");
                    }
                    Err(error) => {
                        self.status = format!("Could not queue Ctrl+D: {error}");
                    }
                }
            }
            
            if ui.button("Send exit command").clicked() {
                match self.input_sender.try_send(
                    SessionCommand::Input(b"exit\n".to_vec()),
                ) {
                    Ok(()) => {
                        self.status = String::from("Exit command queued");
                    }
                    Err(error) => {
                        self.status = format!("Could not queue exit: {error}");
                    }
                }
            }

            ui.label(&self.status);
            let text = String::from_utf8_lossy(&self.output);
            ui.monospace(format!("Pty output: {text:?}"));

            ui.label("Our terminal window is running...");

            // Will be the home to GUI's state.
            match &self.shell_path {
                Ok(path) => {
                    ui.label("Shell executable found");
                    ui.monospace(path.display().to_string());
                }
                Err(message) => {
                    ui.label(message);
                }
            }

        });
    }  
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_keeps_newest_bytes() {
        let (_sender, receiver) = std::sync::mpsc::sync_channel(1);

        let (input_sender, _input_receiver) = std::sync::mpsc::sync_channel::<SessionCommand>(1);

        let mut app = TerminalApp {
            shell_path: Err(String::from("unused in this test")),
            events: receiver,
            status: String::new(),
            output: Vec::new(),
            input_sender,
            command_input: String::new(),
        };

        app.append_output(&vec! [b'a'; OUTPUT_LIMIT -
        2]);
        app.append_output(b"bcde");

        assert_eq!(app.output.len(), OUTPUT_LIMIT);
        assert!(app.output.ends_with(b"bcde"));

        app.append_output(&vec![b'z'; OUTPUT_LIMIT + 10]);

        assert_eq!(app.output, vec![b'z'; OUTPUT_LIMIT]);
    }
}
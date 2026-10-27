use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;
use std::io::{self, Read, Write};
use std::time::Duration;
use std::sync::mpsc::RecvTimeoutError;

use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use eframe::egui;

use crate::process::ManagedChild;

pub(crate) enum SessionEvent {
    Output(Vec<u8>),
    Status(String),
}

pub(crate) enum SessionCommand {
    Input(Vec<u8>),
    Shutdown,
}

pub(crate) fn start(
    ctx: egui::Context,
) -> (Receiver<SessionEvent>, SyncSender<SessionCommand>) {
    let (sender, receiver) = mpsc::sync_channel(32);

    let (input_sender, input_receiver) = mpsc::sync_channel::<SessionCommand>(1);

    thread::spawn(move || {
        let message = match run_probe(&sender, &ctx, &input_receiver) {
            Ok(message) => message,
            Err(error) => format!("PTY startup failed: {error}"),
        };

        if sender.send(SessionEvent::Status(message)).is_ok() {
            ctx.request_repaint();
        }
    });

    (receiver, input_sender)
}

fn run_probe(
    sender: &SyncSender<SessionEvent>,
    ctx: &egui::Context,
    input: &Receiver<SessionCommand>,
) -> Result<String, String> {
    let system = native_pty_system();

    let pair = system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    })
    .map_err(|error| error.to_string())?;

    let mut reader = pair.master.try_clone_reader()
        .map_err(|error| error.to_string())?;

    let mut writer = pair.master.take_writer()
        .map_err(|error| error.to_string())?;

    let command = CommandBuilder::new("/bin/cat");

    let child = pair.slave.spawn_command(command)
        .map_err(|error| error.to_string())?;

    drop(pair.slave);

    thread::scope(|scope| {
        let mut child = ManagedChild::new(child);

        let output_worker = scope.spawn(move || {
            stream_output(reader.as_mut(), sender, ctx)
        });

        let status = loop {
            if let Some(status) = child.try_wait()
                .map_err(|error| error.to_string())?
            {
                break status;
            }

            match input.recv_timeout(Duration::from_millis(25))
            {
                Ok(SessionCommand::Input(bytes)) => {
                    writer.write_all(&bytes)
                        .map_err(|error| {
                            format!("Could not write PTY input: {error}")
                        })?;
                }
                Ok(SessionCommand::Shutdown) => {
                    return Ok(String::from("PTY probe stopped"));
                }
                Err(RecvTimeoutError::Timeout) => {
                    continue;
                }
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(String::from("GUI input channel closed"));
                }
            }
        };

        let read_result = output_worker.join()
            .map_err(|_| String::from("PTY reader thread panicked"))?;

        read_result.map_err(|error| error.to_string())?;

        Ok(format!("PTY diagnostic finished: {status}"))
    })
}

fn stream_output(
    reader: &mut dyn Read,
    sender: &SyncSender<SessionEvent>,
    ctx: &egui::Context,
) -> io::Result<()> {
    let mut buffer = [0_u8; 1024];

    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => {
                let bytes = buffer[..count].to_vec();

                sender.send(SessionEvent::Output(bytes))
                    .map_err(|_| io::Error::new(
                        io::ErrorKind::BrokenPipe, 
                        "GUI receiver closed",
                    ))?;

                    ctx.request_repaint();
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {
                continue;
            }
            Err(error) => return Err(error),
        }
    }
    
    Ok(())
} 
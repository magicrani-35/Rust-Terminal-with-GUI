use std::io;

use portable_pty::{Child, ExitStatus};

pub(crate) struct ManagedChild {
    child: Box<dyn Child + Send + Sync>,
    reaped: bool,
}

impl ManagedChild {
    pub(crate) fn new(child: Box<dyn Child + Send + Sync>) -> Self {
        Self {
            child,
            reaped: false,
        }
    }

    pub(crate) fn try_wait(&mut self) -> 
    io::Result<Option<ExitStatus>> {
        let status = self.child.try_wait()?;

        if status.is_some() {
            self.reaped = true;
        }

        Ok(status)
    }
}

impl Drop for ManagedChild {
    fn drop(&mut self) {
        if self.reaped {
            return;
        }

        match self.child.try_wait() {
            Ok(Some(_)) => {}
            _ => {
                if self.child.kill().is_ok() {
                    let _ = self.child.wait();
                }
            }
        }
    }
}
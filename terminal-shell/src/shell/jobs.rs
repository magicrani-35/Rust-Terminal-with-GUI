use std::process::Child;

pub(crate) struct Job {
    pub(crate) id: usize,
    pub(crate) child: Child,
    pub(crate) command: String,
}

pub(crate) fn report_jobs(jobs: &mut Vec<Job>, include_running: bool) {
    let job_count = jobs.len();
    let mut finished_ids = Vec::new();

    for (index, job) in jobs.iter_mut().enumerate() {
        let marker = if index + 1 == job_count {
            '+'
        } else if index + 2 == job_count {
            '-'
        } else {
            ' '
        };

        let finished = match job.child.try_wait() {
            Ok(Some(_status)) => true,
            Ok(None) => false,
            Err(e) => {
                eprintln!("jobs: {}: {e}", job.id);
                continue;
            }
        };

        if finished || include_running {
            let status = if finished { "Done" } else { "Running" };
            let suffix = if finished { "" } else { " &" };

            println!(
                "[{}]{}  {:<24}{}{}",
                job.id,
                marker,
                status,
                job.command,
                suffix
            );
        }

        if finished {
            finished_ids.push(job.id);
        }
    }

    jobs.retain(|job| !finished_ids.contains(&job.id));
}
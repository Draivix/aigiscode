use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub(super) enum ReportStream {
    Stdout,
    Stderr,
    File,
}

/// Raw streams are files, so a verbose child never waits for a full parent pipe.
pub(super) struct CapturePaths {
    pub stdout: PathBuf,
    pub stderr: PathBuf,
}

impl CapturePaths {
    pub fn new(report: &Path, stream: ReportStream) -> Self {
        Self {
            stdout: if matches!(stream, ReportStream::Stdout) {
                report.to_path_buf()
            } else {
                report.with_extension("stdout.txt")
            },
            stderr: if matches!(stream, ReportStream::Stderr) {
                report.to_path_buf()
            } else {
                report.with_extension("stderr.txt")
            },
        }
    }

    pub fn stderr_preview(&self) -> io::Result<String> {
        let mut bytes = Vec::new();
        File::open(&self.stderr)?
            .take(64 * 1024)
            .read_to_end(&mut bytes)?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
}

#[derive(Debug)]
pub(super) enum ProcessError {
    Io(io::Error),
    Timeout(Duration),
}

impl From<io::Error> for ProcessError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

fn capture_file(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

pub(super) fn run(
    command: &[String],
    cwd: Option<&Path>,
    captures: &CapturePaths,
    timeout: Duration,
) -> Result<ExitStatus, ProcessError> {
    let executable = command
        .first()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "empty external command"))?;
    let mut command_builder = Command::new(executable);
    command_builder
        .args(&command[1..])
        .stdin(Stdio::null())
        .stdout(capture_file(&captures.stdout)?)
        .stderr(capture_file(&captures.stderr)?);
    if let Some(cwd) = cwd {
        command_builder.current_dir(cwd);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command_builder.process_group(0);
    }
    let mut child = OwnedChild(Some(command_builder.spawn()?));
    let started = Instant::now();
    loop {
        let handle = child.0.as_mut().expect("owned until reaped");
        #[cfg(unix)]
        if exited_without_reaping(handle)? {
            terminate_group(handle)?;
            let status = handle.wait()?;
            child.0 = None;
            return Ok(status);
        }
        #[cfg(not(unix))]
        if let Some(status) = handle.try_wait()? {
            child.0 = None;
            return Ok(status);
        }
        if started.elapsed() >= timeout {
            // Explicit cleanup reports failures; Drop also covers every earlier error.
            child.terminate()?;
            return Err(ProcessError::Timeout(timeout));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

struct OwnedChild(Option<Child>);

impl OwnedChild {
    fn terminate(&mut self) -> io::Result<()> {
        if let Some(child) = self.0.as_mut() {
            #[cfg(unix)]
            terminate_group(child)?;
            #[cfg(not(unix))]
            child.kill()?;
            child.wait()?;
            self.0 = None;
        }
        Ok(())
    }
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        if let Err(error) = self.terminate() {
            eprintln!("external process cleanup failed: {error}");
        }
    }
}

#[cfg(unix)]
fn exited_without_reaping(child: &Child) -> io::Result<bool> {
    // WNOWAIT reserves the child's PID while we terminate its process group;
    // reaping first would allow PID reuse before cleanup.
    let mut info = std::mem::MaybeUninit::<libc::siginfo_t>::zeroed();
    // SAFETY: info points to writable storage; child is our unreaped child.
    let result = unsafe {
        libc::waitid(
            libc::P_PID,
            child.id() as libc::id_t,
            info.as_mut_ptr(),
            libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
        )
    };
    if result == -1 {
        let error = io::Error::last_os_error();
        if error.kind() == io::ErrorKind::Interrupted {
            return Ok(false);
        }
        return Err(error);
    }
    // SAFETY: waitid succeeded and initialized info (zero PID means still running).
    Ok(unsafe { info.assume_init().si_pid() } != 0)
}

#[cfg(unix)]
fn terminate_group(child: &Child) -> io::Result<()> {
    let pid = i32::try_from(child.id())
        .map_err(|_| io::Error::other("child PID exceeds the process-group range"))?;
    // SAFETY: process_group(0) created this group; its leader remains unreaped,
    // preventing reuse while this handle owns cleanup. No borrowed process group.
    if unsafe { libc::kill(-pid, libc::SIGKILL) } == -1 {
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::ESRCH) {
            return Err(error);
        }
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn fixture() -> PathBuf {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "aigiscode-process-{}-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        ));
        fs::create_dir(&root).unwrap();
        root
    }

    #[test]
    fn captures_both_large_streams_without_pipe_backpressure() {
        let root = fixture();
        let captures = CapturePaths::new(&root.join("report.json"), ReportStream::Stdout);
        let status = run(
            &[
                "/bin/sh".into(),
                "-c".into(),
                "head -c 2097152 /dev/zero & head -c 2097152 /dev/zero >&2 & wait".into(),
            ],
            Some(&root),
            &captures,
            Duration::from_secs(10),
        )
        .unwrap();
        assert!(status.success());
        assert_eq!(fs::metadata(&captures.stdout).unwrap().len(), 2_097_152);
        assert_eq!(fs::metadata(&captures.stderr).unwrap().len(), 2_097_152);
        assert_eq!(captures.stderr_preview().unwrap().len(), 65_536);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn timeout_preserves_partial_output() {
        let root = fixture();
        let captures = CapturePaths::new(&root.join("report.json"), ReportStream::Stdout);
        let result = run(
            &[
                "/bin/sh".into(),
                "-c".into(),
                "printf partial; sleep 30".into(),
            ],
            Some(&root),
            &captures,
            Duration::from_millis(500),
        );
        assert!(matches!(result, Err(ProcessError::Timeout(_))));
        assert_eq!(fs::read_to_string(&captures.stdout).unwrap(), "partial");
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn normal_exit_terminates_descendants_before_releasing_the_leader_pid() {
        let root = fixture();
        let captures = CapturePaths::new(&root.join("report.json"), ReportStream::Stdout);
        let status = run(
            &[
                "/bin/sh".into(),
                "-c".into(),
                "sleep 30 & printf '%s' $! > child.pid".into(),
            ],
            Some(&root),
            &captures,
            Duration::from_secs(5),
        )
        .unwrap();
        assert!(status.success());
        let pid = fs::read_to_string(root.join("child.pid")).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let state = fs::read_to_string(format!("/proc/{pid}/stat"));
            if state
                .as_ref()
                .is_err_and(|error| error.kind() == io::ErrorKind::NotFound)
                || state.as_ref().is_ok_and(|stat| stat.contains(") Z"))
            {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "descendant is still running: {state:?}"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        fs::remove_dir_all(root).unwrap();
    }
}

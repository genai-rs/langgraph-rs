use anyhow::{bail, ensure, Context, Result};
use clap::Args;
use langgraph_inspector::GraphInfo;
use serde_json::{json, Value};
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};
use tokio::task::JoinHandle;
use tokio::time::{sleep_until, timeout_at, Instant};

const PROTOCOL_VERSION: u64 = 1;
const MAX_STDOUT_BYTES: usize = 16 * 1024 * 1024;
const MAX_STDERR_BYTES: usize = 1024 * 1024;
const CLEANUP_GRACE: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Args)]
pub struct ExtractionOptions {
    /// Python executable to use (overrides LANGGRAPH_PYTHON and PATH lookup)
    #[arg(long, value_name = "PATH")]
    pub python: Option<PathBuf>,

    /// Explicit dotted graph object to extract from the Python file
    #[arg(long, value_name = "OBJECT")]
    pub graph_object: String,

    /// Maximum helper runtime and output collection time (default: 30s)
    #[arg(long, value_name = "DURATION", value_parser = parse_timeout, default_value = "30s")]
    pub timeout: Duration,
}

fn parse_timeout(value: &str) -> std::result::Result<Duration, String> {
    let (number, multiplier) = if let Some(number) = value.strip_suffix("ms") {
        (number, 1u64)
    } else if let Some(number) = value.strip_suffix('s') {
        (number, 1_000)
    } else if let Some(number) = value.strip_suffix('m') {
        (number, 60_000)
    } else if let Some(number) = value.strip_suffix('h') {
        (number, 3_600_000)
    } else {
        (value, 1_000)
    };
    let amount = number
        .parse::<u64>()
        .map_err(|_| "timeout must be a positive duration such as 30s".to_string())?;
    if amount == 0 {
        return Err("timeout must be greater than zero".to_string());
    }
    let millis = amount
        .checked_mul(multiplier)
        .ok_or_else(|| "timeout is too large".to_string())?;
    Ok(Duration::from_millis(millis))
}

pub async fn extract_graph_info(input: &Path, options: &ExtractionOptions) -> Result<GraphInfo> {
    let metadata = fs::metadata(input)
        .with_context(|| format!("Cannot inspect Python input: {}", input.display()))?;
    ensure!(
        metadata.is_file(),
        "Input is not a regular file: {}",
        input.display()
    );
    let source_file = fs::canonicalize(input)
        .with_context(|| format!("Cannot resolve Python input: {}", input.display()))?;

    let request = json!({
        "protocol_version": PROTOCOL_VERSION,
        "operation": "extract_graph_info",
        "source_file": source_file,
        "graph_object": options.graph_object,
    });
    let request = serde_json::to_vec(&request).context("Cannot encode helper request")?;
    let interpreter = select_interpreter(options.python.as_deref())?;
    let deadline = Instant::now() + options.timeout;

    let mut child = Command::new(&interpreter)
        .arg("-m")
        .arg("langgraph_inspector.helper")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .with_context(|| format!("Cannot launch Python interpreter {:?}", interpreter))?;

    let stdin = child
        .stdin
        .take()
        .context("Python helper stdin was not piped")?;
    let stdout = child
        .stdout
        .take()
        .context("Python helper stdout was not piped")?;
    let stderr = child
        .stderr
        .take()
        .context("Python helper stderr was not piped")?;
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();

    let mut writer = tokio::spawn(async move {
        let mut stdin = stdin;
        stdin.write_all(&request).await?;
        stdin.shutdown().await
    });
    let mut stdout_task = tokio::spawn(read_stdout(stdout, event_tx.clone()));
    let mut stderr_task = tokio::spawn(read_stderr_tail(stderr));
    let event_guard = event_tx;
    let outcome = tokio::select! {
        status = child.wait() => WaitOutcome::Status(status),
        event = event_rx.recv() => WaitOutcome::Event(event),
        _ = sleep_until(deadline) => WaitOutcome::Deadline,
    };
    drop(event_guard);

    let status = match outcome {
        WaitOutcome::Status(status) => status.context("Failed while waiting for Python helper")?,
        WaitOutcome::Event(Some(ChildEvent::StdoutLimitExceeded)) => {
            terminate_and_cancel(&mut child, &mut writer, &mut stdout_task, &mut stderr_task).await;
            bail!("Python helper stdout exceeded the 16 MiB protocol limit");
        }
        WaitOutcome::Event(None) => unreachable!("event sender remains alive until after wait"),
        WaitOutcome::Deadline => {
            terminate_and_cancel(&mut child, &mut writer, &mut stdout_task, &mut stderr_task).await;
            bail!(
                "Python metadata extraction timed out after {:?}",
                options.timeout
            );
        }
    };

    let collected = tokio::select! {
        result = collect_output(&mut writer, &mut stdout_task, &mut stderr_task) => result,
        _ = sleep_until(deadline) => {
            cancel_tasks(
                &mut writer,
                &mut stdout_task,
                &mut stderr_task,
                Instant::now() + CLEANUP_GRACE,
            ).await;
            bail!("Python metadata extraction timed out while collecting output after {:?}", options.timeout);
        }
    };
    let (stdout, stderr, input_result) = collected?;
    let stderr = String::from_utf8_lossy(&stderr).trim().to_string();
    if !status.success() {
        let detail = if stderr.is_empty() {
            String::new()
        } else {
            format!("; helper stderr: {stderr}")
        };
        bail!("Python helper exited with {status}{detail}");
    }
    input_result.context("Failed to send the extraction request to Python helper")?;

    let response: Value = serde_json::from_slice(&stdout)
        .context("Python helper returned malformed JSON or more than one JSON value")?;
    let object = response
        .as_object()
        .context("Python helper response must be a JSON object")?;
    ensure!(
        object.len() == 2,
        "Python helper response has unknown or missing fields"
    );
    ensure!(
        object.get("protocol_version").and_then(Value::as_u64) == Some(PROTOCOL_VERSION),
        "Python helper returned an unsupported protocol version"
    );
    let graph_info = object
        .get("graph_info")
        .cloned()
        .context("Python helper response is missing graph_info")?;
    serde_json::from_value(graph_info).context("Python helper returned invalid GraphInfo metadata")
}

fn select_interpreter(explicit: Option<&Path>) -> Result<OsString> {
    if let Some(explicit) = explicit {
        ensure!(!explicit.as_os_str().is_empty(), "--python cannot be empty");
        return Ok(explicit.as_os_str().to_owned());
    }
    if let Some(from_env) = env::var_os("LANGGRAPH_PYTHON") {
        ensure!(!from_env.is_empty(), "LANGGRAPH_PYTHON cannot be empty");
        return Ok(from_env);
    }

    if find_on_path(OsStr::new("python3")).is_some() {
        return Ok(OsString::from("python3"));
    }
    #[cfg(windows)]
    if find_on_path(OsStr::new("python")).is_some() {
        return Ok(OsString::from("python"));
    }
    bail!("Python interpreter not found; pass --python or set LANGGRAPH_PYTHON")
}

fn find_on_path(program: &OsStr) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    for directory in env::split_paths(&path) {
        let candidate = directory.join(program);
        if candidate.is_file() {
            return Some(candidate);
        }
        #[cfg(windows)]
        for extension in [".exe", ".cmd", ".bat"] {
            let candidate = directory.join(format!("{}{extension}", program.to_string_lossy()));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

enum ChildEvent {
    StdoutLimitExceeded,
}

enum WaitOutcome {
    Status(io::Result<std::process::ExitStatus>),
    Event(Option<ChildEvent>),
    Deadline,
}

async fn read_stdout<R>(
    mut reader: R,
    event_tx: tokio::sync::mpsc::UnboundedSender<ChildEvent>,
) -> io::Result<Vec<u8>>
where
    R: AsyncRead + Unpin,
{
    let mut captured = Vec::with_capacity(8192);
    let mut chunk = [0u8; 8192];
    let mut exceeded = false;
    loop {
        let read = reader.read(&mut chunk).await?;
        if read == 0 {
            break;
        }
        if !exceeded && captured.len().saturating_add(read) <= MAX_STDOUT_BYTES {
            captured.extend_from_slice(&chunk[..read]);
        } else if !exceeded {
            exceeded = true;
            let _ = event_tx.send(ChildEvent::StdoutLimitExceeded);
        }
    }
    if exceeded {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "stdout limit exceeded",
        ))
    } else {
        Ok(captured)
    }
}

struct TailBuffer {
    bytes: Vec<u8>,
    start: usize,
    len: usize,
}

impl TailBuffer {
    fn new() -> Self {
        Self {
            bytes: vec![0; MAX_STDERR_BYTES],
            start: 0,
            len: 0,
        }
    }

    fn push(&mut self, chunk: &[u8]) {
        if chunk.len() >= MAX_STDERR_BYTES {
            self.bytes
                .copy_from_slice(&chunk[chunk.len() - MAX_STDERR_BYTES..]);
            self.start = 0;
            self.len = MAX_STDERR_BYTES;
            return;
        }
        for byte in chunk {
            if self.len < MAX_STDERR_BYTES {
                let index = (self.start + self.len) % MAX_STDERR_BYTES;
                self.bytes[index] = *byte;
                self.len += 1;
            } else {
                self.bytes[self.start] = *byte;
                self.start = (self.start + 1) % MAX_STDERR_BYTES;
            }
        }
    }

    fn into_vec(self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.len);
        let first = self.len.min(MAX_STDERR_BYTES - self.start);
        result.extend_from_slice(&self.bytes[self.start..self.start + first]);
        if first < self.len {
            result.extend_from_slice(&self.bytes[..self.len - first]);
        }
        result
    }
}

async fn read_stderr_tail<R>(mut reader: R) -> io::Result<Vec<u8>>
where
    R: AsyncRead + Unpin,
{
    let mut tail = TailBuffer::new();
    let mut chunk = [0u8; 8192];
    loop {
        let read = reader.read(&mut chunk).await?;
        if read == 0 {
            break;
        }
        tail.push(&chunk[..read]);
    }
    Ok(tail.into_vec())
}

async fn collect_output(
    writer: &mut JoinHandle<io::Result<()>>,
    stdout: &mut JoinHandle<io::Result<Vec<u8>>>,
    stderr: &mut JoinHandle<io::Result<Vec<u8>>>,
) -> Result<(Vec<u8>, Vec<u8>, io::Result<()>)> {
    let (writer, stdout, stderr) = tokio::join!(&mut *writer, &mut *stdout, &mut *stderr);
    let input_result = writer.context("Python request writer task failed")?;
    let stdout = stdout
        .context("Python stdout reader task failed")?
        .map_err(|error| {
            if error.kind() == io::ErrorKind::InvalidData {
                anyhow::anyhow!("Python helper stdout exceeded the 16 MiB protocol limit")
            } else {
                anyhow::Error::new(error).context("Failed to read Python helper stdout")
            }
        })?;
    let stderr = stderr
        .context("Python stderr reader task failed")?
        .context("Failed to read Python helper stderr")?;
    Ok((stdout, stderr, input_result))
}

async fn terminate_and_cancel<W, S, E>(
    child: &mut Child,
    writer: &mut JoinHandle<W>,
    stdout: &mut JoinHandle<S>,
    stderr: &mut JoinHandle<E>,
) {
    let cleanup_deadline = Instant::now() + CLEANUP_GRACE;
    let _ = child.start_kill();
    let _ = timeout_at(cleanup_deadline, child.wait()).await;
    cancel_tasks(writer, stdout, stderr, cleanup_deadline).await;
}

async fn cancel_tasks<W, S, E>(
    writer: &mut JoinHandle<W>,
    stdout: &mut JoinHandle<S>,
    stderr: &mut JoinHandle<E>,
    cleanup_deadline: Instant,
) {
    // A cancelled collect_output future may already have polled some handles
    // to Ready while waiting for the remaining pipe readers. Polling a
    // completed JoinHandle again panics, so only abort and reap unfinished
    // tasks.
    let reap_writer = !writer.is_finished();
    let reap_stdout = !stdout.is_finished();
    let reap_stderr = !stderr.is_finished();

    if reap_writer {
        writer.abort();
    }
    if reap_stdout {
        stdout.abort();
    }
    if reap_stderr {
        stderr.abort();
    }

    let cleanup = async {
        if reap_writer {
            let _ = (&mut *writer).await;
        }
        if reap_stdout {
            let _ = (&mut *stdout).await;
        }
        if reap_stderr {
            let _ = (&mut *stderr).await;
        }
    };
    let _ = timeout_at(cleanup_deadline, cleanup).await;
}

#[cfg(test)]
mod tests {
    use super::{cancel_tasks, parse_timeout, TailBuffer, CLEANUP_GRACE, MAX_STDERR_BYTES};
    use std::time::Duration;
    use tokio::task::JoinHandle;
    use tokio::time::{sleep, Instant};

    #[tokio::test]
    async fn task_cancellation_uses_remaining_shared_cleanup_grace() {
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let mut writer: JoinHandle<()> = tokio::task::spawn_blocking(move || {
            started_tx.send(()).unwrap();
            std::thread::sleep(Duration::from_millis(1500));
        });
        started_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("blocking task should start");
        let mut stdout = tokio::spawn(std::future::pending::<()>());
        let mut stderr = tokio::spawn(std::future::pending::<()>());
        let cleanup_deadline = Instant::now() + CLEANUP_GRACE;

        sleep(Duration::from_millis(800)).await;
        let cancel_started = Instant::now();
        cancel_tasks(&mut writer, &mut stdout, &mut stderr, cleanup_deadline).await;

        assert!(
            cancel_started.elapsed() < Duration::from_millis(500),
            "task cancellation restarted the cleanup grace instead of using its remaining time"
        );
    }
    use langgraph_inspector::GraphInfo;

    #[test]
    fn parses_positive_timeouts() {
        assert_eq!(parse_timeout("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_timeout("45").unwrap(), Duration::from_secs(45));
        assert_eq!(parse_timeout("250ms").unwrap(), Duration::from_millis(250));
        assert!(parse_timeout("0s").is_err());
        assert!(parse_timeout("nope").is_err());
    }

    #[test]
    fn stderr_buffer_keeps_only_the_tail() {
        let mut tail = TailBuffer::new();
        tail.push(&vec![b'a'; MAX_STDERR_BYTES + 3]);
        tail.push(b"end");
        let output = tail.into_vec();
        assert_eq!(output.len(), MAX_STDERR_BYTES);
        assert!(output.ends_with(b"end"));
    }

    #[test]
    fn graph_info_deserialization_rejects_unknown_fields() {
        let response = serde_json::json!({
            "nodes": [],
            "edges": [],
            "state_schema": {"fields": []},
            "entry_point": "start",
            "conditional_edges": {},
            "unexpected": true
        });
        assert!(serde_json::from_value::<GraphInfo>(response).is_err());
    }
}

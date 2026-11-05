//! Running a rule's `check` program: arguments only (never through a shell), working directory
//! = the rule's root, stdin and stderr closed, killed with its process group (Windows: its job object) at the timeout.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// What a check run produced. `Err` from [`Runner::run`] means it could not run or timed out.
pub struct Ran {
    pub success: bool,
    pub stdout: String,
}

pub trait Runner {
    fn run(&self, program: &Path, args: &[String], cwd: &Path, timeout: Duration) -> Result<Ran, String>;
}

/// The program as written: `~/x` → home, a relative path with a separator → root, a bare name →
/// looked up on PATH by the OS.
pub fn resolve(program: &str, home: Option<&Path>, root: &Path) -> Result<PathBuf, String> {
    if program.starts_with("~/") || program.contains('/') || program.contains('\\') {
        return crate::paths::expand(program, home, root);
    }
    if program.is_empty() {
        return Err("check.program rỗng".into());
    }
    Ok(PathBuf::from(program))
}

/// The most of stdout a check may print; more is an error (a verdict cut short is not a verdict).
const KEEP: usize = 1 << 20;

/// Runs checks against one shared deadline, so every check ends (and its group is killed)
/// before the hook's own 30-second timer exits the process.
pub struct ProcessRunner {
    deadline: Instant,
}

impl ProcessRunner {
    /// All checks of one hook call together get at most `budget`.
    pub fn new(budget: Duration) -> Self {
        Self::until(Instant::now() + budget)
    }

    /// All checks end by `deadline` (the hook passes one counted from its own start).
    pub fn until(deadline: Instant) -> Self {
        ProcessRunner { deadline }
    }
}

#[cfg(unix)]
fn kill_group(pid: u32) {
    // SAFETY: kill(2) with a negative pid signals that process group; no memory is touched.
    unsafe {
        libc::kill(-(pid as i32), libc::SIGKILL);
    }
}

/// Windows: the check runs in a job object that kills every process in it when it is closed, so a
/// grandchild dies with the check even after its parent has exited (`taskkill /T` finds a tree
/// only through a parent that is still alive; review 1d round 2). The check is created suspended,
/// assigned to the job, then resumed, so it cannot start a child outside the job (review 1d round 3).
#[cfg(windows)]
pub mod job {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation, SetInformationJobObject,
        TerminateJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };

    pub struct Job(HANDLE);

    impl Job {
        pub fn new() -> Result<Job, String> {
            let err = |what: &str| format!("{what} lỗi ({})", std::io::Error::last_os_error());
            // SAFETY: plain Win32 calls on a handle this struct owns; `info` outlives the call.
            unsafe {
                let h = CreateJobObjectW(std::ptr::null(), std::ptr::null());
                if h.is_null() {
                    return Err(err("CreateJobObjectW"));
                }
                let job = Job(h);
                let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
                info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
                let ok = SetInformationJobObject(h, JobObjectExtendedLimitInformation, &info as *const _ as *const _,
                                                 std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32);
                if ok == 0 {
                    return Err(err("SetInformationJobObject"));
                }
                Ok(job)
            }
        }

        pub fn assign(&self, child: &std::process::Child) -> Result<(), String> {
            // SAFETY: both handles are valid for the duration of the call.
            if unsafe { AssignProcessToJobObject(self.0, child.as_raw_handle() as HANDLE) } == 0 {
                return Err(format!("AssignProcessToJobObject lỗi ({})", std::io::Error::last_os_error()));
            }
            Ok(())
        }

        /// Resume every thread of a process created with `CREATE_SUSPENDED` (std keeps the main
        /// thread handle private, so the threads are found through a ToolHelp snapshot).
        pub fn resume(pid: u32) -> Result<(), String> {
            use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
            use windows_sys::Win32::System::Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
            };
            use windows_sys::Win32::System::Threading::{OpenThread, ResumeThread, THREAD_SUSPEND_RESUME};
            let err = |what: &str| format!("{what} lỗi ({})", std::io::Error::last_os_error());
            // SAFETY: plain Win32 calls; every handle opened here is closed here; `te` outlives the calls.
            unsafe {
                let snap = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
                if snap == INVALID_HANDLE_VALUE {
                    return Err(err("CreateToolhelp32Snapshot"));
                }
                let mut te: THREADENTRY32 = std::mem::zeroed();
                te.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;
                let (mut resumed, mut failed) = (0, None);
                let mut more = Thread32First(snap, &mut te) != 0;
                while more {
                    if te.th32OwnerProcessID == pid {
                        let th = OpenThread(THREAD_SUSPEND_RESUME, 0, te.th32ThreadID);
                        if th.is_null() {
                            failed = Some(err("OpenThread"));
                        } else {
                            if ResumeThread(th) == u32::MAX {
                                failed = Some(err("ResumeThread"));
                            } else {
                                resumed += 1;
                            }
                            CloseHandle(th);
                        }
                    }
                    more = Thread32Next(snap, &mut te) != 0;
                }
                CloseHandle(snap);
                match failed {
                    Some(e) => Err(e),
                    None if resumed == 0 => Err("không thấy luồng nào của lệnh kiểm để chạy tiếp".into()),
                    None => Ok(()),
                }
            }
        }

        /// Kill every process still in the job.
        pub fn kill(&self) {
            // SAFETY: valid handle owned by self.
            unsafe {
                TerminateJobObject(self.0, 1);
            }
        }
    }

    impl Drop for Job {
        fn drop(&mut self) {
            self.kill();
            // SAFETY: closed once, here.
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}

impl Runner for ProcessRunner {
    fn run(&self, program: &Path, args: &[String], cwd: &Path, timeout: Duration) -> Result<Ran, String> {
        let mut c = Command::new(program);
        c.args(args).current_dir(cwd).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            c.process_group(0);
        }
        let left = self.deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(format!("hết thời gian cho các lệnh kiểm trước khi chạy {}", program.display()));
        }
        let timeout = timeout.min(left);
        let started = Instant::now();
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            c.creation_flags(windows_sys::Win32::System::Threading::CREATE_SUSPENDED);
        }
        #[cfg(windows)]
        let job = job::Job::new().map_err(|e| format!("không dựng được job object cho {} ({e})", program.display()))?;
        let mut child = c.spawn().map_err(|e| format!("không chạy được {} ({e})", program.display()))?;
        #[cfg(windows)]
        let kill_tree = || job.kill();
        #[cfg(unix)]
        let kill_tree = {
            let pid = child.id();
            move || kill_group(pid)
        };
        #[cfg(windows)]
        if let Err(e) = job.assign(&child).and_then(|()| job::Job::resume(child.id())) {
            // Still suspended (assign failed) or in the job (resume failed): no grandchild exists.
            job.kill();
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("{}: {e}", program.display()));
        }
        let mut out = child.stdout.take().expect("piped stdout");
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut kept = Vec::new();
            let mut buf = [0u8; 16 * 1024];
            let r = loop {
                match out.read(&mut buf) {
                    Ok(0) => break Ok(kept),
                    Ok(n) if kept.len() + n > KEEP => break Err(format!("in ra hơn {} MiB", KEEP >> 20)),
                    Ok(n) => kept.extend_from_slice(&buf[..n]),
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                    Err(e) => break Err(format!("đọc output lỗi ({e})")),
                }
            };
            let _ = tx.send(r);
        });
        let status = loop {
            match child.try_wait() {
                Ok(Some(st)) => break Some(st),
                Ok(None) => {}
                Err(e) => return Err(format!("chờ {} lỗi ({e})", program.display())),
            }
            if started.elapsed() >= timeout {
                kill_tree();
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        kill_tree();
        let Some(st) = status else {
            return Err(format!("{} chạy quá {} giây", program.display(), timeout.as_secs_f32()));
        };
        let kept = match rx.recv_timeout(Duration::from_secs(2)) {
            Ok(r) => r.map_err(|e| format!("{}: {e}", program.display()))?,
            Err(_) => return Err(format!("{}: không đọc xong output", program.display())),
        };
        Ok(Ran { success: st.success(), stdout: String::from_utf8_lossy(&kept).into_owned() })
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn runs_and_times_out() {
        let d = Path::new("/");
        let pr = ProcessRunner::new(Duration::from_secs(25));
        let r = pr.run(Path::new("sh"), &["-c".into(), "echo hi; exit 3".into()], d, Duration::from_secs(5)).unwrap();
        assert!(!r.success);
        assert_eq!(r.stdout, "hi\n");
        let t = Instant::now();
        assert!(pr.run(Path::new("sleep"), &["10".into()], d, Duration::from_secs(1)).is_err());
        assert!(t.elapsed() < Duration::from_secs(5));
        let big = pr.run(Path::new("sh"), &["-c".into(), "head -c 2000000 /dev/zero".into()], d, Duration::from_secs(5));
        assert!(big.is_err(), "output over KEEP must be an error");
        let short = ProcessRunner::new(Duration::from_millis(300));
        let t = Instant::now();
        assert!(short.run(Path::new("sleep"), &["10".into()], d, Duration::from_secs(10)).is_err());
        assert!(t.elapsed() < Duration::from_secs(3), "the shared budget caps the per-check timeout");
        assert!(short.run(Path::new("true"), &[], d, Duration::from_secs(10)).is_err(), "no budget left");
        assert!(pr.run(Path::new("/nonexistent/x"), &[], d, Duration::from_secs(1)).is_err());
    }
}

/// A grandchild left behind by a check that exited 0 must not outlive the run (review 1d round 2).
#[cfg(test)]
mod tree_tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn a_grandchild_dies_with_its_check() {
        let pr = ProcessRunner::new(Duration::from_secs(25));
        let r = pr.run(Path::new("sh"), &["-c".into(), "sleep 60 >/dev/null 2>&1 & echo $!".into()], Path::new("/"),
                       Duration::from_secs(5)).unwrap();
        let pid: i32 = r.stdout.trim().parse().unwrap();
        std::thread::sleep(Duration::from_millis(200));
        // SAFETY: signal 0 only probes whether the pid exists.
        let alive = unsafe { libc::kill(pid, 0) } == 0;
        assert!(!alive, "grandchild {pid} still alive");
    }

    #[cfg(windows)]
    #[test]
    fn a_grandchild_dies_with_its_check() {
        let pr = ProcessRunner::new(Duration::from_secs(25));
        let script = "$p = Start-Process -PassThru -NoNewWindow -FilePath ping -ArgumentList '-n','60','127.0.0.1' \
                      -RedirectStandardOutput NUL; $p.Id";
        let r = pr.run(Path::new("powershell"), &["-NoProfile".into(), "-Command".into(), script.into()],
                       &std::env::temp_dir(), Duration::from_secs(20)).unwrap();
        let pid = r.stdout.trim().to_string();
        assert!(pid.parse::<u32>().is_ok(), "no pid in {:?}", r.stdout);
        std::thread::sleep(Duration::from_millis(500));
        let out = Command::new("tasklist").args(["/FI", &format!("PID eq {pid}"), "/NH"]).output().unwrap();
        let list = String::from_utf8_lossy(&out.stdout);
        assert!(!list.contains(&pid), "grandchild {pid} still alive: {list}");
    }
}

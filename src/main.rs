use glob::glob;
use json::parse;
use nix::sys::signal::{SigSet, Signal};
use nix::sys::wait::{WaitPidFlag, waitpid};
use nix::unistd::Pid;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
mod cgroup_manager; // module name must match the file name
use cgroup_manager::CgroupManager; // import the service cgroup manager

/// Mount the essential pseudo-filesystems required by the initramfs environment.
///
/// This is needed before spawning any shell or service processes so that `/proc`,
/// `/sys`, and `/dev` are available to programs started later.
fn mount_important_filesystems() {
    Command::new("mount")
        .args(["-t", "proc", "proc", "/proc"])
        .status()
        .unwrap();

    Command::new("mount")
        .args(["-t", "sysfs", "sysfs", "/sys"])
        .status()
        .unwrap();

    Command::new("mount")
        .args(["-t", "devtmpfs", "devtmpfs", "/dev"])
        .status()
        .unwrap();

    fs::create_dir_all("/sys/fs/cgroup").unwrap();
    Command::new("mount")
        .args(["-t", "cgroup2", "none", "/sys/fs/cgroup"])
        .status()
        .unwrap();

    Command::new("mount")
        .args(["-t", "tmpfs", "tmpfs", "/run"])
        .status()
        .unwrap();

    fs::create_dir_all("/run/lock").unwrap();

    let links = [("/var/run", "/run"), ("/var/lock", "/run/lock")];

    for (old_dir, new_target) in links.iter() {
        let path = std::path::Path::new(old_dir);
        if path.exists() || path.is_symlink() {
            if path.is_dir() && !path.is_symlink() {
                fs::remove_dir_all(path).unwrap();
            } else {
                fs::remove_file(path).unwrap();
            }
        }
        std::os::unix::fs::symlink(new_target, old_dir).unwrap();
    }
}

/// Reap terminated child processes and restart configured services when needed.
///
/// This function checks all child processes using non-blocking `waitpid` and
/// restarts service entries whose process IDs have exited or been signaled.
fn reap_children(mut service_list: Vec<(u32, PathBuf)>) {
    while let Ok(status) = waitpid(Pid::from_raw(-1), Some(WaitPidFlag::WNOHANG)) {
        match status {
            nix::sys::wait::WaitStatus::Exited(pid, _)
            | nix::sys::wait::WaitStatus::Signaled(pid, _, _) => {
                let raw_pid = pid.as_raw() as u32;

                if let Some(index) = service_list.iter().position(|entry| entry.0 == raw_pid) {
                    let (_, path) = service_list.remove(index);

                    if let Ok(contents) = fs::read_to_string(&path)
                        && let Ok(json) = parse(&contents)
                        && let Some(command) = json["command"].as_str()
                    {
                        let args = json["args"].members().map(|arg| arg.as_str().unwrap_or(""));

                        if let Ok(child) = Command::new(command).args(args).spawn() {
                            let service_name = extract_service_name(&path);
                            if let Err(err_msg) =
                                CgroupManager::attach_service(&service_name, child.id())
                            {
                                eprintln!(
                                    "[rid-cgroups] failed to attach service '{}': {}",
                                    service_name, err_msg
                                );
                            }

                            service_list.push((child.id(), path));
                            std::mem::forget(child);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Start all service definitions found in `/etc/rid/services/`.
///
/// Each JSON service file should contain a `command` string and an optional `args`
/// array. Failed files are skipped silently.
fn extract_service_name(path: &std::path::Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("unknown-service")
        .to_owned()
}

fn service_starter() -> Vec<(u32, PathBuf)> {
    let mut started_services = Vec::new();

    for path in glob("/etc/rid/services/*.json").unwrap().flatten() {
        let contents = match fs::read_to_string(&path) {
            Ok(data) => data,
            Err(_) => continue,
        };

        let json = match parse(&contents) {
            Ok(data) => data,
            Err(_) => continue,
        };

        let command = match json["command"].as_str() {
            Some(cmd) => cmd,
            None => continue,
        };

        let args = json["args"].members().map(|arg| arg.as_str().unwrap_or(""));

        if let Ok(child) = Command::new(command).args(args).spawn() {
            let service_name = extract_service_name(&path);
            if let Err(err_msg) = CgroupManager::attach_service(&service_name, child.id()) {
                eprintln!(
                    "[rid-cgroups] failed to attach service '{}': {}",
                    service_name, err_msg
                );
            }

            started_services.push((child.id(), path));
            std::mem::forget(child);
        }
    }
    started_services
}

fn main() {
    let in_test = false;

    mount_important_filesystems();

    let mut sigset = SigSet::empty();
    sigset.add(Signal::SIGCHLD);
    sigset.thread_block().unwrap();

    if in_test {
        std::mem::forget(Command::new("/bin/sh").spawn().unwrap());
    }

    let service_pids = service_starter();

    loop {
        let sig = sigset.wait().unwrap();

        if sig == Signal::SIGCHLD {
            reap_children(service_pids.clone());
        }
    }
}

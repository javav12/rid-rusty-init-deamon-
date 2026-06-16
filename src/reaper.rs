/*
 * RID - A Rust-based init system.
 * Copyright (C) 2026  javav
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::cgroup_manager::CgroupManager;
use crate::services::extract_service_name;
use json::parse;
use nix::sys::wait::{WaitPidFlag, waitpid};
use nix::unistd::Pid;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Reap terminated child processes and restart configured services when needed.
///
/// This function checks all child processes using non-blocking `waitpid` and
/// restarts service entries whose process IDs have exited or been signaled.
pub fn reap_children(service_list: &mut Vec<(u32, PathBuf)>, stop: &bool) {
    while let Ok(status) = waitpid(Pid::from_raw(-1), Some(WaitPidFlag::WNOHANG))
        && !stop
    {
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

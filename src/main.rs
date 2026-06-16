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

use nix::sys::signal::{SigSet, Signal};
use nix::sys::signalfd::SignalFd;
use std::process::Command;
mod cgroup_manager;
mod utils;
use utils::mount_important_filesystems;
mod reaper;
use reaper::reap_children;
mod services;
use services::auto_service_starter;

fn main() {
    let in_test: bool = false;
    let stop: bool = false;
    mount_important_filesystems();

    let mut sig_mask = SigSet::empty();
    sig_mask.add(Signal::SIGCHLD);
    sig_mask.thread_block().unwrap();

    let sfd = SignalFd::new(&sig_mask).unwrap();

    if in_test {
        std::mem::forget(Command::new("/bin/sh").spawn().unwrap());
    }

    let mut service_pids = auto_service_starter();

    loop {
        if let Some(sig_info) = sfd.read_signal().ok().flatten()
            && let Ok(sig) = Signal::try_from(sig_info.ssi_signo as i32)
        {
            match sig {
                Signal::SIGCHLD => {
                    if !stop {
                        reap_children(&mut service_pids, &stop)
                    }
                }
                _ => {}
            }
        }
    }
}

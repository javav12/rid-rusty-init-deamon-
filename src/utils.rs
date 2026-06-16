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

use std::fs;
/// Mount the essential pseudo-filesystems required by the initramfs environment.
///
/// This is needed before spawning any shell or service processes so that `/proc`,
/// `/sys`, and `/dev` are available to programs started later.
use std::process::Command;

pub fn mount_important_filesystems() {
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

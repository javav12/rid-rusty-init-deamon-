/*
 * RID(cgroup_manager) - A Rust-based init system.
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

// src/cgroup_manager.rs

use std::fs;
use std::path::Path;
pub struct CgroupManager;

impl CgroupManager {
    /// Create a cgroup v2 group using the Linux cgroup filesystem and attach the PID.
    pub fn attach_service(service_name: &str, pid: u32) -> Result<(), String> {
        let base_dir = "/sys/fs/cgroup";
        let target_dir = format!("{}/{}", base_dir, service_name);
        let target_path = Path::new(&target_dir);

        // Create the target service cgroup directory if it does not exist.
        if !target_path.exists()
            && let Err(e) = fs::create_dir_all(target_path)
        {
            return Err(format!(
                "Failed to create cgroup directory ({}): {}",
                target_dir, e
            ));
        }

        // Write the PID into the cgroup.procs file for the group.
        let procs_file = target_path.join("cgroup.procs");

        // Writing the PID to cgroup.procs moves the process into this cgroup.
        if let Err(e) = fs::write(&procs_file, pid.to_string()) {
            return Err(format!(
                "Failed to write PID ({}) to '{}': {}",
                pid,
                procs_file.display(),
                e
            ));
        }

        Ok(())
    }
}

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
use glob::glob;
use json::parse;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn extract_service_name(path: &std::path::Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("unknown-service")
        .to_owned()
}

/// Start all service definitions found in `/etc/rid/services/`.
///
/// Each JSON service file should contain a `command` string and an optional `args`
/// array. Failed files are skipped silently.

pub fn auto_service_starter() -> Vec<(u32, PathBuf)> {
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

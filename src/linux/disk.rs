//
// Sysinfo
//
// Copyright (c) 2017 Guillaume Gomez
//

use super::system::get_all_data;
use utils;
use DiskExt;
use DiskType;

use libc::statvfs;
use std::ffi::{OsStr, OsString};
use std::fmt::{Debug, Error, Formatter};
use std::mem;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

fn find_type_for_name(name: &OsStr) -> DiskType {
    /* turn "sda1" into "sda": */
    let mut trimmed: &[u8] = name.as_bytes();
    while trimmed.len() > 1
        && trimmed[trimmed.len() - 1] >= b'0'
        && trimmed[trimmed.len() - 1] <= b'9'
    {
        trimmed = &trimmed[..trimmed.len() - 1]
    }
    let trimmed: &OsStr = OsStrExt::from_bytes(trimmed);

    let path = Path::new("/sys/block/")
        .to_owned()
        .join(trimmed)
        .join("queue/rotational");
    // Normally, this file only contains '0' or '1' but just in case, we get 8 bytes...
    let rotational_int = get_all_data(path, 8).unwrap_or_default().trim().parse();
    DiskType::from(rotational_int.unwrap_or(-1))
}

macro_rules! cast {
    ($x:expr) => {
        u64::from($x)
    };
}

pub fn new(name: &OsStr, mount_point: &Path, file_system: &[u8]) -> Disk {
    let mount_point_cpath = utils::to_cpath(mount_point);
    let type_ = find_type_for_name(name);
    let mut total = 0;
    let mut available = 0;
    unsafe {
        let mut stat: statvfs = mem::zeroed();
        if statvfs(mount_point_cpath.as_ptr() as *const _, &mut stat) == 0 {
            total = cast!(stat.f_bsize) * cast!(stat.f_blocks);
            available = cast!(stat.f_bsize) * cast!(stat.f_bavail);
        }
    }
    Disk {
        type_,
        name: name.to_owned(),
        file_system: file_system.to_owned(),
        mount_point: mount_point.to_owned(),
        total_space: cast!(total),
        available_space: cast!(available),
    }
}

/// Struct containing a disk information.
pub struct Disk {
    type_: DiskType,
    name: OsString,
    file_system: Vec<u8>,
    mount_point: PathBuf,
    total_space: u64,
    available_space: u64,
}

impl Debug for Disk {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(
            fmt,
            "Disk({:?})[FS: {:?}][Type: {:?}] mounted on {:?}: {}/{} B",
            self.get_name(),
            self.get_file_system(),
            self.get_type(),
            self.get_mount_point(),
            self.get_available_space(),
            self.get_total_space()
        )
    }
}

impl DiskExt for Disk {
    fn get_type(&self) -> DiskType {
        self.type_
    }

    fn get_name(&self) -> &OsStr {
        &self.name
    }

    fn get_file_system(&self) -> &[u8] {
        &self.file_system
    }

    fn get_mount_point(&self) -> &Path {
        &self.mount_point
    }

    fn get_total_space(&self) -> u64 {
        self.total_space
    }

    fn get_available_space(&self) -> u64 {
        self.available_space
    }

    fn refresh(&mut self) -> bool {
        unsafe {
            let mut stat: statvfs = mem::zeroed();
            let mount_point_cpath = utils::to_cpath(&self.mount_point);
            if statvfs(mount_point_cpath.as_ptr() as *const _, &mut stat) == 0 {
                let tmp = cast!(stat.f_bsize) * cast!(stat.f_bavail);
                self.available_space = cast!(tmp);
                true
            } else {
                false
            }
        }
    }
}

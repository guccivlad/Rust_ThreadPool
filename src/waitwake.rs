#[cfg(not(target_os = "linux"))]
compile_error!("Linux only");

use std::sync::atomic::AtomicU32;

use libc::{self};

pub fn wait(uaddr: &AtomicU32, expected: u32) {
    unsafe {
        libc::syscall(
            libc::SYS_futex,
            uaddr as *const AtomicU32,
            libc::FUTEX_WAIT,
            expected,
            std::ptr::null::<libc::timespec>(),
        );
    }
}

pub fn wake_one(uaddr: &AtomicU32) {
    unsafe {
        libc::syscall(
            libc::SYS_futex,
            uaddr as *const AtomicU32,
            libc::FUTEX_WAKE,
            1,
        );
    }
}

pub fn wake_all(uaddr: &AtomicU32) {
    unsafe {
        libc::syscall(
            libc::SYS_futex,
            uaddr as *const AtomicU32,
            libc::FUTEX_WAKE,
            i32::MAX,
        );
    }
}
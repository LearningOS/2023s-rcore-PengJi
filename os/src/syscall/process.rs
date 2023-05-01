//! Process management syscalls
use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE}, 
    mm::{virtaddr2phyaddr, VirtAddr},
    task::{
        change_program_brk, exit_current_and_run_next, get_current_task_start_time, get_syscall_times,
        mmap, munmap, suspend_current_and_run_next, TaskStatus,
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let virt_us: VirtAddr = (ts as usize).into();
    if let Some(pa) = virtaddr2phyaddr(virt_us) {
        let us = get_time_us();
        let phy_ts = pa.0 as *mut TimeVal;
        unsafe {
            *phy_ts = TimeVal {
                sec: us / 1_000_000,
                usec: us % 1_000_000,
            };
        }
        0
    } else {
        -1
    }
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let virt_ti: VirtAddr = (ti as usize).into();
    if let Some(pa) = virtaddr2phyaddr(virt_ti) {
        let phy_ti = pa.0 as *mut TaskInfo;
        unsafe {
            *phy_ti = TaskInfo {
                status: TaskStatus::Running,
                syscall_times: get_syscall_times(),
                time: (get_time_us() - get_current_task_start_time()) / 1_000,
            };
        }
        0
    } else {
        -1
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    if start % 0x1000 != 0 {
        return -1;
    }
    if port & !0x7 != 0 || port == 0 {
        return -1;
    }
    let mut ll = len;
    if len % 0x1000 != 0 {
        ll = (len / PAGE_SIZE + 1) * PAGE_SIZE;
    }

    mmap(start, ll, port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    if start % PAGE_SIZE != 0 || len % PAGE_SIZE != 0 {
        return -1;
    }
    munmap(start, len)
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

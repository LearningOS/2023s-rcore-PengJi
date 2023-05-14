//! Weak Bank Detector

use alloc::vec::Vec;

#[derive(Clone)]
/// ResourceList
pub struct ResourceList {
    /// available
    pub available: Vec<i32>,
    /// allocation
    pub allocation: Vec<Vec<usize>>,
    /// need
    pub need: Vec<Vec<usize>>,
}

impl ResourceList {
    /// new
    pub fn new() -> Self {
        Self {
            available: Vec::new(),
            allocation: Vec::new(),
            need: Vec::new(),
        }
    }

    /// init_size
    pub fn init_size(&mut self, size: usize, rid: usize) {
        if rid >= self.available.len() {
            let n = rid - self.available.len();
            for _ in 0..n {
                self.available.push(0);
            }
            self.available.push(size as i32);
        } else {
            self.available[rid] = size as i32;
        }
    }

    /// alloc_one
    pub fn alloc(&mut self, size: usize, rid: usize, tid: usize) {
        debug!("{} alloc {}, size: {}", tid, rid, size);
        if tid >= self.allocation.len() {
            let n = tid - self.allocation.len();
            for _ in 0..=n {
                self.allocation.push(Vec::new());
            }
        }
        if rid >= self.allocation[tid].len() {
            let n = rid - self.allocation[tid].len();
            for _ in 0..=n {
                self.allocation[tid].push(0);
            }
        }

        debug!("before alloc: allocated: {:?}", self.allocation[tid]);
        self.allocation[tid][rid] += size;
        self.available[rid] -= size as i32;
        self.need[tid][rid] -= size;
        debug!("after alloc: allocated: {:?}", self.allocation[tid]);
    }

    /// release
    pub fn release(&mut self, rid: usize, size: usize, tid: usize) {
        debug!("{} release {} size: {}", tid, rid, size);

        debug!("before release, allocated: {:?}", self.allocation[tid]);
        self.available[rid] += size as i32;
        self.allocation[tid][rid] -= size;
        debug!("after release, allocated: {:?}", self.allocation[tid]);
    }

    /// is_enough
    pub fn is_enough(&self, rid: usize, size: usize) -> bool {
        if self.available[rid] < size as i32 {
            return false;
        }
        true
    }

    /// is_dead_lock
    pub fn is_dead_lock(&mut self, tid: usize, rid: usize, size: usize, finish: Vec<bool>) -> bool {
        debug!("{} check deadlock, request {}", tid, rid);
        if tid >= self.need.len() {
            let n = tid - self.need.len();
            for _ in 0..=n {
                self.need.push(Vec::new());
            }
        }
        if rid >= self.need[tid].len() {
            let n = rid - self.need[tid].len();
            for _ in 0..=n {
                self.need[tid].push(0);
            }
        }

        debug!(
            "before {} add need: \navail: {:?}, allocated: {:?}, need: {:?}, finish: {:?}",
            tid, self.available, self.allocation, self.need, finish
        );
        self.need[tid][rid] = size;
        if tid != 0 && rid == 0 {
            return false;
        }
        if self.is_enough(rid, size) {
            return false;
        }

        let not_finish = finish.iter().any(|t| *t == false);
        if !not_finish {
            return false;
        }
        let mut finish = finish;
        let allocation = self.allocation.clone();
        let mut avail = self.available.clone();
        loop {
            let all_finish = finish.iter().all(|t| *t == true);
            if all_finish {
                return false;
            }
            let mut cnt = 0;
            for i in 0..finish.len() {
                if i >= self.need.len() {
                    break;
                }
                if finish[i] {
                    continue;
                }
                let enough_i = self.need[i]
                    .iter()
                    .enumerate()
                    .all(|(idx, &t)| avail[idx] >= t as i32);
                if enough_i {
                    avail.iter_mut().enumerate().for_each(|(idx, n)| {
                        if idx < allocation[i].len() {
                            *n += allocation[i][idx] as i32;
                        }
                    });
                    finish[i] = true;
                    cnt += 1;
                }
            }
            if cnt == 0 {
                return true;
            }
        }
    }
}

#[derive(Clone)]
///Detector
pub struct Detector {
    /// ResourceList
    pub mutexes: ResourceList,
    /// ResourceList
    pub semes: ResourceList,
}

impl Detector {
    /// new
    pub fn new() -> Self {
        Self {
            mutexes: ResourceList::new(),
            semes: ResourceList::new(),
        }
    }

    /// create_mutex
    pub fn create_mutex(&mut self, mid: usize) {
        self.mutexes.init_size(1, mid);
    }

    /// alloc_mutex
    pub fn alloc_mutex(&mut self, tid: usize, mid: usize) {
        self.mutexes.alloc(1, mid, tid);
    }

    /// release_mutex
    pub fn release_mutex(&mut self, tid: usize, mid: usize) {
        self.mutexes.release(mid, 1, tid);
    }

    /// check_mutex
    pub fn check_mutex(&mut self, tid: usize, mid: usize, task_set: Vec<bool>) -> bool {
        self.mutexes.is_dead_lock(tid, mid, 1, task_set)
    }

    /// create_sem
    pub fn create_semaphore(&mut self, sid: usize, size: usize) {
        self.semes.init_size(size, sid);
    }

    /// alloc_semaphore
    pub fn alloc_semaphore(&mut self, tid: usize, sid: usize) {
        self.semes.alloc(1, sid, tid);
    }

    /// release_semaphore
    pub fn release_semaphore(&mut self, tid: usize, sid: usize) {
        self.semes.release(sid, 1, tid)
    }

    /// check_semaphore
    pub fn check_semaphore(&mut self, tid: usize, sid: usize, task_set: Vec<bool>) -> bool {
        self.semes.is_dead_lock(tid, sid, 1, task_set)
    }
}

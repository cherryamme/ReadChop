use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use log::info;

/// 线程池管理器
pub struct ThreadPoolManager {
    /// 总线程数限制
    max_threads: usize,
    /// 当前活跃线程数
    active_threads: Arc<AtomicUsize>,
    /// 线程句柄存储
    _thread_handles: Vec<thread::JoinHandle<()>>,
}

impl ThreadPoolManager {
    /// 创建新的线程池管理器
    pub fn new(max_threads: usize) -> Self {
        info!("创建线程池管理器，最大线程数: {}", max_threads);
        Self {
            max_threads,
            active_threads: Arc::new(AtomicUsize::new(0)),
            _thread_handles: Vec::new(),
        }
    }

    /// 获取当前活跃线程数
    pub fn get_active_thread_count(&self) -> usize {
        self.active_threads.load(Ordering::Relaxed)
    }

    /// 获取剩余可用线程数
    pub fn get_available_threads(&self) -> usize {
        let active = self.active_threads.load(Ordering::Relaxed);
        if active < self.max_threads {
            self.max_threads - active
        } else {
            0
        }
    }

    /// 检查是否可以创建新线程
    pub fn can_spawn_thread(&self) -> bool {
        self.active_threads.load(Ordering::Relaxed) < self.max_threads
    }

    /// 分配线程资源
    pub fn allocate_threads(&self, requested_threads: usize) -> usize {
        let available = self.get_available_threads();
        let allocated = std::cmp::min(requested_threads, available);
        if allocated > 0 {
            self.active_threads.fetch_add(allocated, Ordering::Relaxed);
        }
        allocated
    }

    /// 释放线程资源
    pub fn release_threads(&self, count: usize) {
        if count > 0 {
            self.active_threads.fetch_sub(count, Ordering::Relaxed);
        }
    }

    /// 创建受控的线程
    pub fn spawn_controlled_thread<F, T>(&mut self, f: F) -> Option<thread::JoinHandle<T>>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        if !self.can_spawn_thread() {
            return None;
        }

        self.active_threads.fetch_add(1, Ordering::Relaxed);
        let active_threads = Arc::clone(&self.active_threads);
        
        let handle = thread::spawn(move || {
            let result = f();
            active_threads.fetch_sub(1, Ordering::Relaxed);
            result
        });

        Some(handle)
    }

    /// 等待所有线程完成
    pub fn wait_for_completion(&mut self) {
        for handle in self._thread_handles.drain(..) {
            handle.join().expect("线程执行失败");
        }
    }

    /// 获取线程使用统计
    pub fn get_thread_stats(&self) -> (usize, usize, usize) {
        let active = self.active_threads.load(Ordering::Relaxed);
        let available = self.get_available_threads();
        (self.max_threads, active, available)
    }
}

/// 线程分配策略
pub enum ThreadAllocationStrategy {
    /// 均衡分配：处理线程和写入线程按比例分配
    Balanced {
        processing_ratio: f32,  // 处理线程占比 (0.0-1.0)
    },
    /// 优先级分配：优先保证处理线程，剩余给写入线程
    Priority {
        min_processing_threads: usize,  // 最少处理线程数
    },
    /// 固定分配：固定数量的处理线程和写入线程
    Fixed {
        processing_threads: usize,
        writing_threads: usize,
    },
}

impl ThreadAllocationStrategy {
    /// 计算线程分配
    pub fn calculate_allocation(&self, total_threads: usize) -> (usize, usize) {
        match self {
            ThreadAllocationStrategy::Balanced { processing_ratio } => {
                let processing_threads = (total_threads as f32 * processing_ratio) as usize;
                let writing_threads = total_threads - processing_threads;
                (processing_threads.max(1), writing_threads)
            }
            ThreadAllocationStrategy::Priority { min_processing_threads } => {
                let processing_threads = std::cmp::max(*min_processing_threads, total_threads / 2);
                let writing_threads = total_threads - processing_threads;
                (processing_threads, writing_threads)
            }
            ThreadAllocationStrategy::Fixed { processing_threads, writing_threads } => {
                (*processing_threads, *writing_threads)
            }
        }
    }
}

/// 线程使用监控器
pub struct ThreadMonitor {
    thread_pool: ThreadPoolManager,
    _allocation_strategy: ThreadAllocationStrategy,
    processing_threads: usize,
    writing_threads: usize,
}

impl ThreadMonitor {
    /// 创建新的线程监控器
    pub fn new(total_threads: usize, strategy: ThreadAllocationStrategy) -> Self {
        let (processing_threads, writing_threads) = strategy.calculate_allocation(total_threads);
        
        info!(
            "线程分配策略: 总线程数={}, 处理线程={}, 写入线程={}", 
            total_threads, processing_threads, writing_threads
        );

        Self {
            thread_pool: ThreadPoolManager::new(total_threads),
            _allocation_strategy: strategy,
            processing_threads,
            writing_threads,
        }
    }

    /// 获取处理线程数
    pub fn get_processing_threads(&self) -> usize {
        self.processing_threads
    }

    /// 获取写入线程数
    pub fn get_writing_threads(&self) -> usize {
        self.writing_threads
    }

    /// 获取线程池管理器
    pub fn get_thread_pool(&mut self) -> &mut ThreadPoolManager {
        &mut self.thread_pool
    }

    /// 打印线程使用统计
    pub fn print_thread_stats(&self) {
        let (max, active, available) = self.thread_pool.get_thread_stats();
        info!(
            "线程使用统计: 最大={}, 活跃={}, 可用={}, 处理线程={}, 写入线程={}",
            max, active, available, self.processing_threads, self.writing_threads
        );
    }
}

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use log::info;

/// Thread pool manager
pub struct ThreadPoolManager {
    /// Maximum thread count limit
    max_threads: usize,
    /// Current active thread count
    active_threads: Arc<AtomicUsize>,
    /// Thread handle storage
    _thread_handles: Vec<thread::JoinHandle<()>>,
}

impl ThreadPoolManager {
    /// Create new thread pool manager
    pub fn new(max_threads: usize) -> Self {
        // info!("Creating thread pool manager, max threads: {}", max_threads);
        Self {
            max_threads,
            active_threads: Arc::new(AtomicUsize::new(0)),
            _thread_handles: Vec::new(),
        }
    }


    /// Get remaining available thread count
    pub fn get_available_threads(&self) -> usize {
        let active = self.active_threads.load(Ordering::Relaxed);
        if active < self.max_threads {
            self.max_threads - active
        } else {
            0
        }
    }

    /// Check if new thread can be created
    pub fn can_spawn_thread(&self) -> bool {
        self.active_threads.load(Ordering::Relaxed) < self.max_threads
    }

    /// Allocate thread resources
    pub fn allocate_threads(&self, requested_threads: usize) -> usize {
        let available = self.get_available_threads();
        let allocated = std::cmp::min(requested_threads, available);
        if allocated > 0 {
            self.active_threads.fetch_add(allocated, Ordering::Relaxed);
        }
        allocated
    }

    /// Release thread resources
    pub fn release_threads(&self, count: usize) {
        if count > 0 {
            self.active_threads.fetch_sub(count, Ordering::Relaxed);
        }
    }

    /// Create controlled thread
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

    /// Wait for all threads to complete

    /// Get thread usage statistics
    pub fn get_thread_stats(&self) -> (usize, usize, usize) {
        let active = self.active_threads.load(Ordering::Relaxed);
        let available = self.get_available_threads();
        (self.max_threads, active, available)
    }
}

/// Thread allocation strategy
pub enum ThreadAllocationStrategy {
    /// Balanced allocation: processing and writing threads allocated by ratio
    Balanced {
        processing_ratio: f32,  // Processing thread ratio (0.0-1.0)
    },
}

impl ThreadAllocationStrategy {
    /// Calculate thread allocation
    pub fn calculate_allocation(&self, total_threads: usize) -> (usize, usize) {
        match self {
            ThreadAllocationStrategy::Balanced { processing_ratio } => {
                let processing_threads = (total_threads as f32 * processing_ratio) as usize;
                let writing_threads = total_threads - processing_threads;
                (processing_threads.max(1), writing_threads)
            }
        }
    }
}

/// Thread usage monitor
pub struct ThreadMonitor {
    thread_pool: ThreadPoolManager,
    _allocation_strategy: ThreadAllocationStrategy,
    processing_threads: usize,
    writing_threads: usize,
}

impl ThreadMonitor {
    /// Create new thread monitor
    pub fn new(total_threads: usize, strategy: ThreadAllocationStrategy) -> Self {
        let (processing_threads, writing_threads) = strategy.calculate_allocation(total_threads);
        
        info!(
            "Thread allocation strategy: total_threads={}, processing_threads={}, writing_threads={}", 
            total_threads, processing_threads, writing_threads
        );

        Self {
            thread_pool: ThreadPoolManager::new(total_threads),
            _allocation_strategy: strategy,
            processing_threads,
            writing_threads,
        }
    }

    /// Get processing thread count
    pub fn get_processing_threads(&self) -> usize {
        self.processing_threads
    }

    /// Get writing thread count
    pub fn get_writing_threads(&self) -> usize {
        self.writing_threads
    }

    /// Get thread pool manager
    pub fn get_thread_pool(&mut self) -> &mut ThreadPoolManager {
        &mut self.thread_pool
    }

    /// Print thread usage statistics
    pub fn print_thread_stats(&self) {
        let (max, active, available) = self.thread_pool.get_thread_stats();
        info!(
            "Thread usage statistics: max={}, active={}, available={}, processing_threads={}, writing_threads={}",
            max, active, available, self.processing_threads, self.writing_threads
        );
    }
}

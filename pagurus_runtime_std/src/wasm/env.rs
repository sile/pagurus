use std::sync::{Arc, Mutex};
use wasmer::{LazyInit, Memory, WasmerEnv};

#[derive(Debug, WasmerEnv)]
pub struct Env<S> {
    system: Arc<Mutex<*mut S>>,

    #[wasmer(export)]
    memory: LazyInit<Memory>,
}

impl<S> Env<S> {
    pub fn new() -> Self {
        Self {
            system: Arc::new(Mutex::new(std::ptr::null_mut())),
            memory: Default::default(),
        }
    }

    pub fn set_system(&mut self, system: &mut S) {
        let mut env_system = self
            .system
            .lock()
            .unwrap_or_else(|e| panic!("failed to acquire lock: {e}"));
        *env_system = system as *mut _;
    }

    pub fn with_system<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut S) -> T,
    {
        let mut system = self.system.lock().expect("failed to acquire lock");
        f(unsafe { &mut **system })
    }

    pub fn with_system_and_memory<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut S, &Memory) -> T,
    {
        let mut system = self.system.lock().expect("failed to acquire lock");
        let memory = self.memory_ref().expect("memory is not initialized");
        f(unsafe { &mut **system }, memory)
    }
}

impl<S> Clone for Env<S> {
    fn clone(&self) -> Self {
        Self {
            system: self.system.clone(),
            memory: self.memory.clone(),
        }
    }
}

unsafe impl<S> Send for Env<S> {}

unsafe impl<S> Sync for Env<S> {}

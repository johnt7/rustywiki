
use std::{
    collections::HashMap,
    ops::Deref,
    sync::{RwLock}
};


/// Structure passed to Rocket to store page locks
pub struct PageMap ( RwLock<HashMap<String, String>> );
impl Deref for PageMap {
    type Target = RwLock<HashMap<String, String>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl PageMap {
    pub fn new() -> PageMap {
        PageMap ( RwLock::new(HashMap::new()) )
    }
}


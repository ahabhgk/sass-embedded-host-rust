use std::{collections::HashSet, sync::Mutex};

/// Manages pending inbound and outbound requests. Ensures that requests and
/// responses interact correctly and obey the Embedded Protocol.
pub struct RequestTracker {
  /// The indices of this array correspond to each pending request's ID.
  ids: Mutex<HashSet<u32>>,
}

impl RequestTracker {
  pub fn new() -> Self {
    Self {
      ids: Mutex::new(HashSet::new()),
    }
  }

  /// The next available request ID.
  pub fn next_id(&self) -> u32 {
    let mut id = 0;
    let ids = self.ids.lock().unwrap();
    while ids.contains(&id) {
      id += 1;
    }
    id
  }

  /// Adds an entry for a pending request with ID `id`. Panics if the Protocol
  /// Error is violated.
  pub fn add(&self, id: u32) {
    let mut ids = self.ids.lock().unwrap();
    if ids.contains(&id) {
      panic!("Request ID {id} is already in use by an in-flight request.");
    }
    ids.insert(id);
  }

  /// Resolves a pending request with matching ID `id`. Panics if the Protocol
  /// Error is violated.
  pub fn resolve(&self, id: u32) {
    let mut ids = self.ids.lock().unwrap();
    if !ids.contains(&id) {
      panic!("Response ID {id} does not match any pending requests.");
    }
    ids.remove(&id);
  }
}

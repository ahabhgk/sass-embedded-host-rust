use std::collections::HashSet;

/// Manages pending inbound and outbound requests. Ensures that requests and
/// responses interact correctly and obey the Embedded Protocol.
pub struct RequestTracker {
  /// The indices of this array correspond to each pending request's ID.
  ids: HashSet<u32>,
}

impl RequestTracker {
  pub fn new() -> Self {
    Self {
      ids: HashSet::new(),
    }
  }

  /// The next available request ID.
  pub fn next_id(&self) -> u32 {
    let mut id = 0;
    while self.ids.contains(&id) {
      id += 1;
    }
    id
  }

  /// Adds an entry for a pending request with ID `id`. Panics if the Protocol
  /// Error is violated.
  pub fn add(&mut self, id: u32) {
    if self.ids.contains(&id) {
      panic!("Request ID {id} is already in use by an in-flight request.");
    }
    self.ids.insert(id);
  }

  /// Resolves a pending request with matching ID `id`. Panics if the Protocol
  /// Error is violated.
  pub fn resolve(&mut self, id: u32) {
    if !self.ids.contains(&id) {
      panic!("Response ID {id} does not match any pending requests.");
    }
    self.ids.remove(&id);
  }
}

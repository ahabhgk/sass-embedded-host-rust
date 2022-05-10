/// Manages pending inbound and outbound requests. Ensures that requests and
/// responses interact correctly and obey the Embedded Protocol.
pub struct RequestTracker {
  /// The indices of this array correspond to each pending request's ID.
  requests: Vec<bool>,
}

impl RequestTracker {
  pub fn new() -> Self {
    Self {
      requests: Vec::new(),
    }
  }

  /// The next available request ID.
  pub fn next_id(&mut self) -> u32 {
    for (i, v) in self.requests.iter().enumerate() {
      if !v {
        return i as u32;
      }
    }
    self.requests.push(false);
    self.requests.len() as u32 - 1
  }

  /// Adds an entry for a pending request with ID `id`. Panics if the Protocol
  /// Error is violated.
  pub fn add(&mut self, id: u32) {
    if self.requests[id as usize] {
      panic!("Request ID {id} is already in use by an in-flight request.");
    }
    self.requests[id as usize] = true;
  }

  /// Resolves a pending request with matching ID `id`. Panics if the Protocol
  /// Error is violated.
  pub fn resolve(&mut self, id: u32) {
    if !self.requests[id as usize] {
      panic!("Response ID {id} does not match any pending requests.");
    }
    self.requests[id as usize] = false;
  }
}

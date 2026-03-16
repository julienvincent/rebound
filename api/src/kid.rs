use uuid::Uuid;

/// Generate a new KID: a base58-encoded UUIDv7.
///
/// KIDs are compact, URL-safe, and lexicographically sortable by generation
/// time since UUIDv7 embeds a millisecond timestamp in its most significant
/// bits and base58 preserves the byte-order sort property.
pub fn generate() -> String {
  let uuid = Uuid::now_v7();
  bs58::encode(uuid.as_bytes()).into_string()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn generates_non_empty() {
    let id = generate();
    assert!(!id.is_empty());
  }

  #[test]
  fn maintains_sort_order() {
    let ids: Vec<String> = (0..1000).map(|_| generate()).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted);
  }

  #[test]
  fn consistent_length() {
    let ids: Vec<String> = (0..100).map(|_| generate()).collect();
    let first_len = ids[0].len();
    assert!(
      ids.iter().all(|id| id.len() == first_len),
      "KIDs should have consistent length, got varying lengths"
    );
  }
}

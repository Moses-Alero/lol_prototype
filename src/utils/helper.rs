pub fn remove_index<T: Clone>(arr: &[T], index: usize) -> Vec<T> {
    let (head, tail) = arr.split_at(index);
    // `tail` starts at the index we want to remove, so we skip the first element of `tail`
    let mut new_vec = head.to_vec();
    new_vec.extend_from_slice(&tail[1..]);
    new_vec
}

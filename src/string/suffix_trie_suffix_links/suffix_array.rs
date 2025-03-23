use crate::alphabet_model::CharT;
use crate::string_model::AStr;
use alloc::borrow::Cow;

#[derive(Debug)]
pub struct SuffixArray<'s, C: CharT> {
    sorted_suffixes: Vec<usize>,
    s: Cow<'s, AStr<C>>,
}

// pub fn build_trie<'s, C: CharT>(s: Cow<'s, AStr<C>>) -> SuffixArray<'s, C, alloc::Global> {
//     build_trie_with_allocator(s, alloc::Global)
// }

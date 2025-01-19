use serde::Deserialize;
use serde::Serialize;

use crate::word_id::{WordId, WordIdSet};
use std::cmp::Ordering;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Debug, Eq)]
struct Item<'a> {
    arr: &'a Vec<usize>,
    idx: usize,
    pub word_id: WordId,
}

impl<'a> Item<'a> {
    fn new(arr: &'a Vec<usize>, idx: usize, word_id: WordId) -> Self {
        Self { arr, idx, word_id }
    }

    #[inline]
    fn get_item(&self) -> usize {
        self.arr[self.idx]
    }

    #[inline]
    fn get_pair(&self) -> (usize, WordId) {
        (self.arr[self.idx], self.word_id)
    }
}

impl<'a> PartialEq for Item<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.get_item() == other.get_item()
    }
}

impl<'a> PartialOrd for Item<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get_item().partial_cmp(&other.get_item())
    }
}

impl<'a> Ord for Item<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_item().cmp(&other.get_item())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WordSegmentRange {
    /// This is input word count * closeness range
    pub min: usize,
    pub max: usize,
    pub elements: Vec<usize>,
    pub set: WordIdSet,
}

impl WordSegmentRange {
    pub fn new(first_element: usize, word_id: WordId) -> Self {
        Self {
            min: first_element,
            max: first_element,
            elements: vec![first_element],
            set: WordIdSet::new(word_id),
        }
    }

    pub fn total_range(&self) -> usize {
        self.max - self.min
    }

    pub fn add(&mut self, element: usize, allowed_range: usize) -> bool {
        let can_add = self.can_add(element, allowed_range);
        if can_add {
            self.elements.push(element);
            if element < self.min {
                self.min = element;
            } else if element > self.max {
                self.max = element;
            }
        }
        can_add
    }

    /// return whether or not the value can be added
    pub fn can_add(&self, element: usize, allowed_range: usize) -> bool {
        if element < self.min {
            let lowest_min = self.max - allowed_range;
            element > lowest_min
        } else {
            let highest_max = self.min + allowed_range;
            element < highest_max
        }
    }
}

pub fn merge_special(arrays: Vec<Vec<usize>>, allowed_range: usize) -> Vec<WordSegmentRange> {
    let mut sorted: Vec<WordSegmentRange> = vec![];

    let mut heap = BinaryHeap::with_capacity(arrays.len());
    for (idx, arr) in arrays.iter().enumerate() {
        let item = Item::new(arr, 0, WordId::from_index(idx));
        heap.push(Reverse(item));
    }

    while !heap.is_empty() {
        let mut it = heap.pop().unwrap();
        let this = &mut it.0;
        let (this_index, word_id) = this.get_pair();

        // ! evaluate what this is doing
        let mut at_least_one_added = false;
        // while i can add to elements of the end of the array, do it!
        for range in sorted.iter_mut().rev() {
            // try adding to the segment range
            if range.add(this_index, allowed_range) {
                // then add the word id to the set to keep track of unique elements
                range.set.add(word_id);
                at_least_one_added = true;
            } else {
                break;
            }
        }

        // if the `next element` can't reach the `last element`, but the `next element` could reach `this element`, then i add `this element`
        if let Some(last) = sorted.last() {
            if let Some(next) = heap.peek() {
                let next_index = next.0.get_item();
                let next_and_last_cant_reach = !last.can_add(next_index, allowed_range);
                let this_and_last_can_reach = next_index.abs_diff(this_index) <= allowed_range;
                // seriously, what is the !at_least_one_added doing here?
                if (next_and_last_cant_reach && this_and_last_can_reach) || !at_least_one_added {
                    sorted.push(WordSegmentRange::new(this_index, word_id));
                }
            } else {
                if !at_least_one_added {
                    sorted.push(WordSegmentRange::new(this_index, word_id));
                }
            }
        }
        // the base case at the beginning with no last element
        else {
            sorted.push(WordSegmentRange::new(this_index, word_id));
        }

        // advance to next word id for the heap
        this.idx += 1;
        // if elements remain, put back in heap
        if it.0.idx < it.0.arr.len() {
            heap.push(it)
        }
    }

    sorted
}

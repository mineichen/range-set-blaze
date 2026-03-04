use crate::map::ValueRef;
use crate::merge_map::KMergeMap;
use crate::sorted_disjoint_map::{Priority, PrioritySortedStartsMap};
use crate::{AssumeSortedStarts, MergeMap, SortedDisjointMap, UnionKMergeMap, UnionMergeMap};
use alloc::{collections::BinaryHeap, vec};
use core::cmp::min;
use crate::NonZeroRange;
use core::iter::FusedIterator;
use itertools::Itertools;

use crate::Integer;
use crate::unsorted_priority_map::AssumePrioritySortedStartsMap;
use crate::unsorted_priority_map::UnsortedPriorityMap;

type SortedStartsInVecMap<T, VR> =
    AssumePrioritySortedStartsMap<T, VR, vec::IntoIter<Priority<T, VR>>>;
#[allow(clippy::redundant_pub_crate)]
pub(crate) type SortedStartsInVec<T> = AssumeSortedStarts<T, vec::IntoIter<NonZeroRange<T>>>;

/// This `struct` is created by the [`union`] method. See [`union`]'s
/// documentation for more.
///
/// [`union`]: crate::SortedDisjointMap::union
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct UnionIterMap<T, VR, SS>
where
    T: Integer,
    VR: ValueRef,
    SS: PrioritySortedStartsMap<T, VR>,
{
    iter: SS,
    next_item: Option<Priority<T, VR>>,
    workspace: BinaryHeap<Priority<T, VR>>,
    gather: Option<(NonZeroRange<T>, VR)>,
    ready_to_go: Option<(NonZeroRange<T>, VR)>,
}

impl<T, VR, I> Iterator for UnionIterMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: PrioritySortedStartsMap<T, VR>,
{
    type Item = (NonZeroRange<T>, VR);

    fn next(&mut self) -> Option<(NonZeroRange<T>, VR)> {
        loop {
            if let Some(value) = self.ready_to_go.take() {
                return Some(value);
            }

            if let Some(next_item) = self.next_item.take() {
                let (next_start, next_end) = next_item.start_and_end();

                let Some(best) = self.workspace.peek() else {
                    self.workspace.push(next_item);
                    self.next_item = self.iter.next();
                    continue;
                };
                if next_start == best.start() {
                    if &next_item > best || next_end > best.end() {
                        self.workspace.push(next_item);
                    }
                    self.next_item = self.iter.next();
                    continue;
                }

                self.next_item = Some(next_item);
            }

            let Some(best) = self.workspace.peek() else {
                debug_assert!(self.next_item.is_none());
                debug_assert!(self.ready_to_go.is_none());
                return self.gather.take();
            };

            let next_end = self.next_item.as_ref().map_or_else(
                || best.end(),
                |next_item| min(next_item.start(), best.end()),
            );

            if let Some(mut gather) = self.gather.take() {
                if gather.1.borrow() == best.value().borrow()
                    && gather.0.end == best.start()
                {
                    gather.0 = unsafe {
                        NonZeroRange::new_unchecked(gather.0.start..next_end)
                    };
                    self.gather = Some(gather);
                } else {
                    self.ready_to_go = Some(gather);
                    self.gather = Some((
                        unsafe { NonZeroRange::new_unchecked(best.start()..next_end) },
                        best.value().clone(),
                    ));
                }
            } else {
                self.gather = Some((
                    unsafe { NonZeroRange::new_unchecked(best.start()..next_end) },
                    best.value().clone(),
                ));
            }

            let mut new_workspace = BinaryHeap::new();
            while let Some(item) = self.workspace.pop() {
                let mut item = item;
                if item.end() <= next_end {
                    continue;
                }
                item.set_range(unsafe {
                    NonZeroRange::new_unchecked(next_end..item.end())
                });
                let Some(new_best) = new_workspace.peek() else {
                    new_workspace.push(item);
                    continue;
                };
                if &item < new_best && item.end() <= new_best.end() {
                    continue;
                }

                new_workspace.push(item);
            }
            self.workspace = new_workspace;
        }
    }
}

impl<T, VR, I> UnionIterMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: PrioritySortedStartsMap<T, VR>,
{
    #[inline]
    pub(crate) fn new(mut iter: I) -> Self {
        let item = iter.next();
        Self {
            iter,
            next_item: item,
            workspace: BinaryHeap::new(),
            gather: None,
            ready_to_go: None,
        }
    }
}

impl<T, VR, L, R> UnionMergeMap<T, VR, L, R>
where
    T: Integer,
    VR: ValueRef,
    L: SortedDisjointMap<T, VR>,
    R: SortedDisjointMap<T, VR>,
{
    #[inline]
    pub(crate) fn new2(left: L, right: R) -> Self {
        let iter = MergeMap::new(left, right);
        Self::new(iter)
    }
}

impl<T, VR, J> UnionKMergeMap<T, VR, J>
where
    T: Integer,
    VR: ValueRef,
    J: SortedDisjointMap<T, VR>,
{
    #[inline]
    pub(crate) fn new_k<K>(k: K) -> Self
    where
        K: IntoIterator<Item = J>,
    {
        let iter = KMergeMap::new(k);
        Self::new(iter)
    }
}

// UnionIterMap from iter (T, VR)
impl<T, VR> FromIterator<(NonZeroRange<T>, VR)>
    for UnionIterMap<T, VR, SortedStartsInVecMap<T, VR>>
where
    T: Integer,
    VR: ValueRef,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (NonZeroRange<T>, VR)>,
    {
        let iter = iter.into_iter();
        let iter = UnsortedPriorityMap::new(iter);
        // We sort only by start -- priority is not used until later.
        let iter = iter.sorted_by(|a, b| a.start().cmp(&b.start()));
        let iter = AssumePrioritySortedStartsMap::new(iter);
        Self::new(iter)
    }
}

impl<T, VR, I> FusedIterator for UnionIterMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: PrioritySortedStartsMap<T, VR> + FusedIterator,
{
}

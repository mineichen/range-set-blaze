use core::{
    cmp::{self, min},
    iter::FusedIterator,
};

use alloc::collections::BinaryHeap;

use crate::{
    Integer, MergeMap, NonZeroRange, SortedDisjointMap, SymDiffKMergeMap, SymDiffMergeMap,
    map::ValueRef,
    merge_map::KMergeMap,
    sorted_disjoint_map::{Priority, PrioritySortedStartsMap},
};

/// This `struct` is created by the [`symmetric_difference`] method on [`SortedDisjointMap`]. See [`symmetric_difference`]'s
/// documentation for more.
///
/// [`SortedDisjointMap`]: crate::SortedDisjointMap
/// [`symmetric_difference`]: crate::SortedDisjointMap::symmetric_difference
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct SymDiffIterMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: PrioritySortedStartsMap<T, VR>,
{
    iter: I,
    next_item: Option<Priority<T, VR>>,
    workspace: BinaryHeap<Priority<T, VR>>,
    workspace_next_end: Option<T>,
    gather: Option<(NonZeroRange<T>, VR)>,
    ready_to_go: Option<(NonZeroRange<T>, VR)>,
}

#[expect(clippy::ref_option)]
fn min_next_end<T>(next_end: &Option<T>, next_item_end: T) -> T
where
    T: Integer,
{
    next_end.map_or_else(
        || next_item_end,
        |current_end| cmp::min(current_end, next_item_end),
    )
}

impl<T, VR, I> FusedIterator for SymDiffIterMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: PrioritySortedStartsMap<T, VR>,
{
}

impl<T, VR, I> Iterator for SymDiffIterMap<T, VR, I>
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
                    self.workspace_next_end =
                        Some(min_next_end(&self.workspace_next_end, next_end));
                    self.workspace.push(next_item);
                    self.next_item = self.iter.next();
                    continue;
                };
                let best = best.range_value();
                if next_start == best.0.start {
                    self.workspace_next_end =
                        Some(min_next_end(&self.workspace_next_end, next_end));
                    self.workspace.push(next_item);
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
            let best = best.range_value();

            let mut next_end = self
                .workspace_next_end
                .take()
                .expect("Real Assert: safe because we know the workspace is not empty");
            if let Some(next_item) = self.next_item.as_ref() {
                next_end = min(next_item.start(), next_end);
            }

            if let Some(mut gather) = self.gather.take() {
                if gather.1.borrow() == best.1.borrow()
                    && gather.0.end == best.0.start
                {
                    if self.workspace.len().is_odd() {
                        gather.0 = unsafe {
                            NonZeroRange::new_unchecked(gather.0.start..next_end)
                        };
                        self.gather = Some(gather);
                    } else {
                        self.ready_to_go = Some(gather);
                        debug_assert!(self.gather.is_none());
                    }
                } else {
                    self.ready_to_go = Some(gather);
                    if self.workspace.len().is_odd() {
                        self.gather = Some((
                            unsafe { NonZeroRange::new_unchecked(best.0.start..next_end) },
                            best.1.clone(),
                        ));
                    } else {
                        debug_assert!(self.gather.is_none());
                    }
                }
            } else {
                if self.workspace.len().is_odd() {
                    self.gather = Some((
                        unsafe { NonZeroRange::new_unchecked(best.0.start..next_end) },
                        best.1.clone(),
                    ));
                } else {
                    debug_assert!(self.gather.is_none());
                }
            }

            self.workspace_next_end = None;
            self.workspace = self
                .workspace
                .drain()
                .filter(|item| item.end() > next_end)
                .map(|mut item| {
                    item.set_range(unsafe {
                        NonZeroRange::new_unchecked(next_end..item.end())
                    });
                    self.workspace_next_end =
                        Some(min_next_end(&self.workspace_next_end, item.end()));
                    item
                })
                .collect();
        }
    }
}

impl<T, VR, L, R> SymDiffMergeMap<T, VR, L, R>
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

impl<T, VR, J> SymDiffKMergeMap<T, VR, J>
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

impl<T, VR, I> SymDiffIterMap<T, VR, I>
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
            workspace_next_end: None,
            gather: None,
            ready_to_go: None,
        }
    }
}

#[allow(clippy::wrong_self_convention)]
#[allow(clippy::redundant_pub_crate)]
pub(crate) trait UsizeExtensions {
    fn is_odd(self) -> bool;
}

impl UsizeExtensions for usize {
    #[inline]
    fn is_odd(self) -> bool {
        self & 1 != 0
    }
}

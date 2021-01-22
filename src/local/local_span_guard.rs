// Copyright 2020 TiKV Project Authors. Licensed under Apache-2.0.

use crate::local::span_line::{LocalSpanHandle, SpanLine, SPAN_LINE};

#[must_use]
pub struct LocalSpanGuard {
    span_handle: Option<LocalSpanHandle>,
}
impl !Send for LocalSpanGuard {}
impl !Sync for LocalSpanGuard {}

impl LocalSpanGuard {
    #[inline]
    pub(crate) fn new(event: &'static str) -> Self {
        SPAN_LINE.with(|span_line| {
            let mut span_line = span_line.borrow_mut();
            let span_handle = span_line.enter_span(event);
            Self { span_handle }
        })
    }

    #[inline]
    pub fn with_properties<I: IntoIterator<Item = (&'static str, String)>, F: FnOnce() -> I>(
        self,
        properties: F,
    ) -> Self {
        self.with_span_line(move |span_handle, span_line| {
            span_line.add_properties(span_handle, properties)
        });
        self
    }

    #[inline]
    pub fn with_property<F: FnOnce() -> (&'static str, String)>(self, property: F) -> Self {
        self.with_span_line(move |span_handle, span_line| {
            span_line.add_property(span_handle, property);
        });
        self
    }
}

impl LocalSpanGuard {
    #[inline]
    fn with_span_line(&self, f: impl FnOnce(&LocalSpanHandle, &mut SpanLine)) {
        if let Some(local_span_handle) = &self.span_handle {
            SPAN_LINE.with(|span_line| {
                let span_line = &mut *span_line.borrow_mut();
                f(local_span_handle, span_line);
            })
        }
    }
}

impl Drop for LocalSpanGuard {
    #[inline]
    fn drop(&mut self) {
        if let Some(span_handle) = self.span_handle.take() {
            SPAN_LINE.with(|span_line| {
                let mut span_line = span_line.borrow_mut();
                span_line.exit_span(span_handle);
            });
        }
    }
}

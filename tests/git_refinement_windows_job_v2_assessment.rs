#![cfg(windows)]

//! P0102/B3 — Windows Job Object v2 assessment.
//!
//! This gate deliberately does not emulate Windows and does not turn a Linux skip or a
//! cross-compile into runtime evidence.  The production API currently exposes no private,
//! testable Job lifecycle seam through which this integration target can inject the three
//! mandatory failures (`CreateJobObject`, configuration and assignment).  Compiling this
//! target on Windows therefore freezes that missing capability as RED instead of silently
//! weakening the assessment or using a production environment variable.
//!
//! Once the L3 seam exists, replacement of this compile-time RED must preserve all of the
//! following runtime cases in this same gate:
//!
//! 1. create and configure a kill-on-close Job before hostile code can execute;
//! 2. associate the leader and every descendant before publishing any content;
//! 3. contain both timeout and early leader exit, including descendants with no pipes;
//! 4. repeat every lifecycle case and return the process handle count to the frozen
//!    baseline tolerance;
//! 5. inject create, configure and assign failures independently and require
//!    `GitRevisionError::ContainmentFailure` with no published bytes;
//! 6. bound every invocation with an independent 15-second watchdog.
//!
//! Runtime classification is intentionally **NOT RUN / BLOCKED** on non-Windows hosts.

compile_error!(
    "P0102/B3 RED: the private testable Windows Job lifecycle seam required for create/configure/assign fault injection is absent"
);

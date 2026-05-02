//! Per-pattern execution-time budget enforcement.
//!
//! Two-tier policy per FR-006:
//!
//! * [`crate::PatternKind::Regex`] — `regex` is RE2-style, linear-time, immune to
//!   catastrophic backtracking. The fast path runs in the caller thread with no
//!   timeout overhead.
//! * [`crate::PatternKind::Fancy`] — `fancy-regex` allows lookaround at the cost
//!   of backtracking. Match is run on a worker thread that is joined with a
//!   deadline; when the deadline expires, the worker is detached, a `WARN`
//!   `tracing` event is emitted with the pattern name, and the recognizer
//!   continues with the next pattern.

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::error::PatternError;
use crate::pattern::{CompiledPattern, Pattern};

/// Byte-offset span returned by [`run`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Span {
    pub start: usize,
    pub end: usize,
}

/// Outcome of a single per-pattern execution attempt.
#[derive(Debug)]
pub(crate) enum Outcome {
    /// Pattern completed; carries every match it produced.
    Matches(Vec<Span>),
    /// Pattern compilation failed at use-time.
    CompileError,
    /// `Fancy` pattern exceeded the budget; results dropped, `WARN` logged.
    TimedOut,
}

/// Run a pattern over `text` honouring `budget`.
pub(crate) fn run(pattern: &Pattern, text: &str, budget: Duration) -> Outcome {
    let compiled = match pattern.compiled() {
        Ok(c) => c,
        Err(PatternError::InvalidRegex { .. } | PatternError::InvalidScore { .. }) => {
            return Outcome::CompileError;
        }
    };

    match compiled {
        CompiledPattern::Regex(re) => Outcome::Matches(run_regex(re, text)),
        CompiledPattern::Fancy(re) => run_fancy(pattern.name(), re, text, budget),
    }
}

fn run_regex(re: &regex::Regex, text: &str) -> Vec<Span> {
    re.find_iter(text)
        .map(|m| Span {
            start: m.start(),
            end: m.end(),
        })
        .collect()
}

fn run_fancy(name: &str, re: &fancy_regex::Regex, text: &str, budget: Duration) -> Outcome {
    let (tx, rx) = mpsc::channel::<Vec<Span>>();
    // We must move owned copies of the regex source + input into the worker.
    // `fancy_regex::Regex` is `Send + Sync`, so we send a clone of it into the worker thread.
    let re_cloned = re.clone();
    let text_owned: String = text.to_owned();

    thread::spawn(move || {
        let mut spans = Vec::new();
        for m in re_cloned.find_iter(&text_owned).flatten() {
            spans.push(Span {
                start: m.start(),
                end: m.end(),
            });
        }
        let _ = tx.send(spans);
    });

    let started = Instant::now();
    match rx.recv_timeout(budget) {
        Ok(spans) => Outcome::Matches(spans),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            let elapsed = started.elapsed();
            tracing::warn!(
                target: "dbmcp_pii::timeout",
                pattern = name,
                budget_ms = u64::try_from(budget.as_millis()).unwrap_or(u64::MAX),
                elapsed_ms = u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX),
                "pattern exceeded execution budget; results dropped"
            );
            Outcome::TimedOut
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => Outcome::Matches(Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Outcome, run};
    use crate::pattern::Pattern;
    use crate::score::Score;

    fn s(v: f32) -> Score {
        Score::new(v).unwrap()
    }

    #[test]
    fn regex_fast_path_returns_matches() {
        let p = Pattern::new("digits", r"\d+", s(0.5)).unwrap();
        let out = run(&p, "abc 12 def 34", Duration::from_millis(10));
        match out {
            Outcome::Matches(spans) => assert_eq!(spans.len(), 2),
            other => panic!("unexpected outcome: {other:?}"),
        }
    }

    #[test]
    fn fancy_zero_budget_times_out() {
        // The worker thread cannot finish before a one-microsecond deadline expires,
        // so the path through `recv_timeout` must take the `Timeout` branch.
        let p = Pattern::new_fancy("ip_pre", r"(?<![\w:])\d+", s(0.6)).unwrap();
        let input = "a".repeat(2048) + " 123";
        let out = run(&p, &input, Duration::from_micros(1));
        assert!(matches!(out, Outcome::TimedOut));
    }

    #[test]
    fn fancy_completes_when_fast() {
        let p = Pattern::new_fancy("ip_pre", r"(?<![\w:])\d+", s(0.6)).unwrap();
        let out = run(&p, "abc 123", Duration::from_millis(50));
        match out {
            Outcome::Matches(spans) => assert_eq!(spans.len(), 1),
            other => panic!("unexpected outcome: {other:?}"),
        }
    }
}

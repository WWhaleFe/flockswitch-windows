//! Window ownership and fallback for an interactive Store update.
use anyhow::Result;

/// The user closed the Store's update dialog: not a failure, so no Store page is opened.
#[derive(Debug)]
pub struct Canceled;

impl std::fmt::Display for Canceled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("the Store update was canceled")
    }
}

impl std::error::Error for Canceled {}

pub fn run<T>(
    owner: Option<isize>,
    initialize: impl FnOnce(isize) -> Result<T>,
    install: impl FnOnce(T) -> Result<()>,
    fallback: impl FnOnce(),
) -> Result<()> {
    let result = owner
        .filter(|hwnd| *hwnd != 0)
        .ok_or_else(|| anyhow::anyhow!("the settings window handle is unavailable"))
        .and_then(initialize)
        .and_then(install);
    result.inspect_err(|e| {
        if e.downcast_ref::<Canceled>().is_none() {
            fallback();
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn missing_window_opens_the_store_without_requesting_a_dialog() {
        for owner in [None, Some(0)] {
            let opened = Cell::new(false);
            let result = run(
                owner,
                |_| panic!("no valid owner"),
                |_: ()| panic!("no dialog"),
                || opened.set(true),
            );
            assert!(result.is_err());
            assert!(opened.get());
        }
    }

    #[test]
    fn canceling_the_dialog_does_not_open_the_store() {
        let result = run(Some(123), Ok, |_| Err(Canceled.into()), || panic!("canceled"));
        assert!(result.unwrap_err().downcast_ref::<Canceled>().is_some());
    }

    #[test]
    fn initialization_failure_opens_the_store_and_does_not_install() {
        let opened = Cell::new(false);
        let result = run(
            Some(123),
            |hwnd| {
                assert_eq!(hwnd, 123);
                anyhow::bail!("Initialize failed")
            },
            |_: ()| panic!("initialization failed"),
            || opened.set(true),
        );
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Initialize failed"));
        assert!(opened.get());
    }

    #[test]
    fn installation_uses_the_captured_window_and_falls_back_on_failure() {
        let opened = Cell::new(false);
        let result = run(
            Some(123),
            Ok,
            |hwnd| {
                assert_eq!(hwnd, 123);
                anyhow::bail!("Store request failed")
            },
            || opened.set(true),
        );
        assert!(result.is_err());
        assert!(opened.get());
        run(
            Some(456),
            Ok,
            |hwnd| {
                assert_eq!(hwnd, 456);
                Ok(())
            },
            || panic!("update succeeded"),
        )
        .unwrap();
    }
}

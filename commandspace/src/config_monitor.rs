use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use cba::_dbg;
use log::{debug, error, warn};
use notify::event::{ModifyKind, RenameMode};
use notify::{
    Config, Error as NotifyError, Event as NotifyEvent, EventKind, RecommendedWatcher,
    RecursiveMode, Watcher,
};

use alacritty_terminal::thread;

use crate::event::{Event, EventLoopProxy, EventType};
use crate::paths::{config_dir, config_path};

const DEBOUNCE_DELAY: Duration = Duration::from_millis(10);

/// The fallback for `RecommendedWatcher` polling.
const FALLBACK_POLLING_TIMEOUT: Duration = Duration::from_secs(1);

/// Config file update monitor.
pub struct ConfigMonitor {
    thread: JoinHandle<()>,
    shutdown_tx: Sender<Result<NotifyEvent, NotifyError>>,
}

impl ConfigMonitor {
    pub fn new(event_proxy: EventLoopProxy) -> Option<Self> {
        // The Duration argument is a debouncing period.
        let (tx, rx) = mpsc::channel();
        let mut watcher = match RecommendedWatcher::new(
            tx.clone(),
            Config::default().with_poll_interval(FALLBACK_POLLING_TIMEOUT),
        ) {
            Ok(watcher) => watcher,
            Err(err) => {
                error!("Unable to watch config file: {err}");
                return None;
            },
        };

        let join_handle = thread::spawn_named("config watcher", move || {
            if let Err(err) = watcher.watch(&config_dir(), RecursiveMode::NonRecursive) {
                debug!("Unable to watch config directory: {err}");
            }

            // The current debouncing time.
            let mut debouncing_deadline: Option<Instant> = None;

            // The events accumulated during the debounce period.
            let mut received_events = Vec::new();

            loop {
                // We use `recv_timeout` to debounce the events coming from the watcher and reduce
                // the amount of config reloads.
                let event = match debouncing_deadline.as_ref() {
                    Some(debouncing_deadline) => rx.recv_timeout(
                        debouncing_deadline.saturating_duration_since(Instant::now()),
                    ),
                    None => {
                        let event = rx.recv().map_err(Into::into);
                        // Set the debouncing deadline after receiving the event.
                        debouncing_deadline = Some(Instant::now() + DEBOUNCE_DELAY);
                        event
                    },
                };

                match event {
                    Ok(Ok(event)) => match event.kind {
                        EventKind::Other if event.info() == Some("shutdown") => break,
                        // Ignore when config file is moved as it's equivalent to deletion.
                        // Some editors trigger this as they move the file as part of saving.
                        EventKind::Modify(ModifyKind::Name(
                            RenameMode::From | RenameMode::Both,
                        )) => (),
                        EventKind::Any
                        | EventKind::Create(_)
                        | EventKind::Modify(_)
                        | EventKind::Other => {
                            received_events.push(event);
                        },
                        _ => (),
                    },
                    Err(RecvTimeoutError::Timeout) => {
                        _dbg!(&received_events);

                        // Go back to polling the events.
                        debouncing_deadline = None;

                        if received_events
                            .drain(..)
                            .flat_map(|event| event.paths.into_iter())
                            .any(|path| path == config_path())
                        {
                            // Always reload the primary configuration file.
                            let event = Event::new(EventType::ConfigReload, None);
                            event_proxy.send_event(event);
                        }
                    },
                    Ok(Err(err)) => {
                        debug!("Config watcher errors: {err:?}");
                    },
                    Err(err) => {
                        debug!("Config watcher channel dropped unexpectedly: {err}");
                        break;
                    },
                };
            }
        });

        Some(Self { thread: join_handle, shutdown_tx: tx })
    }

    /// Synchronously shut down the monitor.
    pub fn shutdown(self) {
        // Request shutdown.
        let mut event = NotifyEvent::new(EventKind::Other);
        event = event.set_info("shutdown");
        let _ = self.shutdown_tx.send(Ok(event));

        // Wait for thread to terminate.
        if let Err(err) = self.thread.join() {
            warn!("config monitor shutdown failed: {err:?}");
        }
    }
}

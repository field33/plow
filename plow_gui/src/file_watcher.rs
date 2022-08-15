use async_std::stream::Stream;

use iced::futures;
use iced_native::futures::channel::mpsc::{channel, Receiver};
use iced_native::futures::{SinkExt, StreamExt};
use iced_native::subscription::Recipe;

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

pub struct FileWatcher {
    path: PathBuf,
}

impl FileWatcher {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self { path: path.into() }
    }
}

impl<H, I> Recipe<H, I> for FileWatcher
where
    H: Hasher,
{
    type Output = Result<Event, notify::Error>;

    fn hash(&self, state: &mut H) {
        self.path.hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        Box::pin(async_watch_stream(self.path.clone()).filter_map(futures::future::ready))
    }
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        notify::Config::default(),
    )?;

    Ok((watcher, rx))
}

fn async_watch_stream<P: AsRef<Path>>(
    path: P,
) -> impl Stream<Item = Option<notify::Result<notify::Event>>> {
    let (mut watcher, rx) = async_watcher().unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher
        .watch(path.as_ref(), RecursiveMode::NonRecursive)
        .unwrap();
    Box::pin(futures::stream::unfold(
        (watcher, rx),
        move |(watcher, mut rx)| async move {
            match rx.try_next() {
                Ok(Some(event)) => Some((Some(event), (watcher, rx))),
                Ok(None) => None,
                Err(_) => Some((None, (watcher, rx))),
            }
        },
    ))
}

use std::sync::Arc;
use std::io;
use std::collections::HashMap;
use futures::{stream, StreamExt};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::{mpsc, OwnedRwLockReadGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio_stream::Stream;
use crate::{Driver, Operation};
use crate::driver::event_based::EventBasedRepr;
use crate::driver::{BlockStatus, EventBased};

pub struct DriversReadGuard(OwnedRwLockReadGuard<HashMap<usize, Arc<RwLock<Driver<EventBased>>>>>);

impl DriversReadGuard {
    pub async fn get(&self, index: usize) -> Option<OwnedRwLockReadGuard<Driver<EventBased>>> {
        if let Some(driver) = self.0.get(&index) {
            Some(Arc::clone(driver).read_owned().await)
        } else {
            None
        }
    }

    pub async fn stream(&self) -> impl Stream<Item = OwnedRwLockReadGuard<Driver<EventBased>>> + Unpin + '_ {
        Box::pin(stream::unfold(self.0.keys().copied(), |mut values| async {
            Some((self.get(values.next()?).await?, values))
        }))
    }
}

pub struct Drivers {
    inner: Arc<RwLock<HashMap<usize, Arc<RwLock<Driver<EventBased>>>>>>,
}

impl Drivers {
    pub async fn read(&self) -> DriversReadGuard {
        DriversReadGuard(Arc::clone(&self.inner).read_owned().await)
    }
}

pub async fn drivers() -> io::Result<(Drivers, UnboundedReceiver<io::Error>)> {
    let drivers = Arc::new(RwLock::new(HashMap::new()));
    let (sender, receiver) = mpsc::unbounded_channel();
    let mut events = crate::events().await?;

    tokio::spawn({
        let drivers = Arc::clone(&drivers);

        async move {
            while let Some(event) = events.next().await {
                match event {
                    Ok(event) => match event.operation {
                        Operation::Add => {
                            let _ = drivers.write().await.insert(event.index, Arc::new(RwLock::new(Driver(EventBasedRepr {
                                index: event.index,
                                kind: event.kind,
                                block_status: BlockStatus::from_block(event.block),
                            }))));
                        }
                        Operation::Delete => {
                            let _ = drivers.write().await.remove(&event.index);
                        },
                        Operation::Change { .. } => {
                            let mut drivers = drivers.write().await;

                            // todo: get rid of this unwrap
                            let driver = drivers.get_mut(&event.index).unwrap();

                            let mut driver: RwLockWriteGuard<Driver<EventBased>> = driver.write().await;
                            driver.0.block_status = BlockStatus::from_block(event.block);
                        }
                    },
                    Err(error) => {
                        let _ = sender.send(error);
                        continue
                    }
                }
            }
        }
    });

    Ok((Drivers { inner: drivers }, receiver))
}

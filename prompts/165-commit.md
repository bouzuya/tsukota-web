# 165 commit log

```
Apply Repository trait to FirestoreDeviceRepository

Replace the manual load_events_impl/save_events_impl/build_aggregate_writes_in_tx
implementations with the shared Repository trait. The query collection writes
(devices/*) are handled via the save_events callback.
```

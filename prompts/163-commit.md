# 163 commit log

```
Add callback parameter to Repository::save_events

Add a `SaveEventsCallback` type alias and a `callback` parameter to
`Repository::save_events` to allow callers to perform additional
Firestore writes within the same transaction. The callback receives
`&mut Transaction` and is invoked after the event documents are written.
```

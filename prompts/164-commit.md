Change Repository::save_events callback to async

The callback parameter type is changed from a sync FnOnce returning
Result to an async FnOnce returning Pin<Box<dyn Future>>. This allows
callbacks to perform async operations like transaction.get() within the
same transaction.

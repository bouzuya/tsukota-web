Remove InMemoryEventStore and InMemoryProjection

These in-memory implementations are no longer used anywhere
in the codebase. Remove the source files and their exports
from the infra crate.

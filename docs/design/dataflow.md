## How the data flows inside the application

```
              ┌────────┐
              │        │
    ┌─────────┤ Reader │
    │         │        │
    │         └───┬────┘
    │             │
    │          on new    ┌──────────────┐
    │             │      │              │
    │             ▼      ▼              │
 append       Process queue◄────┐       │
    │             │             │       │
    │             │          Chunking   │
┌───▼──┐       ┌──▼──────┐      │       │
│LogBuf├─read──►         ├─┐    │       │
└───┬──┘       │ Workers │ │    │       │
    │          │         │ ├────┘  on search input
    │          └─┬───────┤ │            │
    │            └─┼─────┴─┘            │
    │              │                    │
    │            queue                  │
    │              │                    │
    │          ┌───▼───────┐            │
    │          │Sort thread│            │
    │          └───┬───────┘            │
    │              │                    │
    │          ┌───▼───┐                │
    │          │Matches├────read────┐   │
    │          └───────┘            │   │
    │                            ┌──▼───┴────┐
    └─────────────read───────────► UI Thread │
                                 └───────────┘
```

### Reader thread
This thread owns the stdin and consumes it creating new log lines.
Each log line is appended to `LogBuf` and then a processing request is enqueued.

### Workers
Those threads are responsible of finding matches in `LogBuf`.
They wait on process queue for requests.
If the request range is larger than set block size, the thread first chunks it and enqueues new requests back.
When correctly sized range is received, relevant log lines are read from `LogBuf` and checked for matches.
Any found matches are enqueued in sorted order to a queue.

### Sort thread
This thread receives blocks of sorted matches and inserts it to shared data structure in sorted way.

### UI Thread
UI thread reads matches and log lines, renders and displays relevant data to user.


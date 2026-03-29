```text
Datasource Layer
  |
  | emits base Carbon updates
  v
+-------------------+
| AccountUpdate     |
| TransactionUpdate |
| AccountDeletion   |
| BlockDetails      |
+-------------------+
  |
  | Carbon transforms / routes
  v
+------------------------------+
| Transaction parsing          |
| Nested instruction parsing   |
| Account decoding             |
| Log/CPI event extraction     |
+------------------------------+
  |
  v
+------------------------------+
| Decoded account families     |
| Decoded instruction families |
| Decoded event families       |
| Optional block families      |
+------------------------------+
  |
  v
+------------------------------+
| Decoder-owned ClickHouse     |
| schema + row mapping         |
+------------------------------+
  |
  v
+------------------------------+
| ClickHouse landing tables    |
| buffered, append-only writes |
+------------------------------+
  |
  v
+------------------------------+
| Pipeline finalization drains |
| remaining buffered rows      |
+------------------------------+

```

More concretely:

```text
Datasource
  -> TransactionUpdate
     -> TransactionMetadata
        -> InstructionMetadata
           -> typed instruction decoder
              -> optional CPI/log event decoder

Datasource
  -> AccountUpdate
     -> typed account decoder

Datasource
  -> AccountDeletion
     -> account-state tombstone / deletion handling

Datasource
  -> BlockDetails
     -> optional block analytics tables
```

Current V1 note:

```text
Implemented today:
- decoded CPI/event path only
- Jupiter swap `swap_event` landing table only
- no serving table yet
- no account ClickHouse sink yet
```

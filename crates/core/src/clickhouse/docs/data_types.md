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
|          ClickHouse          |
|             Sink             |
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

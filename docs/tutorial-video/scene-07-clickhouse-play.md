Finally, ClickHouse Play validates the data.

We open the database menu, choose `default`, and browse generated landing tables
with row and byte metadata. Then we inspect representative Jupiter event and
instruction rows, plus Token Program instruction and account rows.

The rows show decoded Solana activity by table family, slot, signature, amounts,
source, and mode. At this point the pipeline is proven end to end.

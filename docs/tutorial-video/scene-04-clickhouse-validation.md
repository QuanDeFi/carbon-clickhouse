# Scene 04: ClickHouse SQL Validation

After ingestion, we query ClickHouse directly. The goal is to prove that the
landing tables were created and populated with typed Jupiter rows.

The table list shows the generated landing tables for Jupiter instructions and
CPI events. The count query checks how many route rows and unique transaction
signatures landed. Then the sample route rows show the decoded swap amount
fields and the number of route legs, while the event query shows per-swap AMM
execution amounts. These samples are more useful than counts alone because they
show what analysts can actually inspect from the landing layer.

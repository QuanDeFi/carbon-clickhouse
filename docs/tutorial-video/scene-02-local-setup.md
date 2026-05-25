# Scene 02: Local Setup

We start from the repository root with the local stack already configured.
The first command checks the containers that matter for this tutorial:
ClickHouse for storage, Prometheus for metrics, and Grafana as an optional UI.

The next three checks prove the local endpoints are reachable. ClickHouse
returns `1`, Prometheus reports that it is ready, and Grafana returns its health
payload. We do not need to show runtime secrets or recording configuration here;
those are already loaded outside the visible terminal.

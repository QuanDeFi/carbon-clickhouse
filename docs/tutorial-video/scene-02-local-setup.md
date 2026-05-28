# Scene 02: Local Services Check

We start from the repository root with the local stack already running. The
first command checks the services that matter for the tutorial: ClickHouse for
storage, Prometheus for metric scraping, and Grafana for the dashboard.

The next commands are simple health checks. ClickHouse returns 1, Prometheus
reports that it is ready, and Grafana returns its health payload. Runtime values
such as the provider RPC URL are already loaded outside the visible terminal, so
the viewer sees only the local wiring.

Before checking any runtime output, we show the local setup files that make the
demo environment reproducible.

`monitoring/compose.yaml` brings up Prometheus and Grafana on the host network.
`monitoring/prometheus/prometheus.yml` scrapes the two example metrics ports:
Jupiter on 9464 and Token Program on 9465. `scripts/demo/setup-monitoring.sh`
is the small deterministic setup command that starts the stack and checks each
endpoint.

The next scene runs those checks in a terminal. This scene is just the map: where
the setup lives, and which files someone would change if they needed to adjust
the local observability stack.

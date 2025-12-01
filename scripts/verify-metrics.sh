#!/usr/bin/env bash
# Verify metrics pipeline is working
# Tests each stage: Application → OTLP → Prometheus → Grafana

set -euo pipefail

echo "🔍 Verifying Metrics Pipeline"
echo "=============================="
echo

# Check if containers are running
echo "1. Checking containers..."
if ! podman ps --format "{{.Names}}" | grep -q "botticelli"; then
    echo "❌ Bot server container not running"
    echo "   Start with: just bot-up"
    exit 1
fi
echo "✅ Bot server running"

if ! podman ps --format "{{.Names}}" | grep -q "prometheus"; then
    echo "❌ Prometheus container not running"
    echo "   Start with: podman-compose -f docker-compose.observability.yml up -d"
    exit 1
fi
echo "✅ Prometheus running"

if ! podman ps --format "{{.Names}}" | grep -q "grafana"; then
    echo "❌ Grafana container not running"
    echo "   Start with: podman-compose -f docker-compose.observability.yml up -d"
    exit 1
fi
echo "✅ Grafana running"
echo

# Check Prometheus is accessible
echo "2. Checking Prometheus..."
if ! curl -s http://localhost:9090/-/healthy > /dev/null; then
    echo "❌ Prometheus not responding at http://localhost:9090"
    exit 1
fi
echo "✅ Prometheus healthy"
echo

# Check for LLM metrics in Prometheus
echo "3. Checking for LLM metrics in Prometheus..."
METRICS=$(curl -s http://localhost:9090/api/v1/label/__name__/values | jq -r '.data[]' | grep -i llm || true)

if [ -z "$METRICS" ]; then
    echo "❌ No LLM metrics found in Prometheus"
    echo
    echo "Troubleshooting steps:"
    echo "  1. Check bot server logs: podman logs botticelli-bot-server"
    echo "  2. Verify OTEL_EXPORTER is set to 'otlp'"
    echo "  3. Verify OTEL_EXPORTER_OTLP_ENDPOINT points to Prometheus/Collector"
    echo "  4. Trigger an LLM request to generate metrics"
    exit 1
fi

echo "✅ Found LLM metrics:"
echo "$METRICS" | sed 's/^/   - /'
echo

# Check Grafana is accessible
echo "4. Checking Grafana..."
if ! curl -s http://localhost:3000/api/health > /dev/null; then
    echo "❌ Grafana not responding at http://localhost:3000"
    exit 1
fi
echo "✅ Grafana healthy"
echo

# Check Prometheus data source in Grafana
echo "5. Checking Grafana → Prometheus connection..."
DATASOURCES=$(curl -s -u admin:admin http://localhost:3000/api/datasources)
if ! echo "$DATASOURCES" | jq -e '.[] | select(.type=="prometheus")' > /dev/null; then
    echo "❌ Prometheus data source not configured in Grafana"
    echo "   Configure at: http://localhost:3000/connections/datasources"
    exit 1
fi
echo "✅ Prometheus data source configured"
echo

echo "✅ All checks passed!"
echo
echo "Next steps:"
echo "  1. Open Grafana: http://localhost:3000 (admin/admin)"
echo "  2. Go to Explore and select Prometheus"
echo "  3. Query: llm_requests"
echo "  4. Import dashboards from grafana/dashboards/"

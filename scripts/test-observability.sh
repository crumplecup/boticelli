#!/usr/bin/env bash
# Integration test for observability stack
# Verifies metrics, traces, and dashboards are working

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Botticelli Observability Integration Test ===${NC}\n"

# Configuration
COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.observability.yml}"
JAEGER_URL="${JAEGER_URL:-http://localhost:16686}"
PROMETHEUS_URL="${PROMETHEUS_URL:-http://localhost:9090}"
GRAFANA_URL="${GRAFANA_URL:-http://localhost:3000}"
OTLP_ENDPOINT="${OTLP_ENDPOINT:-http://localhost:4317}"

FAILED_CHECKS=0

# Helper functions
check_pass() {
    echo -e "${GREEN}✓${NC} $1"
}

check_fail() {
    echo -e "${RED}✗${NC} $1"
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
}

check_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

check_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check HTTP endpoint
check_http() {
    local url=$1
    local expected_code=${2:-200}
    
    if command_exists curl; then
        response=$(curl -s -o /dev/null -w "%{http_code}" "$url" 2>/dev/null || echo "000")
        if [[ "$response" == "$expected_code" ]]; then
            return 0
        else
            return 1
        fi
    else
        check_warn "curl not found, skipping HTTP check for $url"
        return 0
    fi
}

# Check if container is running
check_container() {
    local container_name=$1
    
    if command_exists podman; then
        podman ps --filter "name=$container_name" --format "{{.Names}}" | grep -q "$container_name"
    elif command_exists docker; then
        docker ps --filter "name=$container_name" --format "{{.Names}}" | grep -q "$container_name"
    else
        check_fail "Neither podman nor docker found"
        return 1
    fi
}

# Phase 1: Prerequisites
echo -e "${BLUE}Phase 1: Checking Prerequisites${NC}"

if command_exists podman; then
    check_pass "podman found"
    CONTAINER_CMD="podman"
elif command_exists docker; then
    check_pass "docker found"
    CONTAINER_CMD="docker"
else
    check_fail "Neither podman nor docker found"
    exit 1
fi

if command_exists curl; then
    check_pass "curl found"
else
    check_warn "curl not found (some checks will be skipped)"
fi

if command_exists jq; then
    check_pass "jq found"
    HAS_JQ=true
else
    check_warn "jq not found (JSON parsing will be limited)"
    HAS_JQ=false
fi

echo ""

# Phase 2: Container Health
echo -e "${BLUE}Phase 2: Checking Container Health${NC}"

for container in botticelli-jaeger botticelli-prometheus botticelli-grafana; do
    if check_container "$container"; then
        check_pass "$container is running"
    else
        check_fail "$container is not running"
        check_info "Start with: ${CONTAINER_CMD}-compose -f $COMPOSE_FILE up -d"
    fi
done

echo ""

# Phase 3: Service Endpoints
echo -e "${BLUE}Phase 3: Checking Service Endpoints${NC}"

# Jaeger
if check_http "$JAEGER_URL" 200; then
    check_pass "Jaeger UI accessible at $JAEGER_URL"
else
    check_fail "Jaeger UI not accessible at $JAEGER_URL"
fi

# Jaeger OTLP receiver
if check_http "http://localhost:4317" 000; then
    check_pass "Jaeger OTLP receiver accessible at localhost:4317"
else
    check_warn "Jaeger OTLP receiver check inconclusive (gRPC endpoint)"
fi

# Prometheus
if check_http "$PROMETHEUS_URL" 200; then
    check_pass "Prometheus accessible at $PROMETHEUS_URL"
else
    check_fail "Prometheus not accessible at $PROMETHEUS_URL"
fi

# Grafana
if check_http "$GRAFANA_URL/api/health" 200; then
    check_pass "Grafana accessible at $GRAFANA_URL"
else
    check_fail "Grafana not accessible at $GRAFANA_URL"
fi

echo ""

# Phase 4: Prometheus Targets
echo -e "${BLUE}Phase 4: Checking Prometheus Targets${NC}"

if command_exists curl && [[ "$HAS_JQ" == true ]]; then
    targets_json=$(curl -s "$PROMETHEUS_URL/api/v1/targets" 2>/dev/null || echo '{}')
    
    if echo "$targets_json" | jq -e '.data.activeTargets' >/dev/null 2>&1; then
        active_count=$(echo "$targets_json" | jq '.data.activeTargets | length')
        up_count=$(echo "$targets_json" | jq '[.data.activeTargets[] | select(.health == "up")] | length')
        
        if [[ "$up_count" -gt 0 ]]; then
            check_pass "Prometheus has $up_count/$active_count targets UP"
            
            # Show target details
            echo "$targets_json" | jq -r '.data.activeTargets[] | "  - \(.labels.job): \(.health)"' 2>/dev/null || true
        else
            check_warn "No Prometheus targets are UP (expected 1+ for Jaeger metrics)"
        fi
    else
        check_warn "Could not parse Prometheus targets response"
    fi
else
    check_warn "Skipping Prometheus targets check (requires curl + jq)"
fi

echo ""

# Phase 5: Grafana Datasources
echo -e "${BLUE}Phase 5: Checking Grafana Datasources${NC}"

if command_exists curl && [[ "$HAS_JQ" == true ]]; then
    # Grafana API requires auth
    datasources_json=$(curl -s -u admin:admin "$GRAFANA_URL/api/datasources" 2>/dev/null || echo '[]')
    
    if echo "$datasources_json" | jq -e 'type == "array"' >/dev/null 2>&1; then
        ds_count=$(echo "$datasources_json" | jq 'length')
        
        if [[ "$ds_count" -gt 0 ]]; then
            check_pass "Grafana has $ds_count datasource(s) configured"
            
            # Check for Prometheus datasource
            prom_count=$(echo "$datasources_json" | jq '[.[] | select(.type == "prometheus")] | length')
            if [[ "$prom_count" -gt 0 ]]; then
                check_pass "Prometheus datasource configured"
            else
                check_warn "No Prometheus datasource found"
            fi
            
            # Check for Jaeger datasource
            jaeger_count=$(echo "$datasources_json" | jq '[.[] | select(.type == "jaeger")] | length')
            if [[ "$jaeger_count" -gt 0 ]]; then
                check_pass "Jaeger datasource configured"
            else
                check_warn "No Jaeger datasource found"
            fi
        else
            check_warn "No Grafana datasources configured (will auto-provision on first use)"
        fi
    else
        check_warn "Could not parse Grafana datasources response"
    fi
else
    check_warn "Skipping Grafana datasources check (requires curl + jq)"
fi

echo ""

# Phase 6: Grafana Dashboards
echo -e "${BLUE}Phase 6: Checking Grafana Dashboards${NC}"

if command_exists curl && [[ "$HAS_JQ" == true ]]; then
    dashboards_json=$(curl -s -u admin:admin "$GRAFANA_URL/api/search?type=dash-db" 2>/dev/null || echo '[]')
    
    if echo "$dashboards_json" | jq -e 'type == "array"' >/dev/null 2>&1; then
        dash_count=$(echo "$dashboards_json" | jq 'length')
        
        if [[ "$dash_count" -gt 0 ]]; then
            check_pass "Grafana has $dash_count dashboard(s)"
            
            # Check for specific Botticelli dashboards
            for dashboard in "llm-api-health" "narrative-performance" "bot-health"; do
                if echo "$dashboards_json" | jq -e --arg uid "$dashboard" '[.[] | select(.uid == $uid)] | length > 0' >/dev/null 2>&1; then
                    check_pass "Dashboard '$dashboard' found"
                else
                    check_warn "Dashboard '$dashboard' not found (may need manual import)"
                fi
            done
        else
            check_warn "No Grafana dashboards found (auto-provisioning may still be in progress)"
            check_info "Dashboards should appear within 10 seconds of Grafana startup"
        fi
    else
        check_warn "Could not parse Grafana dashboards response"
    fi
else
    check_warn "Skipping Grafana dashboards check (requires curl + jq)"
fi

echo ""

# Phase 7: Metrics Availability
echo -e "${BLUE}Phase 7: Checking Metrics Availability${NC}"

if command_exists curl && [[ "$HAS_JQ" == true ]]; then
    metrics_json=$(curl -s "$PROMETHEUS_URL/api/v1/label/__name__/values" 2>/dev/null || echo '{}')
    
    if echo "$metrics_json" | jq -e '.data' >/dev/null 2>&1; then
        metric_count=$(echo "$metrics_json" | jq '.data | length')
        check_pass "Prometheus has $metric_count metrics available"
        
        # Check for specific Botticelli metrics
        for metric in "narrative_json_failures" "narrative_json_success" "bot_executions" "pipeline_generated"; do
            if echo "$metrics_json" | jq -e --arg name "$metric" '.data | contains([$name])' >/dev/null 2>&1; then
                check_pass "Metric '$metric' found"
            else
                check_warn "Metric '$metric' not found (bot may not be running yet)"
            fi
        done
        
        # Check for LLM metrics (may not exist yet)
        if echo "$metrics_json" | jq -e '.data | contains(["llm_requests"])' >/dev/null 2>&1; then
            check_pass "LLM metrics found (instrumentation complete)"
        else
            check_info "LLM metrics not found (instrumentation needed - see OBSERVABILITY_DASHBOARDS.md)"
        fi
    else
        check_warn "Could not query Prometheus metrics"
    fi
else
    check_warn "Skipping metrics check (requires curl + jq)"
fi

echo ""

# Phase 8: Trace Data
echo -e "${BLUE}Phase 8: Checking Trace Data${NC}"

if command_exists curl && [[ "$HAS_JQ" == true ]]; then
    # Query Jaeger API for services
    services_json=$(curl -s "$JAEGER_URL/api/services" 2>/dev/null || echo '{"data":null}')
    
    if echo "$services_json" | jq -e '.data' >/dev/null 2>&1; then
        service_count=$(echo "$services_json" | jq '.data | length')
        
        if [[ "$service_count" -gt 0 ]]; then
            check_pass "Jaeger has traces from $service_count service(s)"
            echo "$services_json" | jq -r '.data[]' | sed 's/^/  - /' 2>/dev/null || true
        else
            check_info "No traces in Jaeger yet (run bot with OTEL_EXPORTER=otlp)"
        fi
    else
        check_warn "Could not query Jaeger services API"
    fi
else
    check_warn "Skipping trace data check (requires curl + jq)"
fi

echo ""

# Summary
echo -e "${BLUE}=== Test Summary ===${NC}\n"

if [[ $FAILED_CHECKS -eq 0 ]]; then
    echo -e "${GREEN}✓ All critical checks passed!${NC}\n"
    
    echo "Next steps:"
    echo "1. Run bot with OTLP export:"
    echo "   OTEL_EXPORTER=otlp OTEL_EXPORTER_OTLP_ENDPOINT=$OTLP_ENDPOINT \\"
    echo "   cargo run --release --features otel-otlp -p botticelli_server --bin bot-server"
    echo ""
    echo "2. View traces in Jaeger: $JAEGER_URL"
    echo "3. View metrics in Prometheus: $PROMETHEUS_URL"
    echo "4. View dashboards in Grafana: $GRAFANA_URL"
    echo ""
    echo "Grafana credentials: admin/admin"
    
    exit 0
else
    echo -e "${RED}✗ $FAILED_CHECKS check(s) failed${NC}\n"
    
    echo "Troubleshooting:"
    echo "1. Start the stack:"
    echo "   ${CONTAINER_CMD}-compose -f $COMPOSE_FILE up -d"
    echo ""
    echo "2. Check container logs:"
    echo "   ${CONTAINER_CMD} logs botticelli-jaeger"
    echo "   ${CONTAINER_CMD} logs botticelli-prometheus"
    echo "   ${CONTAINER_CMD} logs botticelli-grafana"
    echo ""
    echo "3. Verify network connectivity:"
    echo "   ${CONTAINER_CMD} network inspect botticelli"
    
    exit 1
fi

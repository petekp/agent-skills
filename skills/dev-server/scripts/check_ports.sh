#!/bin/bash

# check_ports.sh - Check port availability and list processes using common dev ports
# Usage: ./check_ports.sh [port] [--kill]
#   port: specific port to check (optional, defaults to scanning common ports)
#   --kill: kill processes on specified port

COMMON_PORTS=(3000 3001 5173 5174 8080 8000 4000 4200 8888)

check_port() {
    local port=$1
    local pid=$(lsof -ti :$port 2>/dev/null)
    if [[ -n "$pid" ]]; then
        local cmd=$(ps -p $pid -o comm= 2>/dev/null)
        local full_cmd=$(ps -p $pid -o args= 2>/dev/null | head -c 60)
        echo "Port $port: IN USE by PID $pid ($cmd)"
        echo "  Command: $full_cmd"
        return 1
    else
        echo "Port $port: AVAILABLE"
        return 0
    fi
}

kill_port() {
    local port=$1
    local pids=$(lsof -ti :$port 2>/dev/null)
    if [[ -n "$pids" ]]; then
        echo "Killing processes on port $port: $pids"
        echo "$pids" | xargs kill -9 2>/dev/null
        sleep 0.5
        if lsof -ti :$port >/dev/null 2>&1; then
            echo "Warning: Some processes may still be running"
            return 1
        else
            echo "Successfully killed processes on port $port"
            return 0
        fi
    else
        echo "No processes found on port $port"
        return 0
    fi
}

find_available_port() {
    local start_port=${1:-3000}
    for port in $(seq $start_port $((start_port + 100))); do
        if ! lsof -ti :$port >/dev/null 2>&1; then
            echo $port
            return 0
        fi
    done
    echo ""
    return 1
}

scan_common_ports() {
    echo "=== Scanning Common Dev Ports ==="
    echo ""
    local available_ports=()
    local used_ports=()

    for port in "${COMMON_PORTS[@]}"; do
        if lsof -ti :$port >/dev/null 2>&1; then
            used_ports+=($port)
            check_port $port
            echo ""
        else
            available_ports+=($port)
        fi
    done

    echo "=== Summary ==="
    if [[ ${#available_ports[@]} -gt 0 ]]; then
        echo "Available: ${available_ports[*]}"
    fi
    if [[ ${#used_ports[@]} -gt 0 ]]; then
        echo "In use: ${used_ports[*]}"
    fi

    if [[ ${#available_ports[@]} -gt 0 ]]; then
        echo ""
        echo "Suggested port: ${available_ports[0]}"
    fi
}

list_node_processes() {
    echo "=== Node/Dev Server Processes ==="
    ps aux | grep -E '(node|npm|pnpm|yarn|vite|next|webpack)' | grep -v grep | head -20
}

# Main
case "$1" in
    --scan)
        scan_common_ports
        ;;
    --list)
        list_node_processes
        ;;
    --find)
        port=$(find_available_port ${2:-3000})
        if [[ -n "$port" ]]; then
            echo $port
        else
            echo "No available ports found" >&2
            exit 1
        fi
        ;;
    --kill)
        if [[ -n "$2" ]]; then
            kill_port $2
        else
            echo "Usage: $0 --kill <port>"
            exit 1
        fi
        ;;
    [0-9]*)
        check_port $1
        ;;
    *)
        scan_common_ports
        ;;
esac

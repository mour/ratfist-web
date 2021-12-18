#!/bin/sh

function cleanup() {
	cd "$my_dir"

	if [ -n "$node_0_pid" ]; then
		kill $node_0_pid 2> /dev/null
	fi
	if [ -n "$node_1_pid" ]; then
		kill $node_1_pid 2> /dev/null
	fi
	if [ -n "$tunnel_0_pid" ]; then
		kill $tunnel_0_pid 2> /dev/null
	fi
	if [ -n "$tunnel_1_pid" ]; then
		kill $tunnel_1_pid 2> /dev/null
	fi

	rmdir "$work_dir"
}

function check_retval() {
	if [ $? -ne 0 ]; then
		echo "FAILED"

		cleanup

		exit 1
	fi
}

script_dir=$(dirname "$(readlink -e "$0")")
my_dir=$(pwd)
work_dir="/tmp/ratfist_$$"


mkdir -p "$work_dir"
check_retval

echo "BUILDING NODE STUB"
cd "${script_dir}/../node_stub"
check_retval
cargo build --release
check_retval

echo "BUILDING SERVER APP"
cd "${script_dir}/../server"
check_retval
cargo build --release --features meteo --bin ratfist_server
check_retval

cd $my_dir
check_retval

echo "STARTING SERIAL TUNNELS"
socat pty,raw,echo=0,link="${work_dir}/server_serial_0" pty,raw,echo=0,link="${work_dir}/node_serial_0" & tunnel_0_pid=$!
check_retval
socat pty,raw,echo=0,link="${work_dir}/server_serial_1" pty,raw,echo=0,link="${work_dir}/node_serial_1" & tunnel_1_pid=$!
check_retval

sleep 1

echo "STARTING NODE STUBS"
RUST_LOG=ratfist_node_stub=trace "${script_dir}/../target/release/ratfist_node_stub" "${work_dir}/node_serial_0" > "${my_dir}/node_0.log" 2>&1 & node_0_pid=$!
check_retval
RUST_LOG=ratfist_node_stub=trace "${script_dir}/../target/release/ratfist_node_stub" "${work_dir}/node_serial_1" > "${my_dir}/node_1.log" 2>&1 & node_1_pid=$!
check_retval

echo "STARTING SERVER"
for env_var_line in $(cat ${script_dir}/../server/.env | grep -v "^#"); do
	env_var_name=$(echo $env_var_line | cut -d '=' -f 1)
	env_var_val=$(echo $env_var_line | cut -d '=' -f 2)

	set | cut -d '=' -f 1 | grep "$env_var_name"
	if [ $? -ne 0 ]; then
		export ${env_var_name}="${env_var_val}"
	fi
done

trap cleanup INT

SERIAL_PORT_0_PATH="${work_dir}/server_serial_0" SERIAL_PORT_1_PATH="${work_dir}/server_serial_1" "${script_dir}/../target/release/ratfist_server"

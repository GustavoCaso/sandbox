#! /usr/bin/env bash

log_error() {
  echo "[ERROR] $(date '+%Y-%m-%d %H:%M:%S') - $*"
}

log_info() {
  echo "[INFO] $(date '+%Y-%m-%d %H:%M:%S') - $*"
}

execute_command() {
  local command
  command=$*;
  result=$(eval "$command")
  exit_code=$?
  if [ $exit_code -ne 0 ]; then 
    log_error "$result";
    return 1
  fi
  
  log_info "$result";
  return 0
}

execute_command "cat hello_world.txt 2>&1"
execute_command "cat non_exiting.txt 2>&1 >/dev/null"


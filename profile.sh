#! /bin/bash

if ! command -v samply &> /dev/null
then
  echo "Please install sample with 'cargo install samply'"
fi

call_graph=lbf

# Check if your CPU supports LBR (Last Branch Record)
if ! grep lbr < /proc/cpuinfo
then
  # Use DWARF mode if it doesn't
  call_graph=dwarf
fi

sudo perf record -a -g --call-graph=$call_graph $@
sudo chown "$USER" perf.data
samply import perf.data

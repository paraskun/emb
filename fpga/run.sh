#!/bin/sh

mkdir -p out

iverilog -g2012 -o out/sim.out $1/*.sv
./out/sim.out
vcd2lxt out/sim.vcd out/sim.lxt

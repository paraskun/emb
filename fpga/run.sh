#!/bin/sh

mkdir -p out

iverilog -g2012 -o out/sim.out $1/$1.sv $1/tb_$1.sv
./out/sim.out
vcd2lxt out/sim.vcd out/sim.lxt

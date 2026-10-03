`timescale 1ns / 1ps

module tb_sum2();

  logic a;
  logic b;
  logic cin;
  logic s;
  logic cout;
  
  sum2 DUT(.a(a), .b(b), .cin(cin), .s(s), .cout(cout));
  
  initial begin
    $dumpfile("out/sum2/sim.vcd");
    $dumpvars(0, tb_sum2);

    {cin, a, b} = 3'b000;            #10;

    repeat (7) begin
      {cin, a, b} = {cin, a, b} + 1; #10;
    end
      
  $finish;
  end

endmodule

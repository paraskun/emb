`timescale 1ns / 1ps

module tb_sum16();

  logic [3:0] a;
  logic [3:0] b;
  logic       cin;
  logic [3:0] s;
  logic       cout;
  
  sum16 DUT(.a(a), .b(b), .cin(cin), .s(s), .cout(cout));
  
  initial begin
    $dumpfile("out/sum16/sim.vcd");
    $dumpvars(0, tb_sum16);

    {cin, a, b} = 9'b0;              #10;

    repeat (511) begin
      {cin, a, b} = {cin, a, b} + 1; #10;
    end
      
  $finish;
  end

  // (1) SUM 64-bit (из 16-bit) -> (2) a + b + c из него

endmodule

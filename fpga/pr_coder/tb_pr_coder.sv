`timescale 1ns / 1ps

module tb_pr_coder();

  logic [7:0] a;
  logic [2:0] f;
  
  pr_coder DUT(.a(a), .f(f));
  
  initial begin
    $dumpfile("out/sim.vcd");
    $dumpvars(0, tb_pr_coder);

    a = 8'b11111111;  #10;
    repeat (8) begin
        a = a >> 1;   #10;
    end
      
  $finish;
  end

endmodule

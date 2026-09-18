`timescale 1ns / 1ps

module tb_coder();

  logic [7:0] a;
  logic [2:0] f;
  
  coder DUT(.a(a), .f(f));
  
  initial begin
    $dumpfile("out/sim.vcd");
    $dumpvars(0, tb_coder);

    a = 8'b00000001;  #10;
    repeat (7) begin
        a = a << 1;   #10;
    end
      
  $finish;
  end

endmodule

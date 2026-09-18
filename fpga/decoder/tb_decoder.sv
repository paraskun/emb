`timescale 1ns / 1ps

module tb_decoder();

  logic [2:0] a;
  logic [7:0] f;
  
  decoder DUT(.a(a), .f(f));
  
  initial begin
    $dumpfile("out/sim.vcd");
    $dumpvars(0, tb_decoder);

    a = 3'b000;       #10;
    repeat (7) begin
        a = a + 1;    #10;
    end
      
  $finish;
  end

endmodule

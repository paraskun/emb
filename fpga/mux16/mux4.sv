`timescale 1ns / 1ps

module mux4(
    input        [3:0] a,
    input        [1:0] s,
    output logic       f
    );

  // always_comb
  // if      (s == 2'b00) f = a[0];
  // else if (s == 2'b01) f = a[1];
  // else if (s == 2'b10) f = a[2];
  // else                 f = a[3];
  
  // always_comb
  // case (s)
  // 2'b00: f = a[0];
  // 2'b01: f = a[1];
  // 2'b10: f = a[2];
  // 2'b11: f = a[3];
  // endcase

  assign f = !s[1] & !s[0] & a[0]
           | !s[1] &  s[0] & a[1]
           |  s[1] & !s[0] & a[2]
           |  s[1] &  s[0] & a[3];
    
endmodule

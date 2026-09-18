`timescale 1ns / 1ps

module coder(
    input        [7:0] a,
    output logic [2:0] f
    );
    
    // always_comb 
    // if      (a == 8'b00000001) f = 3'b000;
    // else if (a == 8'b00000010) f = 3'b001;
    // else if (a == 8'b00000100) f = 3'b010;
    // else if (a == 8'b00001000) f = 3'b011;
    // else if (a == 8'b00010000) f = 3'b100;
    // else if (a == 8'b00100000) f = 3'b101;
    // else if (a == 8'b01000000) f = 3'b110;
    // else                       f = 3'b111;
      
    // always_comb
    // case (a)
    // 8'b00000001: f = 3'b000;
    // 8'b00000010: f = 3'b001;
    // 8'b00000100: f = 3'b010;
    // 8'b00001000: f = 3'b011;
    // 8'b00010000: f = 3'b100;
    // 8'b00100000: f = 3'b101;
    // 8'b01000000: f = 3'b110;
    // 8'b10000000: f = 3'b111;
    // endcase
    
    assign f[2] = a[4] | a[5] | a[6] | a[7];
    assign f[1] = a[2] | a[3] | a[6] | a[7];
    assign f[0] = a[1] | a[3] | a[5] | a[7];
    
endmodule

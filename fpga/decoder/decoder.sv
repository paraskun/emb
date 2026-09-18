`timescale 1ns / 1ps

module decoder(
    input        [2:0] a,
    output logic [7:0] f
    );
    
    // always_comb 
    // if      (a == 3'b000) f = 8'b00000001;
    // else if (a == 3'b001) f = 8'b00000010;
    // else if (a == 3'b010) f = 8'b00000100;
    // else if (a == 3'b011) f = 8'b00001000;
    // else if (a == 3'b100) f = 8'b00010000;
    // else if (a == 3'b101) f = 8'b00100000;
    // else if (a == 3'b110) f = 8'b01000000;
    // else                  f = 8'b10000000;

    // always_comb 
    // case (a)
    // 3'b000: f = 8'b00000001;
    // 3'b001: f = 8'b00000010;
    // 3'b010: f = 8'b00000100;
    // 3'b011: f = 8'b00001000;
    // 3'b100: f = 8'b00010000;
    // 3'b101: f = 8'b00100000;
    // 3'b110: f = 8'b01000000;
    // 3'b111: f = 8'b10000000;
    // endcase
    
    assign f[7] =  a[2] &  a[1] &  a[0];
    assign f[6] =  a[2] &  a[1] & !a[0];
    assign f[5] =  a[2] & !a[1] &  a[0];
    assign f[4] =  a[2] & !a[1] & !a[0];
    assign f[3] = !a[2] &  a[1] &  a[0];
    assign f[2] = !a[2] &  a[1] & !a[0];
    assign f[1] = !a[2] & !a[1] &  a[0];
    assign f[0] = !a[2] & !a[1] & !a[0];
    
endmodule

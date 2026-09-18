`timescale 1ns / 1ps

module pr_coder(
    input        [7:0] a,
    output logic [2:0] f
    );

    // always_comb 
    // if      (a[7]) f = 3'b111;
    // else if (a[6]) f = 3'b110;
    // else if (a[5]) f = 3'b101;
    // else if (a[4]) f = 3'b100;
    // else if (a[3]) f = 3'b011;
    // else if (a[2]) f = 3'b010;
    // else if (a[1]) f = 3'b001;
    // else           f = 3'b000;

    // always_comb
    // casex (a)
    // 8'b1xxxxxxx: f = 3'b111;
    // 8'b01xxxxxx: f = 3'b110;
    // 8'b001xxxxx: f = 3'b101;
    // 8'b0001xxxx: f = 3'b100;
    // 8'b00001xxx: f = 3'b011;
    // 8'b000001xx: f = 3'b010;
    // 8'b0000001x: f = 3'b001;
    // 8'b0000000x: f = 3'b000;
    // endcase

    //            7        6        5                 4        3                        2                        1
    assign f[2] = (a[7]) | (a[6]) | (a[5])          | (a[4])                                                                                   ;
    assign f[1] = (a[7]) | (a[6])                            | (a[3] & !a[4] & !a[5]) | (a[2] & !a[4] & !a[5])                                 ;
    assign f[0] = (a[7])          | (a[5] & !a[6])           | (a[3] & !a[4] & !a[6])                          | (a[1] & !a[2] & !a[4] & !a[6]);

endmodule

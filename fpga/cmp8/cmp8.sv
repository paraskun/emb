`timescale 1ns / 1ps

module cmp8(
    input        [7:0] a,
    input        [7:0] b,
    output             m,
    output             l
    );

    logic [3:0] a1;
    logic [3:0] b1;

    logic [1:0] a2;
    logic [1:0] b2;

    // cmp2 C0(.a(a[1:0]), .b(b[1:0]), .m(a1[0]), .l(b1[0]));
    // cmp2 C1(.a(a[3:2]), .b(b[3:2]), .m(a1[1]), .l(b1[1]));
    // cmp2 C2(.a(a[5:4]), .b(b[5:4]), .m(a1[2]), .l(b1[2]));
    // cmp2 C3(.a(a[7:6]), .b(b[7:6]), .m(a1[3]), .l(b1[3]));

    // cmp2 C4(.a(a1[1:0]), .b(b1[1:0]), .m(a2[0]), .l(b2[0]));
    // cmp2 C5(.a(a1[3:2]), .b(b1[3:2]), .m(a2[1]), .l(b2[1]));

    // cmp2 CR(.a(a2), .b(b2), .m(m), .l(l));
    
    genvar i;
    generate
      for (i = 0; i < 4; i = i + 1)
      begin label
        cmp2 C0(.a(a[i * 2 + 1:i * 2]), .b(b[i * 2 + 1:i * 2]), .m(a1[i]), .l(b1[i]));
    endgenerate

    genvar i;
    generate
      for (i = 0; i < 2; i = i + 1)
      begin label
        cmp2 C1(
          .a(a1[i * 2 + 1:i * 2]),
          .b(b1[i * 2 + 1:i * 2]),
          .m(a2[i]),
          .l(b2[i]));
    endgenerate

    cmp2 CR(.a(a2), .b(b2), .m(m), .l(l));
    
endmodule

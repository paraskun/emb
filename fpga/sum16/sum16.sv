`timescale 1ns / 1ps

module sum16(
    input  [15:0] a,
    input  [15:0] b,
    input         cin,
    output [15:0] s,
    output        cout
    );

    logic [4:0] c;
    logic [3:0] g;
    logic [3:0] p;

    assign c[0] = cin;

    genvar i;
    generate
      for (i = 0; i < 4; i = i + 1) begin: label
        sum4 S0(.a(a[4*i+3:4*i]),
                .b(b[4*i+3:4*i]),
                .cin(c[i]),
                .s(s[4*i+3:4*i]),
                .cout(),
                .g_group(g[i]),
                .p_group(p[i]));
      end
    endgenerate

    assign cout = c[4];

    assign c[1] = g[0]
                | p[0] & c[0];

    assign c[2] = g[1] 
                | p[1] & g[0]
                | p[1] & p[0] & c[0]);

    assign c[3] = g[2]
                | p[2] & g[1]
                | p[2] & p[1] & g[0]
                | p[2] & p[1] & p[0] & c[0];

    assign c[4] = g[3]
                | p[3] & g[2]
                | p[3] & p[2] & g[1]
                | p[3] & p[2] & p[1] & g[0]
                | p[3] & p[2] & p[1] & p[0] & c[0];
endmodule

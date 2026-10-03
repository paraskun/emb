`timescale 1ns / 1ps

module sum4(
    input  [3:0] a,
    input  [3:0] b,
    input        cin,
    output [3:0] s,
    output       cout
    );

    logic [4:0] c;

    // assign c[0] = cin;

    // genvar i;
    // generate
    //   for (i = 0; i < 4; i = i + 1) begin: label
    //     assign s[i] = a[i] ^ b[i] ^ c[i];
    //     assign c[i+1] = (a[i] & b[i]) | ((a[i] ^ b[i]) & c[i]);
    //   end
    // endgenerate

    // assign cout = c[4];

    logic [4:0] c;
    logic [3:0] g;
    logic [3:0] p;

    assign c[0] = cin;

    genvar i;
    generate
      for (i = 0; i < 4; i = i + 1) begin: label
        assign g[i] = a[i] & b[i];
        assign p[i] = a[i] ^ b[i];
        assign s[i] = a[i] ^ b[i] ^ c[i];
      end
    endgenerate

    assign cout = c[4];

    // assign c[1] = g[0] | p[0] & c[0];
    // assign c[2] = g[1] | p[1] & c[1];
    // assign c[3] = g[2] | p[2] & c[2];
    // assign c[4] = g[3] | p[3] & c[3];
    
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

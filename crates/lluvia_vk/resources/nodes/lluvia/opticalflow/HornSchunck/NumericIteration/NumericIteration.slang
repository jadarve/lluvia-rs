import lluvia.core;

[[vk::binding(0, 0)]]
[[format("rgba32f")]]
RWTexture2D<float4> in_image_params;

[[vk::binding(1, 0)]]
[[format("rg32f")]]
RWTexture2D<float2> in_flow;

[[vk::binding(2, 0)]]
[[format("rg32f")]]
RWTexture2D<float2> out_flow;

[shader("compute")]
[numthreads(workgroup_x, workgroup_y, workgroup_z)]
void main(uint3 threadId : SV_DispatchThreadID)
{
    uint2 coords = threadId.xy;
    uint width, height;
    out_flow.GetDimensions(width, height);

    if (coords.x >= width || coords.y >= height) {
        return;
    }

    bool leftBorder   = (coords.x == 0);
    bool rightBorder  = (coords.x == width - 1);
    bool topBorder    = (coords.y == 0);
    bool bottomBorder = (coords.y == height - 1);

    // central pixel in the 3x3 window
    float2 flow_11 = in_flow[coords];

    // row 0
    float2 flow_00 = leftBorder || topBorder ? flow_11 : in_flow[coords + int2(-1, -1)];
    float2 flow_01 = topBorder ? flow_11 : in_flow[coords + int2(0, -1)];
    float2 flow_02 = rightBorder || topBorder ? flow_11 : in_flow[coords + int2(1, -1)];

    // row 1
    float2 flow_10 = leftBorder ? flow_11 : in_flow[coords + int2(-1, 0)];
    float2 flow_12 = rightBorder ? flow_11 : in_flow[coords + int2(1, 0)];

    // row 2
    float2 flow_20 = leftBorder || bottomBorder ? flow_11 : in_flow[coords + int2(-1, 1)];
    float2 flow_21 = bottomBorder ? flow_11 : in_flow[coords + int2(0, 1)];
    float2 flow_22 = rightBorder || bottomBorder ? flow_11 : in_flow[coords + int2(1, 1)];

    // Laplacian flow average
    float2 flowAvg = 0.0833 * (flow_00 + flow_02 + flow_20 + flow_22)
                     + 0.1667 * (flow_01 + flow_10 + flow_12 + flow_21);

    // Read image parameters
    float4 imgParams = in_image_params[coords];

    float2 imgGradient       = imgParams.xy;
    float imgTimeDifference = imgParams.z;
    float regularization    = imgParams.w;

    float gain = (dot(imgGradient, flowAvg) + imgTimeDifference) * regularization;

    float2 flow = flowAvg - imgGradient * gain;

    out_flow[coords] = flow;
}

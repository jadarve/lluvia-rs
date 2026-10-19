import lluvia.core;
import lluvia.color;

[[vk::binding(0, 0)]]
[[format("rgba32f")]]
RWTexture2D<float4> in_hsva;

[[vk::binding(1, 0)]]
[[format("rgba8ui")]]
RWTexture2D<uint4> out_rgba;

[shader("compute")]
[numthreads(workgroup_x, workgroup_y, workgroup_z)]
void main(uint3 threadId : SV_DispatchThreadID)
{
    uint2 coords = threadId.xy;
    uint width, height;
    out_rgba.GetDimensions(width, height);

    if (coords.x >= width || coords.y >= height) {
        return;
    }

    float4 HSVA = in_hsva[coords];
    uint4 RGBA = color_hsva2rgba(HSVA);

    out_rgba[coords] = RGBA;
}

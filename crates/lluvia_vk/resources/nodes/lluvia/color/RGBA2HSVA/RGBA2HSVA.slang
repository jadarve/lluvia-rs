import lluvia.core;
import lluvia.color;

struct PushConsts
{
    float min_chroma;
};

[[vk::push_constant]]
PushConsts pushConsts;

[[vk::binding(0, 0)]]
[[format("rgba8ui")]]
RWTexture2D<uint4> in_rgba;

[[vk::binding(1, 0)]]
[[format("rgba32f")]]
RWTexture2D<float4> out_hsva;

[shader("compute")]
[numthreads(workgroup_x, workgroup_y, workgroup_z)]
void main(uint3 threadId : SV_DispatchThreadID)
{
    uint2 coords = threadId.xy;
    uint width, height;
    out_hsva.GetDimensions(width, height);

    if (coords.x >= width || coords.y >= height) {
        return;
    }

    uint4 RGBA = in_rgba[coords];
    float4 HSVA = color_rgba2hsva(RGBA, pushConsts.min_chroma);

    out_hsva[coords] = HSVA;
}

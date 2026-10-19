import lluvia.core;

struct PushConsts
{
    float min_value;
    float max_value;
    float alpha;
    float reverse;
};

[[vk::push_constant]]
PushConsts pushConsts;

[[vk::binding(0, 0)]]
[[format("r32ui")]]
RWTexture2D<uint> in_image;

[[vk::binding(1, 0)]]
[[format("rgba8ui")]]
Sampler1D<uint4> in_colormap;

[[vk::binding(2, 0)]]
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

    uint imageValue = in_image[coords];
    float normalizedValue = clamp(((float)imageValue - pushConsts.min_value) / (pushConsts.max_value - pushConsts.min_value), 0.0f, 1.0f);
    normalizedValue = pushConsts.reverse == 1.0f ? 1.0f - normalizedValue : normalizedValue;

    uint4 colorSample = in_colormap.SampleLevel(normalizedValue, 0.0f);
    uint4 rgba = uint4(colorSample.xyz, uint(pushConsts.alpha * 255.0f));

    out_rgba[coords] = rgba;
}

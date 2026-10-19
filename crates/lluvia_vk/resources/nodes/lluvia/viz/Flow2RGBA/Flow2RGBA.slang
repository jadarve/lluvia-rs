import lluvia.core;
import lluvia.color;

struct PushConsts
{
    float max_flow;
};

[[vk::push_constant]]
PushConsts pushConsts;

[[vk::binding(0, 0)]]
[[format("rg32f")]]
RWTexture2D<float2> in_flow;

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

    float2 flow = in_flow[coords];
    bool isInvalid = any(isinf(flow)) || any(isnan(flow));

    float normNormalized = length(flow) / pushConsts.max_flow;
    float normClamped = clamp(normNormalized, 0.0f, 1.0f);

    // * Saturation increases linearly with the norm
    // * Value increases logaritmically with the norm.
    // * For norm equal zero, the rgb value is black
    // * For vector norms greater than max_flow, set value to 0.8.
    // * Invalid pixels are coloured white.

    float4 hsva = isInvalid ? float4(0.0f, 0.0f, 1.0f, 1.0f) : float4(3.14159265f + atan2(-flow.y, -flow.x), normClamped, normNormalized <= 1.0f ? 0.3f * log(normClamped) + 1.0f : 0.8f, 1.0f);

    uint4 rgba = normClamped < 1e-3f ? uint4(0, 0, 0, 255) : color_hsva2rgba(hsva);

    out_rgba[coords] = rgba;
}

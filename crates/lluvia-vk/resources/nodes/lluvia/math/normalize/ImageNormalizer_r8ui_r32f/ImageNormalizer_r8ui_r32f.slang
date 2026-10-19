import lluvia.core;

struct PushConsts
{
    float max_value;
};

[[vk::push_constant]]
PushConsts pushConsts;

[[vk::binding(0, 0)]]
[[format("r8ui")]]
RWTexture2D<uint> in_gray;

[[vk::binding(1, 0)]]
[[format("r32f")]]
RWTexture2D<float> out_gray;

[shader("compute")]
[numthreads(workgroup_x, workgroup_y, workgroup_z)]
void main(uint3 threadId : SV_DispatchThreadID)
{
    uint2 coords = threadId.xy;
    uint width, height;
    out_gray.GetDimensions(width, height);

    if (coords.x >= width || coords.y >= height) {
        return;
    }

    float in_value = (float)in_gray[coords];
    float max_value = pushConsts.max_value;

    float out_value = max_value > 0.0 ? in_value / max_value : in_value;

    out_gray[coords] = out_value;
}

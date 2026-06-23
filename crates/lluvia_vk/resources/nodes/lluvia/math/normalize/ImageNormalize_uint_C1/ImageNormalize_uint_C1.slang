import lluvia.core;

struct PushConsts
{
    float max_value;
};

[[vk::push_constant]]
PushConsts pushConsts;

[[vk::binding(0, 0)]]
[[format("r32ui")]]
RWTexture2D<uint> in_image_uint;

[[vk::binding(1, 0)]]
[[format("r32f")]]
RWTexture2D<float> out_image_float;

[shader("compute")]
[numthreads(workgroup_x, workgroup_y, workgroup_z)]
void main(uint3 threadId : SV_DispatchThreadID)
{
    uint2 coords = threadId.xy;
    uint width, height;
    out_image_float.GetDimensions(width, height);

    if (coords.x >= width || coords.y >= height) {
        return;
    }

    float in_value = (float)in_image_uint[coords];
    float max_value = pushConsts.max_value;

    float out_value = max_value > 0.0 ? in_value / max_value : in_value;

    out_image_float[coords] = out_value;
}

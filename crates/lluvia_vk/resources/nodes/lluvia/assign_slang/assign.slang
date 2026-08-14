import lluvia.core;

struct PushConsts
{
    float offset;
};

[[vk::push_constant]]
PushConsts pushConsts;

[[vk::binding(0, 0)]]
RWStructuredBuffer<float> out_buffer;

[shader("compute")]
[numthreads(workgroup_x, workgroup_y, workgroup_z)]
void main(uint3 threadId : SV_DispatchThreadID)
{
    uint index = threadId.x;
    out_buffer[index] = (float)index + pushConsts.offset;
}

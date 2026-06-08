#define LLUVIA_LOCAL_SIZE \
    [[vk::constant_id(1)]] const uint local_size_x = 1; \
    [[vk::constant_id(2)]] const uint local_size_y = 1; \
    [[vk::constant_id(3)]] const uint local_size_z = 1;

struct PushConsts
{
    float offset;
};

[[vk::push_constant]]
PushConsts pushConsts;

[[vk::binding(0, 0)]]
RWStructuredBuffer<float> out_buffer;

LLUVIA_LOCAL_SIZE

[shader("compute")]
[numthreads(local_size_x, local_size_y, local_size_z)]
void main(uint3 threadId : SV_DispatchThreadID)
{
    uint index = threadId.x;
    out_buffer[index] = (float)index + pushConsts.offset;
}

import lluvia.core;

[[vk::binding(0, 0)]]
[[format("rgba8ui")]]
RWTexture2D<uint4> in_rgba;

[[vk::binding(1, 0)]]
[[format("rgba8ui")]]
RWTexture2D<uint4> out_bgra;

[shader("compute")]
[numthreads(workgroup_x, workgroup_y, workgroup_z)]
void main(uint3 threadId : SV_DispatchThreadID)
{
    uint2 coords = threadId.xy;
    uint width, height;
    out_bgra.GetDimensions(width, height);

    if (coords.x >= width || coords.y >= height) {
        return;
    }

    uint4 RGBA = in_rgba[coords];
    out_bgra[coords] = RGBA.zyxw;
}

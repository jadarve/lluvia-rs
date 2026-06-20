#include <lluvia/core.slang>
#include <lluvia/camera.slang>

struct PushConsts
{
    int camera_model;
};

[[vk::push_constant]]
PushConsts pushConsts;

[[vk::binding(0, 0)]]
[[format("rgba8ui")]]
Sampler2D<uint4> in_rgba;

struct UniformInCamera
{
    ll_camera camera;
};

[[vk::binding(1, 0)]]
ConstantBuffer<UniformInCamera> in_camera;

[[vk::binding(2, 0)]]
[[format("rgba8ui")]]
RWTexture2D<uint4> out_rgba;

LLUVIA_WORKGROUP

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

    float2 pixCoords = (float2)coords;
    if (pushConsts.camera_model == 0) {
        pixCoords = ll_camera_standardInterpolationCoordinates(in_camera.camera, (float2)coords);
    } else {
        pixCoords = ll_camera_fisheyeInterpolationCoordinates(in_camera.camera, (float2)coords);
    }

    const uint4 RGBA = in_rgba.SampleLevel(pixCoords, 0.0);
    out_rgba[coords] = RGBA;
}

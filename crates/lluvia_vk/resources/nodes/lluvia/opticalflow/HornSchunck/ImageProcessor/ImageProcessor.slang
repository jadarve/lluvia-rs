import lluvia.core;

struct PushConsts
{
    float alpha_square;
};

[[vk::push_constant]]
PushConsts pushConsts;

[[vk::binding(0, 0)]]
[[format("r8ui")]]
RWTexture2D<uint> in_gray;

[[vk::binding(1, 0)]]
[[format("r32f")]]
RWTexture2D<float> in_gray_old;

[[vk::binding(2, 0)]]
[[format("rgba32f")]]
RWTexture2D<float4> out_image_params;

[shader("compute")]
[numthreads(workgroup_x, workgroup_y, workgroup_z)]
void main(uint3 threadId : SV_DispatchThreadID)
{
    uint2 coords = threadId.xy;
    uint width, height;
    out_image_params.GetDimensions(width, height);

    if (coords.x >= width || coords.y >= height) {
        return;
    }

    float alpha_square = pushConsts.alpha_square;

    bool rightBorder  = (coords.x == width - 1);
    bool bottomBorder = (coords.y == height - 1);

    // current image
    float img_00 = (float)in_gray[coords] / 255.0;
    float img_01 = bottomBorder ? img_00 : (float)in_gray[coords + uint2(0, 1)] / 255.0;
    float img_10 = rightBorder ? img_00 : (float)in_gray[coords + uint2(1, 0)] / 255.0;
    float img_11 = rightBorder || bottomBorder ? img_00 : (float)in_gray[coords + uint2(1, 1)] / 255.0;

    // old image
    float img_old_00 = in_gray_old[coords];
    float img_old_01 = bottomBorder ? img_old_00 : in_gray_old[coords + uint2(0, 1)];
    float img_old_10 = rightBorder ? img_old_00 : in_gray_old[coords + uint2(1, 0)];
    float img_old_11 = rightBorder || bottomBorder ? img_old_00 : in_gray_old[coords + uint2(1, 1)];

    // Normalized image gradient and smoothed output
    float grad_x = 0.25 * ((img_10 - img_00) + (img_11 - img_01) + (img_old_10 - img_old_00) + (img_old_11 - img_old_01));
    float grad_y = 0.25 * ((img_01 - img_00) + (img_11 - img_10) + (img_old_01 - img_old_00) + (img_old_11 - img_old_10));

    // Temporal difference
    float imgTimeDifference = 0.25 * (img_00 + img_01 + img_10 + img_11 - img_old_00 - img_old_01 - img_old_10 - img_old_11);

    // Regularization
    float regularization = 1.0 / (alpha_square + grad_x * grad_x + grad_y * grad_y);

    float4 outputValue = float4(grad_x, grad_y, imgTimeDifference, regularization);

    out_image_params[coords] = outputValue;
}

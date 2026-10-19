import lluvia.core;

[[vk::binding(0, 0)]]
[[format("r8ui")]]
RWTexture2D<uint> in_gray;

[[vk::binding(1, 0)]]
[[format("r8ui")]]
RWTexture2D<uint> out_gray;

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

    bool topBorder    = (coords.y == 0);
    bool bottomBorder = (coords.y == height - 1);

    // image coordinates for in_gray (twice the height of out_gray)
    uint2 inputImgCoords = uint2(coords.x, 2 * coords.y);

    uint img_0 = in_gray[inputImgCoords];

    uint img_m = topBorder ? img_0 : in_gray[inputImgCoords + int2(0, -1)];
    uint img_p = bottomBorder ? img_0 : in_gray[inputImgCoords + int2(0, 1)];

    // this is equivalent to 0.5*img_0 + 0.25*(img_m + img_p)
    uint filterValue = (img_0 >> 1) + ((img_m + img_p) >> 2);

    out_gray[coords] = filterValue;
}

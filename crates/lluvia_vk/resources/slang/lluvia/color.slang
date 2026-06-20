module color;

/**
 * @brief Converts an RGBA unsigned integer color (uint4) to grayscale.
 *
 * Uses standard luminance coefficients to calculate grayscale intensity:
 *   gray = 0.299 * R + 0.587 * G + 0.114 * B
 *
 * @param RGBA The input color vector with values in range [0, 255].
 * @return The calculated grayscale intensity.
 */
public uint color_rgba2gray(uint4 RGBA)
{
    float4 rgba_f = (float4)RGBA;
    return (uint)dot(rgba_f, float4(0.29899999f, 0.58700001f, 0.114f, 0.0f));
}

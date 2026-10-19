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

/**
 * @brief Converts from RGBA (uint8) to HSVA color space.
 *
 * @param RGBA The rgba color in range [0, 255]
 * @param minChroma The minimum chroma in range [0, 1]
 * @return the HSVA value:
 *             * H in [0, 2*pi]
 *             * S in [0, 1]
 *             * V in [0, 1]
 *             * A in [0, 1]
 */
public float4 color_rgba2hsva(uint4 RGBA, float minChroma)
{
    float r = (float)RGBA.r / 255.0f;
    float g = (float)RGBA.g / 255.0f;
    float b = (float)RGBA.b / 255.0f;
    float a = (float)RGBA.a / 255.0f;

    uint M_ui = max(RGBA.r, max(RGBA.g, RGBA.b));

    float M = max(r, max(g, b));
    float m = min(r, min(g, b));
    float C = M - m;

    float H = 0.0f;
    if (C > minChroma) {
        if (M_ui == RGBA.r) {
            float val = (g - b) / C;
            H = val - 6.0f * floor(val / 6.0f);
        } else if (M_ui == RGBA.g) {
            H = ((b - r) / C) + 2.0f;
        } else {
            H = ((r - g) / C) + 4.0f;
        }
    }

    H = radians(60.0f * H);
    float V = M;
    float S = (V == 0.0f) ? 0.0f : (C / V);

    return float4(H, S, V, a);
}

/**
 * @brief Converts from HSVA color space to RGBA (uint8).
 *
 * @param HSVA The hsva color:
 *                 * H in [0, 2*pi]
 *                 * S in [0, 1]
 *                 * V in [0, 1]
 *                 * A in [0, 1]
 * @return The RGBA color value with components in range [0, 255].
 */
public uint4 color_hsva2rgba(float4 HSVA)
{
    float h = degrees(HSVA.x);
    float s = HSVA.y;
    float v = HSVA.z;
    float a = HSVA.w;

    float c = v * s;
    float val2 = h / 60.0f;
    float x = c * (1.0f - abs((val2 - 2.0f * floor(val2 / 2.0f)) - 1.0f));
    float m = v - c;

    float3 rgb = float3(0.0f);
    if (0.0f <= h && h < 60.0f) {
        rgb = float3(c, x, 0.0f);
    } else if (60.0f <= h && h < 120.0f) {
        rgb = float3(x, c, 0.0f);
    } else if (120.0f <= h && h < 180.0f) {
        rgb = float3(0.0f, c, x);
    } else if (180.0f <= h && h < 240.0f) {
        rgb = float3(0.0f, x, c);
    } else if (240.0f <= h && h < 300.0f) {
        rgb = float3(x, 0.0f, c);
    } else {
        rgb = float3(c, 0.0f, x);
    }

    float3 rgb_final = clamp(255.0f * (rgb + m), 0.0f, 255.0f);
    uint alpha = (uint)clamp(255.0f * a, 0.0f, 255.0f);

    return uint4((uint)rgb_final.r, (uint)rgb_final.g, (uint)rgb_final.b, alpha);
}

#ifndef LLUVIA_CAMERA_SLANG_
#define LLUVIA_CAMERA_SLANG_

struct ll_camera
{
    float3x3 K;
    float3x3 Kinv;
    float4 radialDistortion;
    float4 tangentialDistortion;
};

float2 ll_camera_standardInterpolationCoordinates(const ll_camera cam, const float2 pixCoords)
{
    const float3 normalizedCoords = mul(cam.Kinv, float3(pixCoords.xy, 1.0));

    ///////////////////////////////////////////////////////
    // Radial distortion
    ///////////////////////////////////////////////////////

    const float k1 = cam.radialDistortion.x;
    const float k2 = cam.radialDistortion.y;
    const float k3 = cam.radialDistortion.z;

    // image plane coordinate radius squared
    const float r2 = dot(normalizedCoords.xy, normalizedCoords.xy);

    // radial distortion coefficient
    const float R = (k1 * r2) + (k2 * r2 * r2) + (k3 * r2 * r2 * r2);

    // radial distortion coordinates
    const float2 Pradial = normalizedCoords.xy * (1.0 + R);

    ///////////////////////////////////////////////////////
    // Tangential distortion
    ///////////////////////////////////////////////////////
    const float p1 = cam.tangentialDistortion.x;
    const float p2 = cam.tangentialDistortion.y;

    const float2 Ptangential = float2(
        (2.0 * p1 * normalizedCoords.x * normalizedCoords.y) + p2 * (r2 + 2.0 * normalizedCoords.x * normalizedCoords.x),
        (p1 * (r2 + 2.0 * normalizedCoords.y * normalizedCoords.y) + (2.0 * p2 * normalizedCoords.x * normalizedCoords.y))
    );

    // Add both distortions
    const float3 Pdistorted = float3(Pradial + Ptangential, 1.0);

    // Convert back to pixel coordinates
    const float3 outPixCoords = mul(cam.K, Pdistorted);
    return outPixCoords.xy;
}

/**
See See https://docs.opencv.org/3.4/db/d58/group__calib3d__fisheye.html for more details.
*/
float2 ll_camera_fisheyeInterpolationCoordinates(const ll_camera cam, const float2 pixCoords)
{
    const float3 normalizedCoords = mul(cam.Kinv, float3(pixCoords.xy, 1.0));

    // unpack the radial distortion coefficients
    const float k1 = cam.radialDistortion.x;
    const float k2 = cam.radialDistortion.y;
    const float k3 = cam.radialDistortion.z;
    const float k4 = cam.radialDistortion.w;

    // image plane coordinate radius squared
    const float r2 = dot(normalizedCoords.xy, normalizedCoords.xy);
    const float r  = sqrt(r2);

    // pixel angle
    const float theta = atan(r);

    const float theta2 = theta * theta;
    const float theta4 = theta2 * theta2;
    const float theta6 = theta4 * theta2;
    const float theta8 = theta4 * theta4;

    const float thetaDist = theta * (1.0 + k1 * theta2 + k2 * theta4 + k3 * theta6 + k4 * theta8);

    // possible division by zero
    const float scale = r > 1e-5 ? thetaDist / r : 0.0;

    // distorted coordinates
    const float3 Pdistorted = float3(scale * normalizedCoords.xy, 1.0);

    // Convert back to pixel coordinates
    const float3 outPixCoords = mul(cam.K, Pdistorted);
    return outPixCoords.xy;
}

#endif // LLUVIA_CAMERA_SLANG_

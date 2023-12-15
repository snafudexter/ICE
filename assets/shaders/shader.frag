#version 450

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec3 fragPosWorld;
layout(location = 2) in vec3 fragNormalWorld;

layout(location = 0) out vec4 outColor;

struct PointLight {
    vec4 position; // ignore w
    vec4 color; // w is intensity
};

layout(set = 0, binding = 0) uniform GlobalUbo {
    mat4 model_matrix;
    mat4 view;
    mat4 projection;
    vec4 ambient_light_color; // w is intensity
    vec4 camera_pos_world;
    PointLight light;
} ubo;

// layout(push_constant) uniform Push {
//   mat4 modelMatrix;
//   mat4 normalMatrix;
// } push;

void main() {
    vec3 ambientColor = ubo.ambient_light_color.w * ubo.ambient_light_color.xyz;
    vec3 specularLight = vec3(0.0);
    vec3 surfaceNormal = normalize(fragNormalWorld);

    vec3 cameraPosWorld = ubo.camera_pos_world.xyz;
    vec3 viewDirection = normalize(cameraPosWorld - fragPosWorld);

    vec3 directionToLight = ubo.light.position.xyz - fragPosWorld;
    float attenuation = 1.0 / dot(directionToLight, directionToLight); // distance squared
    directionToLight = normalize(directionToLight);

    float cosAngIncidence = max(dot(directionToLight, surfaceNormal), 0);
    vec3 intensity = ubo.light.color.xyz * ubo.light.color.w * attenuation;

    vec3 diffuseColor = cosAngIncidence * ubo.light.color.xyz;

    // specular lighting
    vec3 reflectDir = reflect(-directionToLight, surfaceNormal); // float blinnTerm = dot(surfaceNormal, halfAngle);
    float spec = pow(max(dot(viewDirection, reflectDir), 0.0), 32);
    vec3 specular = 0.5 * spec * ubo.light.color.xyz;

    outColor = vec4(ambientColor + diffuseColor + spec, 1.0);
}

// #version 450

// layout(location = 0) in vec3 fragColor;
// layout(location = 1) in vec3 fragPosWorld;
// layout(location = 2) in vec3 fragNormalWorld;

// layout(location = 0) out vec4 outColor;

// struct PointLight {
//     vec4 position; // ignore w
//     vec4 color; // w is intensity
// };

// layout(set = 0, binding = 0) uniform GlobalUbo {
//     mat4 projection;
//     mat4 view;
//     mat4 invView;
//     vec4 ambientLightColor; // w is intensity
//     PointLight pointLights[10];
//     int numLights;
// } ubo;

// // layout(push_constant) uniform Push {
// //   mat4 modelMatrix;
// //   mat4 normalMatrix;
// // } push;

// void main() {
//     vec3 diffuseLight = ubo.ambientLightColor.xyz * ubo.ambientLightColor.w;
//     vec3 specularLight = vec3(0.0);
//     vec3 surfaceNormal = normalize(fragNormalWorld);

//     vec3 cameraPosWorld = ubo.invView[3].xyz;
//     vec3 viewDirection = normalize(cameraPosWorld - fragPosWorld);

//     for(int i = 0; i < ubo.numLights; i++) {
//         PointLight light = ubo.pointLights[i];
//         vec3 directionToLight = light.position.xyz - fragPosWorld;
//         float attenuation = 1.0 / dot(directionToLight, directionToLight); // distance squared
//         directionToLight = normalize(directionToLight);

//         float cosAngIncidence = max(dot(surfaceNormal, directionToLight), 0);
//         vec3 intensity = light.color.xyz * light.color.w * attenuation;

//         diffuseLight += intensity * cosAngIncidence;

//     // specular lighting
//         vec3 halfAngle = normalize(directionToLight + viewDirection);
//         float blinnTerm = dot(surfaceNormal, halfAngle);
//         blinnTerm = clamp(blinnTerm, 0, 1);
//         blinnTerm = pow(blinnTerm, 512.0); // higher values -> sharper highlight
//         specularLight += intensity * blinnTerm;
//     }

//     outColor = vec4(fragColor, 1.0);//vec4(diffuseLight * fragColor + specularLight * fragColor, 1.0);
// }

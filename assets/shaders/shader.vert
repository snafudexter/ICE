// #version 410

// layout(location = 0) in vec3 position;
// layout(location = 1) in vec3 normal;

// uniform mat4 view_proj;
// uniform mat4 model;
// uniform mat4 light_space_matrix;

// out vec3 surfaceNormal;
// out vec4 worldPos;
// out vec4 fragPosLightSpace;

// void main() {
//     worldPos = model * vec4(position, 1);
//     surfaceNormal = (model * vec4(normal, 0.0)).xyz;
//     fragPosLightSpace = light_space_matrix * worldPos;
//     gl_Position = view_proj * worldPos;
// }

#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;
layout(location = 2) in vec3 normal;
// layout(location = 3) in vec2 uv;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec3 fragPosWorld;
layout(location = 2) out vec3 fragNormalWorld;

struct PointLight {
    vec3 position; // ignore w
    vec4 color; // w is intensity
};

layout(set = 0, binding = 0) uniform GlobalUbo {
    mat4 model_matrix;
    mat4 projection;
    mat4 view;
    mat4 inverse_view;
    vec4 ambient_light_color; // w is intensity
    PointLight point_lights[1];
    int numLights;
} ubo;

// layout(push_constant) uniform Push {
//     mat4 modelMatrix;
//     mat4 normalMatrix;
// } push;

void main() {
    vec4 positionWorld = ubo.model_matrix * vec4(position, 0.0);
    gl_Position = ubo.projection * ubo.view * positionWorld;
    fragNormalWorld = normalize(normal);
    fragPosWorld = positionWorld.xyz;
    //gl_Position = vec4(position.x, position.y, 0.0, 1.0);
    fragColor = color;
}

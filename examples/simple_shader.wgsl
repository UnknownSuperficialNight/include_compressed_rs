// This is just a generic example shader not even sure if any of the comments in here mean anything but
// the point is to show that the minification removes comments so they are sprinkled everywhere
//
// Standalone WGSL cube renderer.
// This shader requires no uniforms, textures, storage buffers, or external inputs.
// It renders a rotating cube using raymarching against a signed-distance field.

const PI: f32 = 3.14159265359;

// Vertex shader entry point.
// Emits a fullscreen triangle that covers the entire render target.
@vertex
fn vtx_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
    // Fullscreen triangle vertices in clip space.
    var pos = array<vec2f, 3>(
        vec2f(-1.0, -3.0),
        vec2f(3.0, 1.0),
        vec2f(-1.0, 1.0),
    );

    return vec4f(pos[vertex_index], 0.0, 1.0);
}

// Clamps a value to the [0, 1] range.
// Useful for lighting, shading, and smooth blending.
fn sat(x: f32) -> f32 {
    return clamp(x, 0.0, 1.0);
}

// Builds a 3D rotation matrix around the Y axis.
fn rot_y(a: f32) -> mat3x3f {
    let s = sin(a);
    let c = cos(a);

    return mat3x3f(
        vec3f(c, 0.0, -s),
        vec3f(0.0, 1.0, 0.0),
        vec3f(s, 0.0, c),
    );
}

// Builds a 3D rotation matrix around the X axis.
fn rot_x(a: f32) -> mat3x3f {
    let s = sin(a);
    let c = cos(a);

    return mat3x3f(
        vec3f(1.0, 0.0, 0.0),
        vec3f(0.0, c, s),
        vec3f(0.0, -s, c),
    );
}

// Signed-distance function for an axis-aligned box.
// The box is centered at the origin with half-extents given by `b`.
fn sd_box(p: vec3f, b: vec3f) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3f(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

// Scene distance function.
// Returns the distance from point `p` to the nearest surface in the scene.
// In this example, the scene contains a single cube.
fn scene(p: vec3f) -> f32 {
    return sd_box(p, vec3f(0.65, 0.65, 0.65));
}

// Estimates the surface normal at point `p` using finite differences.
// This is used for lighting calculations after a ray hits the cube.
fn normal(p: vec3f) -> vec3f {
    let e = 0.001;

    let x = scene(p + vec3f(e, 0.0, 0.0)) - scene(p - vec3f(e, 0.0, 0.0));
    let y = scene(p + vec3f(0.0, e, 0.0)) - scene(p - vec3f(0.0, e, 0.0));
    let z = scene(p + vec3f(0.0, 0.0, e)) - scene(p - vec3f(0.0, 0.0, e));

    return normalize(vec3f(x, y, z));
}

// Fragment shader entry point.
// Performs raymarching, shading, and final color shaping.
@fragment
fn frag_main(@builtin(position) coord: vec4f) -> @location(0) vec4f {
    // Fixed resolution used to derive normalized screen-space coordinates.
    let resolution = vec2f(800.0, 600.0);

    // Convert pixel coordinates into normalized device-like coordinates.
    let uv = (coord.xy / resolution) * 2.0 - 1.0;

    // Camera origin placed slightly in front of the cube.
    let ro = vec3f(0.0, 0.0, 3.0);

    // Camera direction through the current pixel.
    let rd = normalize(vec3f(uv.x * resolution.x / resolution.y, uv.y, -1.8));

    // Rotation phase used to animate the cube.
    let t = 0.75;

    // Combined cube rotation.
    let cube_rot = rot_y(t) * rot_x(t * 0.7);

    // Transform the camera ray into cube space.
    var ray_o = cube_rot * ro;
    var ray_d = cube_rot * rd;

    // Raymarching state.
    var dist = 0.0;
    var hit = false;

    // March along the ray until we hit the cube or exceed the maximum range.
    for (var i: i32 = 0; i < 96; i = i + 1) {
        let p = ray_o + ray_d * dist;
        let d = scene(p);

        if d < 0.001 {
            hit = true;
            break;
        }

        dist = dist + d;

        if dist > 20.0 {
            break;
        }
    }

    // Background color used when the ray misses the cube.
    var color = vec3f(0.04, 0.05, 0.08);

    if hit {
        // Surface position at the hit point.
        let p = ray_o + ray_d * dist;

        // Surface normal for lighting.
        let n = normal(p);

        // Directional light used for diffuse shading.
        let light_dir = normalize(vec3f(0.6, 0.8, 0.5));

        // Lambertian diffuse term.
        let diff = sat(dot(n, light_dir));

        // Specular highlight using a simple Blinn-Phong model.
        let view_dir = normalize(-ray_d);
        let half_dir = normalize(light_dir + view_dir);
        let spec = pow(sat(dot(n, half_dir)), 48.0);

        // Subtle color variation per face using the surface normal.
        let tint = 0.5 + 0.5 * abs(n);

        // Base shaded cube color.
        color = vec3f(0.15, 0.65, 1.0) * tint * diff;

        // Specular highlight.
        color = color + vec3f(1.0, 1.0, 1.0) * spec * 0.8;

        // Rim lighting to emphasize the cube silhouette.
        let rim = pow(1.0 - sat(dot(n, view_dir)), 2.0);
        color = color + vec3f(0.9, 0.4, 0.2) * rim * 0.35;

        // Simple depth-based attenuation for additional visual depth.
        color = color * (0.25 + 0.75 * sat(1.0 - dist * 0.03));
    }

    // Vignette to gently darken the image corners.
    let v = sat(1.0 - dot(uv, uv) * 0.25);
    color = color * v;

    // Basic tone mapping.
    color = color / (color + vec3f(1.0, 1.0, 1.0));

    // Final gamma-like correction.
    color = pow(color, vec3f(0.95, 0.95, 0.95));

    return vec4f(color, 1.0);
}
